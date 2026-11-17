# AGENTS.md

# Nexus CME - Agent Instructions

## 1. Project Identity

Nexus CME (Context Management Engine) is a local-first memory and context management infrastructure designed to improve the reliability, efficiency and continuity of AI agents.

Its purpose is to capture information from agent interactions, organize persistent memories, retrieve relevant historical information and assemble optimized context for LLM inference.

The long-term objective is to create a modular, reusable context management engine that can work with different AI agents and model providers.

## 2. Core Engineering Philosophy

Follow these principles throughout development:

1. Correctness before optimization.
2. Reliability before autonomy.
3. Measurable improvements before adding complexity.
4. Modular architecture without unnecessary microservices.
5. Preserve original information and maintain source provenance.
6. Local-first operation wherever practical.
7. Keep external model dependencies replaceable.
8. Prefer explicit, testable policies over hidden behavior.
9. Never discard important information based solely on an uncertain model judgment.
10. Document significant architectural decisions.

Do not introduce a feature simply because it sounds sophisticated. Every substantial feature must solve a defined problem.

## 3. Technology Stack

### Core service
- Rust
- Tokio
- Axum
- Serde
- tracing
- tracing-subscriber

### Persistent storage
- SQLite
- SQLite WAL mode
- SQLite FTS5

### Database access
- Select a compatible Rust SQLite library after evaluating SQLx and rusqlite against project requirements.
- Document the decision before adding the database implementation.

### Agent integration
- Python
- LangGraph / Deep Agents

### Decision intelligence
- TypeSafe AI Jev API

### Local reasoning
- llama.cpp
- Local Qwen model

### Future capabilities
- Embedding-based semantic retrieval
- Memory consolidation
- Temporal memory
- Context budgeting
- Background processing
- Evaluation and benchmarking

Do not implement future capabilities before their corresponding milestone is approved.

## 4. Architecture Principles

The initial architecture is a single modular Rust service.

The service should eventually contain these logical components:

- API layer
- Event ingestion
- Database layer
- Memory processing
- Retrieval engine
- Decision engine
- Context assembly
- Background workers
- LLM integration

Python should primarily handle agent orchestration and communicate with the Rust service through documented APIs.

Jev should provide structured probabilistic decisions.

Rust should enforce application policies and deterministic operations.

SQLite should remain the authoritative persistent record.

Qwen should handle tasks requiring generative reasoning, summarization and complex interpretation.

Do not make the Rust application dependent on Jev or Qwen to perform basic storage, retrieval or health checks.

## 5. Development Workflow

Work in small, independently testable milestones.

Before implementing a substantial task:

1. Inspect the existing code.
2. Understand the current architecture.
3. Identify the exact files that need changes.
4. Explain the proposed implementation.
5. Identify dependencies and compatibility concerns.
6. Implement only the approved scope.

After implementation:

1. Run formatting.
2. Run compilation checks.
3. Run relevant tests.
4. Run Clippy where applicable.
5. Inspect the Git diff.
6. Report files changed.
7. Explain any deviations from the approved design.

Never claim a test passed unless it was actually executed.

Never hide compiler warnings, failed tests or unresolved errors.

## 6. Coding Standards

### Rust

- Prefer clear, idiomatic Rust.
- Use meaningful names.
- Keep functions focused.
- Avoid unnecessary cloning.
- Handle errors explicitly.
- Avoid unnecessary unwrap() and expect() in production code.
- Use Result and appropriate error types.
- Use asynchronous operations appropriately.
- Avoid blocking operations inside async request handlers.
- Keep modules organized by responsibility.
- Avoid excessive abstraction.
- Add comments when they explain non-obvious decisions, not obvious syntax.

### Python

- Use type hints.
- Prefer asynchronous HTTP communication where appropriate.
- Keep agent orchestration separate from the Rust memory service.
- Do not duplicate database business logic in Python.

### SQL

- Use explicit schema definitions.
- Use parameterized queries.
- Add indexes based on actual query patterns.
- Use transactions for related writes.
- Preserve referential integrity.
- Version schema changes through migrations.

