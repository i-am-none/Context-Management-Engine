# Nexus CME

**Nexus Context Management Engine**

A local-first context management and persistent memory infrastructure for AI agents.

Nexus CME aims to help AI agents maintain continuity across sessions, retrieve relevant historical information, manage context efficiently and reduce unnecessary LLM inference.

The project is being developed with Rust at its core, Python for agent orchestration, Jev for structured decisions and local LLMs for generative reasoning.

---

## 1. Project Vision

Current AI agents often depend heavily on the context supplied during a single interaction.

As conversations and tasks grow, they encounter several problems:

- Limited context windows.
- Repeated processing of historical conversations.
- Loss of useful information across sessions.
- Irrelevant context entering prompts.
- Difficulty maintaining consistent knowledge.
- Expensive or unnecessary LLM calls.
- Poor separation between original events and generated summaries.

Nexus CME aims to address these problems through a dedicated context management layer.

Instead of relying exclusively on an LLM to remember everything, Nexus CME will maintain a persistent, searchable and structured memory system.

The long-term objective is to make this engine reusable across different agent frameworks and model providers.

## 2. Core Design Philosophy

Nexus CME follows five principles:

1. **Persistent memory:** Preserve useful information beyond individual conversations.
2. **Intelligent retrieval:** Retrieve information relevant to the current task.
3. **Efficient context:** Send only useful information to the reasoning model.
4. **Separation of responsibilities:** Use deterministic code, decision models and generative LLMs for different jobs.
5. **Traceability:** Preserve original events and the provenance of derived memories.

## 3. Planned Architecture

The initial architecture consists of one modular Rust service.

### Rust Core

Responsible for:

- Event ingestion.
- API handling.
- Persistent storage.
- Memory management.
- Full-text retrieval.
- Context assembly.
- Token budget enforcement.
- Background processing.

### SQLite Storage

Planned responsibilities:

- Original event storage.
- Structured memory records.
- Memory relationships.
- Processing job tracking.
- Full-text indexing through FTS5.

### Jev Decision Layer

Jev from TypeSafe AI is intended to provide structured decisions for tasks such as:

- Memory classification.
- Relevance assessment.
- Importance scoring.
- Decision routing.

Application policies will validate and enforce the resulting decisions.

### Local LLM Layer

llama.cpp and local Qwen models will provide:

- Complex reasoning.
- Memory summarization.
- Conflict interpretation.
- Contextual understanding.
- Generative responses where required.

### Python Agent Integration

Python and LangGraph / Deep Agents will eventually communicate with Nexus CME through documented APIs.

This allows the memory system to remain independent of a particular agent framework.

## 4. Technology Stack

| Component             | Technology              |
| --------------------- | ----------------------- |
| Core language         | Rust                    |
| Async runtime         | Tokio                   |
| HTTP framework        | Axum                    |
| Serialization         | Serde                   |
| Logging               | tracing                 |
| Database              | SQLite                  |
| Database search       | FTS5                    |
| Agent orchestration   | Python                  |
| Agent framework       | LangGraph / Deep Agents |
| Decision intelligence | TypeSafe AI Jev         |
| Local inference       | llama.cpp               |
| Generative model      | Qwen                    |

Additional technologies may be introduced when justified by actual requirements and evaluation.

## 5. Project Structure

The initial repository is organized around:

```text
nexus-cme/
├── Cargo.toml
├── Cargo.lock
├── .mise.toml
├── .gitignore
├── README.md
├── AGENTS.md
│
├── src/
│   ├── lib.rs
│   ├── main.rs
│   ├── config.rs
│   ├── db/
│   │   └── mod.rs
│   └── api/
│       ├── mod.rs
│       └── health.rs
│
├── migrations/
│   └── 20261114000000_init.sql
│
├── docs/
│   ├── architecture.md
│   └── decision-log.md
│
└── tests/
    └── db_test.rs
```

The structure will expand as new modules are implemented.

## 6. Development Roadmap

Development will happen incrementally.

