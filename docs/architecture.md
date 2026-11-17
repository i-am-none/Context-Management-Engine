# Nexus CME - Architecture Specification

## 1. System Overview

Nexus CME (Context Management Engine) is a local-first memory and context management infrastructure designed to provide persistent, reliable, and structured context for AI agents across sessions.

Nexus CME decouples context retention, memory consolidation, and retrieval from ephemeral LLM inference sessions, maintaining an authoritative and traceable record of events.

---

## 2. Logical Components

```text
+-----------------------------------------------------------+
|                    External Clients                       |
|           (Python Agents / LangGraph / HTTP CLI)          |
+-----------------------------+-----------------------------+
                              | HTTP (JSON)
                              v
+-----------------------------------------------------------+
|                     Nexus CME Core                        |
|                                                           |
|  +-----------------------------------------------------+  |
|  |                     API Layer                       |  |
|  |  - Axum Router (src/api/mod.rs)                     |  |
|  |  - Endpoint Handlers (src/api/health.rs)            |  |
|  |  - Shared State (AppState / SqlitePool)             |  |
|  +---------------------------+-------------------------+  |
|                              |                            |
|  +---------------------------v-------------------------+  |
|  |                  Persistence Layer                  |  |
|  |  - Connection Pool (SqlitePool) (src/db/mod.rs)     |  |
|  |  - SQLite Pragmas: WAL, Foreign Keys, Busy Timeout  |  |
|  |  - Embedded Migrations (migrations/)                |  |
|  +---------------------------+-------------------------+  |
|                              |                            |
|  +---------------------------v-------------------------+  |
|  |                Configuration & Runtime              |  |
|  |  - Localhost-only binding (src/config.rs)           |  |
|  |  - Structured Logging (tracing / subscriber)        |  |
|  |  - Tokio Async Runtime                              |  |
|  +-----------------------------------------------------+  |
|                                                           |
|  [ Planned Future Logical Components ]                    |
|  - Event Ingestion Engine (Milestone 4)                   |
|  - Full-Text Retrieval & FTS5 Indexing (Milestone 5)      |
|  - Context Assembly & Budgeting (Milestone 6)             |
|  - Jev Decision Engine Integration (Milestone 7)          |
|  - Local LLM Generative Reasoning (Milestone 8)           |
+-----------------------------------------------------------+
```

---

## 3. Current Component Boundaries (Milestone NEXUS-003 / NEXUS-003.2)

### 3.1 API Layer (`src/api/`)
- **`src/api/mod.rs`**: Defines `AppState` holding the `SqlitePool`, assembles the Axum router, and exposes `app_router(state)`.
- **`src/api/health.rs`**: Implements `GET /health` with active database connectivity verification. Returns HTTP 200 with `"database": "connected"` when available, or HTTP 503 with `"database": "disconnected"` if unavailable.

### 3.2 Persistence Layer (`src/db/` & `migrations/`)
- **`src/db/mod.rs`**: Manages the SQLite database pool using SQLx. Configures connection options for every connection:
  - `PRAGMA journal_mode = WAL`: Concurrency model where readers observe point-in-time consistent snapshots without blocking the single active writer. Readers see committed frames applicable to their snapshot, never uncommitted transactions. Committed frames in the `-wal` file are periodically merged back into the database file via checkpointing; long-held read snapshots can prevent checkpoints from truncating the WAL, leading to WAL file growth.
  - `PRAGMA synchronous = NORMAL`: Prevents database structural corruption across application crashes while providing significant write performance improvements over `FULL`. In the event of a sudden power outage or OS crash before OS disk buffers flush, recent committed transactions may be lost.
  - `PRAGMA foreign_keys = ON`: Enforces relational integrity across parent and child entities.
  - `PRAGMA busy_timeout = 5000`: Retries lock contention up to 5,000 milliseconds before returning `SQLITE_BUSY`, returning earlier if the lock is freed.
- Provides `init_db`, `init_pool`, `verify_pragmas`, `run_migrations`, and `check_connection`.
- **`migrations/`**: Embedded SQL migrations managed with `sqlx::migrate!`. Migration version tracking is handled authoritatively by SQLx in `_sqlx_migrations`.

### 3.3 Configuration Layer (`src/config.rs`)
- Manages host address, port, `database_url`, and `max_connections`.
- Provides deterministic default values (`127.0.0.1:3000`, `sqlite://nexus.db`, `max_connections: 5`) and environment overrides via `Config::from_env()`.
- Validates configuration boundaries (`max_connections >= 1`, non-empty database URL).
- Enforces localhost binding exclusively to `127.0.0.1:3000`.

### 3.4 Library & Application Entrypoint (`src/lib.rs` & `src/main.rs`)
- **`src/lib.rs`**: Exposes `api`, `config`, and `db` modules as the core crate interface for integration testing and external consumers.
- **`src/main.rs`**: Initializes structured logging with `tracing` and `tracing-subscriber`. Initializes the SQLite database and executes migrations during startup. Injects `AppState { db: db_pool }` into the Axum router. Binds `tokio::net::TcpListener` to `127.0.0.1:3000` and serves via `axum::serve`.

---

## 4. Planned Components & Interfaces

1. **Event Ingestion Layer (Milestone 4):**
   - Ingestion endpoints for raw interaction events.
   - Durability guarantees before ingestion acknowledgment.
2. **Retrieval Layer (Milestone 5):**
   - Full-text search and historical memory indexing via SQLite FTS5.
3. **Decision Layer (TypeSafe AI Jev) (Milestone 7):**
   - Structured probabilistic assessments (relevance scoring, memory classification).
4. **Generative Reasoning Layer (llama.cpp / Qwen) (Milestone 8):**
   - Memory synthesis, conflict resolution, and generative summarization.
5. **Agent Integration Layer (Python) (Milestone 9):**
   - External orchestration through documented HTTP endpoints.

---

## 5. Security & Boundary Guarantees

- **Local-first operation:** The service binds exclusively to `127.0.0.1`.
- **Data Integrity:** Strict foreign keys and WAL mode guarantee transactional consistency and durability.
- **Traceability:** Original events are preserved; derived memories maintain provenance.
- **Strict Scope Discipline:** Future capabilities are deferred until their respective approved milestones.