## 7. Database and Memory Rules

The database is the authoritative source of stored events and memories.

Follow these rules:

- Preserve original events.
- Store derived memories separately.
- Maintain references from memories to their source events.
- Track timestamps.
- Track memory status and expiry where applicable.
- Preserve superseded memories for historical queries.
- Do not silently overwrite historical facts.
- Make important writes durable before acknowledging successful ingestion.
- Use controlled write access and transactions.

Generated summaries are derived information, not replacements for original events.

## 8. AI and Decision Engine Rules

Jev and LLMs are probabilistic systems.

Never treat their outputs as guaranteed truth.

For decisions:

- Validate API responses.
- Validate expected output types.
- Apply confidence thresholds through explicit application policies.
- Preserve uncertain decisions for further evaluation.
- Implement timeouts and failure handling.
- Support graceful operation when external services are unavailable.
- Never allow an external model to directly delete important memories.

Keep decision logic behind replaceable interfaces so different decision providers can be evaluated.

Do not send API keys, passwords, authentication tokens or unnecessary sensitive information to external model providers.

## 9. Security

- Bind the initial service to 127.0.0.1.
- Never expose credentials in source code.
- Keep secrets in environment variables or an appropriate secret store.
- Never commit .env files.
- Maintain .env.example without real credentials.
- Validate all incoming API requests.
- Use parameterized SQL queries.
- Do not execute untrusted model-generated shell commands automatically.
- Treat external model responses and tool outputs as untrusted input.

## 10. Dependencies

- Add only necessary dependencies.
- Prefer maintained and well-supported libraries.
- Check compatibility before introducing new crates.
- Explain why a dependency is needed.
- Avoid duplicate libraries serving the same purpose.
- Do not install unrelated packages.
- Do not upgrade dependencies unnecessarily.

## 11. Git Rules

- Use main as the stable branch.
- Use feature branches for substantial development tasks where appropriate.
- Inspect git status before modifying existing work.
- Never overwrite uncommitted changes without approval.
- Review git diff before completing a task.
- Do not commit secrets, databases or generated build artifacts.
- Do not create commits unless explicitly authorized.

## 12. Testing Requirements

Every significant module should have tests.

Prioritize:

- Unit tests for business logic.
- Integration tests for API behavior.
- Database tests for persistence and migrations.
- Retrieval tests using representative historical data.
- Failure tests for unavailable external services.
- Regression tests for previously discovered bugs.

Tests must verify behavior, not simply confirm that code executes.

## 13. Documentation

Keep these documents updated:

- README.md: project overview and usage.
- AGENTS.md: engineering instructions.
- docs/architecture.md: architecture and component boundaries.
- docs/decision-log.md: architectural decisions and their rationale.

Update documentation whenever a meaningful architectural change occurs.

## 14. Working With the Project Owner

The project owner is learning software engineering and AI infrastructure while developing Nexus CME.

For substantial changes:

- Explain what is being implemented.
- Explain why the design was selected.
- Explain important technical concepts.
- Identify trade-offs.
- Explain commands and their effects.
- Provide test results.
- Identify unresolved decisions.

Do not blindly execute broad implementation instructions if they conflict with existing architecture or project requirements.

If a major architectural change appears necessary, explain the reason and request approval before proceeding.

## 15. Scope Discipline

Implement only the requested milestone.

Do not prematurely introduce:

- Microservices
- Kubernetes
- Docker infrastructure
- Qdrant or other vector databases
- Complex autonomous memory agents
- Unnecessary message brokers
- Distributed processing systems
- Additional model providers

These may be evaluated later when measurable requirements justify them.

## 16. Definition of Done

A task is complete only when:

1. The requested functionality is implemented.
2. The code compiles.
3. Relevant tests pass.
4. Formatting and lint checks pass where applicable.
5. The implementation respects the architecture.
6. Documentation is updated where necessary.
7. The Git diff has been inspected.
8. Remaining limitations are reported honestly.

The objective is to build a reliable, understandable and maintainable engineering system, not merely generate code quickly.