| Stage | Objective                                   | Status                              |
| ----- | ------------------------------------------- | ----------------------------------- |
| 1     | Project foundation and environment          | Completed (NEXUS-001)               |
| 2     | Rust service and HTTP API                   | Completed (NEXUS-002)               |
| 2.1   | Quality gate and documentation corrections  | Completed (NEXUS-002.1)             |
| 3     | SQLite persistent storage                   | Completed (NEXUS-003, NEXUS-003.1)  |
| 3.2   | Migration integrity and baseline freeze     | Completed (NEXUS-003.2)             |
| 4     | Event ingestion                             | Planned                             |
| 5     | Full-text retrieval                         | Planned                    |
| 6     | Context assembly and token budgeting        | Planned                    |
| 7     | Jev decision integration                    | Planned                    |
| 8     | Local Qwen integration                      | Planned                    |
| 9     | Python agent integration                    | Planned                    |
| 10    | Semantic retrieval and memory consolidation | Planned                    |
| 11    | Evaluation and optimization                 | Planned                    |

Features will be implemented only when the previous milestones provide a reliable foundation.

## 7. Development Environment

### Operating system

Initial development environment:

- Linux x86_64
- Omarchy Linux

### Prerequisites

Prerequisites:

- Rust stable toolchain (managed via mise: `rustc 1.100.0`, `cargo 1.100.0`).
- Git.
- SQLite with FTS5 support.
- curl.
- mise for project toolchain management.

Python will be required when agent integration begins.

### Installation

The project uses the project-level `.mise.toml` to manage the Rust toolchain:

```bash
mise install
cargo build
```

## 8. Running the Service

The initial service is configured to run locally at:

```text
http://127.0.0.1:3000
```

It exposes:

```http
GET /health
```

Expected response when database connectivity is verified:

```json
{
  "status": "ok",
  "service": "nexus-cme",
  "database": "connected"
}
```

If database connectivity fails or the pool is unavailable, the endpoint returns HTTP 503 Service Unavailable with `"database": "disconnected"`.

## 9. Engineering and Testing

Nexus CME will prioritize:

- Modular Rust code.
- Explicit error handling.
- Automated tests.
- Database integrity.
- API validation.
- Structured logging.
- Performance measurements.
- Reproducible development environments.

Performance improvements will be evaluated using measurable outcomes rather than assumptions.

Important metrics will include:

- Retrieval precision and recall.
- Context token consumption.
- API latency.
- Decision-model latency.
- Memory classification quality.
- Overall agent task performance.

## 10. Security and Privacy

Nexus CME is designed as a local-first system.

The project will:

- Bind local development services to localhost.
- Protect credentials.
- Avoid committing secrets.
- Preserve original event records.
- Validate external model responses.
- Redact sensitive information before external API calls.
- Avoid automatic deletion based on uncertain model judgments.

External AI providers may process information outside the local machine. Such integrations must be designed with explicit privacy controls.

## 11. Current Development Status

**Current milestones completed: NEXUS-001, NEXUS-002, NEXUS-002.1, NEXUS-003, NEXUS-003.1, NEXUS-003.2.**

The project baseline has been established:

- Linux x86_64 development environment.
- Git repository initialized on branch `main`.
- Rust stable toolchain configured and pinned via `.mise.toml` (`rustc 1.100.0`, `cargo 1.100.0`).
- Runtime SQLite verified with FTS5 virtual table support (`sqlite_compileoption_used('ENABLE_FTS5') = 1`).
- Python 3.14.7 available.
- Minimal modular Axum HTTP service created with `GET /health` bound to `127.0.0.1:3000`.
- SQLite persistent storage foundation implemented via `sqlx` (v0.9.0) with WAL mode, foreign keys, busy timeout, and embedded migrations.
- Migration integrity verified against existing local databases; baseline migration checksum matches.
- Health endpoint dynamically verifies database connectivity and returns `{"status":"ok","service":"nexus-cme","database":"connected"}`.
- Quality gates, test classifications, multi-connection PRAGMA assertions, and documentation verified.

**Next milestone: NEXUS-004.**

Implement event ingestion and persistent event records.

## 12. Project Philosophy

Nexus CME is an engineering and research project.

Its purpose is not simply to create another AI agent.

It is to investigate how persistent memory, structured decision-making, efficient retrieval and generative reasoning can work together to build more capable and reliable AI systems.

Every major feature should be tested, evaluated and justified.

**Build incrementally. Measure honestly. Preserve what matters.**
