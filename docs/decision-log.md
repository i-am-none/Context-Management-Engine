# Nexus CME - Architectural Decision Log

This document records the architectural decisions, context, and rationales for Nexus CME in accordance with `AGENTS.md`.

---

## ADR-001: Project Toolchain Management via mise

- **Date:** 2026-11-13
- **Status:** Accepted
- **Context:**
  Nexus CME requires a stable Rust toolchain (`rustc`, `cargo`, `rustfmt`, `clippy`). The host environment utilizes `mise` for developer environment management.
- **Decision:**
  Use a project-level `.mise.toml` specifying `rust = "stable"` rather than mutating global configurations.
- **Consequences:**
  - Guarantees reproducible, hermetic toolchain versions across development environments.
  - Keeps global configurations clean.

---

## ADR-002: Modular Monolith vs. Microservices Architecture

- **Date:** 2026-11-13
- **Status:** Accepted
- **Context:**
  Context management engines often face premature decomposition into multiple microservices (e.g. separate ingest, embedding, retrieval, and decision services).
- **Decision:**
  Develop Nexus CME initially as a single modular Rust binary (`nexus-cme`). Internal concerns are cleanly separated into modules (`api`, `config`, and future storage/processing modules).
- **Consequences:**
  - Significantly reduces deployment complexity, operational overhead, and IPC latency.
  - Preserves flexibility for future refactoring when measurable requirements justify decomposition.

---

## ADR-003: Axum and Tokio for HTTP Service Layer

- **Date:** 2026-11-13
- **Status:** Accepted
- **Context:**
  The core service requires a fast, ergonomic, and asynchronous HTTP API server to interface with Python agents and management tools.
- **Decision:**
  Use `axum` (0.8) and `tokio` (1.43) as the foundational asynchronous HTTP server framework, along with `serde` for JSON serialization and `tracing` for structured telemetry.
- **Consequences:**
  - High performance and safety with minimal boilerplate.
  - First-class support for modular routing and middleware.

---

## ADR-004: Strict Localhost Network Binding

- **Date:** 2026-11-13
- **Status:** Accepted
- **Context:**
  Security guidelines in `AGENTS.md` require avoiding accidental exposure of local service endpoints and credentials.
- **Decision:**
  Enforce explicit binding exclusively to `127.0.0.1:3000` via application configuration (`src/config.rs`).
- **Consequences:**
  - The service is not reachable from public or external network interfaces by default.

---

## ADR-005: Decoupled Health Endpoint and Honest State Reporting

- **Date:** 2026-11-13
- **Status:** Accepted
- **Context:**
  A health check endpoint (`GET /health`) is required to verify operational status. Database functionality has not yet been introduced.
- **Decision:**
  - Place `health_handler` into `src/api/health.rs`, decoupled from `src/main.rs`.
  - Accurately report `"database": "not_initialized"` in the JSON response rather than assuming or mocking connectivity.
- **Consequences:**
  - Satisfies the principle of honest reporting and explicit state transitions across milestones.

---

## ADR-006: SQLite Access Library Selection (SQLx vs. rusqlite) and Persistence Configuration

- **Date:** 2026-11-14
- **Status:** Accepted (Updated in NEXUS-003.1)
- **Context:**
  Nexus CME requires a persistent, local-first SQLite storage layer supporting WAL mode, foreign keys, concurrency, and future FTS5 full-text indexing. The host application runs on the Tokio async runtime and Axum HTTP framework. Two primary libraries were evaluated: `sqlx` and `rusqlite`.
  
  **Evaluation Comparison:**
  1. *Async Compatibility:* `sqlx` is native asynchronous and integrates seamlessly with Tokio without blocking event loops. `rusqlite` is synchronous, requiring `tokio::task::spawn_blocking` wrappers or manual worker thread channels for every query.
  2. *Connection Pooling & Concurrency:* `sqlx` provides built-in `SqlitePool` with pool sizing, acquire timeouts, and connection lifecycle management. `rusqlite` requires external pooling crates (such as `r2d2` or `deadpool-sqlite`).
  3. *Transaction Support:* `sqlx` provides first-class async transactions (`pool.begin().await`, automatic rollback on drop). `rusqlite` transactions cannot safely span async await points without complex thread pin/actor models.
  4. *Migration Management:* `sqlx` provides compile-time verified embedded migrations (`sqlx::migrate!`), avoiding third-party migration crates. `rusqlite` requires additional dependencies (such as `refinery`).
  5. *FTS5 & SQLite Features:* Both libraries support FTS5. In runtime verification, SQLx bundles SQLite (version 3.51.3) with `sqlite_compileoption_used('ENABLE_FTS5') = 1`, and virtual tables using FTS5 execute successfully.
  6. *Dependency & Maintenance:* While `rusqlite` has a smaller isolated footprint, adding pooling and migration dependencies levels the field. `sqlx` offers a unified, active, and cohesive async ecosystem for Axum/Tokio.
- **Decision:**
  Select `sqlx` (v0.9.0) with features `runtime-tokio`, `sqlite`, `macros`, and `migrate`.
  Enforce the following SQLite pragmas on pool initialization:
  - `PRAGMA journal_mode = WAL;`:
    - **Concurrency Model:** Readers operate on point-in-time consistent snapshots. Readers can observe committed WAL frames applicable to their snapshot, but never uncommitted transactions.
    - **Single Writer Constraint:** SQLite permits only one active writer at any time across all connections.
    - **Checkpointing & WAL Size:** Committed frames written to the `-wal` file are periodically transferred back into the main database file via checkpoints. If long-running read transactions hold open old snapshots, checkpointing cannot advance past those frames, potentially allowing the WAL file to grow until the read transactions finish.
  - `PRAGMA synchronous = NORMAL;`:
    - **Consistency vs. Durability Trade-off:** With WAL mode, `NORMAL` guarantees that the database file structure will never become corrupt as a result of an application crash. However, in the event of an abrupt host power failure or operating system crash before dirty OS disk buffers are flushed, the most recently committed transaction(s) could be lost. This trade-off significantly improves write throughput and reduce I/O wear while preserving ACID consistency.
  - `PRAGMA foreign_keys = ON;`:
    - Explicitly enabled for every connection to ensure strict referential integrity.
  - `PRAGMA busy_timeout = 5000;`:
    - Sets a retry loop up to 5,000 milliseconds for lock contention scenarios before returning `SQLITE_BUSY`. If the lock becomes available earlier, the query proceeds immediately. This does not guarantee every contention waits the full duration (e.g., deadlock detection or same-connection table locks return immediately).
- **Consequences:**
  - Asynchronous handlers interact with the database without blocking Tokio threads.
  - Built-in schema migrations are compiled directly into the binary and run on startup.
  - Pragmas are verified on startup before the service accepts traffic.

---

## ADR-007: Elimination of Redundant Schema Metadata Table

- **Date:** 2026-11-14
- **Status:** Accepted
- **Context:**
  The initial foundation migration introduced a custom `schema_metadata` table to store `schema_version`. However, SQLx's migration framework already establishes and manages the authoritative `_sqlx_migrations` table, which tracks migration version numbers, descriptions, checksums, and execution timestamps.
- **Decision:**
  Remove `schema_metadata` from the baseline migration. Rely exclusively on SQLx's built-in `_sqlx_migrations` for migration state and version tracking until concrete domain requirements demand an application-level metadata table.
- **Consequences:**
  - Eliminates duplicate state and unnecessary boilerplate schema.
  - Schema migrations remain clean and lean.
