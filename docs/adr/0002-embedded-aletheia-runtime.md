# ADR 0002: Embedded Aletheia Runtime as CLI Default

Date: 2026-02-27  
Status: Accepted

## Context

`logos` is a local personal-finance CLI/TUI. The primary workflow is on one machine with fast local writes and historical reads. Running a local HTTP server for every CLI command adds process overhead that is unnecessary for the default use case.

## Decision

1. `logos-cli` uses embedded `AletheiaDB` through `logos-store-aletheia` as the default runtime mode.
2. Durable data is stored locally at:
   - `LOGOS_DB_PATH` when set, otherwise
   - `%USERPROFILE%\.logos\ledger` on Windows, or
   - `~/.logos/ledger` on Unix-like systems.
3. Journal writes are append-only with correction links; historical reads are bi-temporal (`valid_time` + `tx_time`).
4. HTTP `aletheia-server` remains supported as an optional integration/testing mode, not a requirement for local CLI operation.

## Consequences

Positive:

- Local CLI commands read/write directly without network/server orchestration.
- Persistence survives process restarts by default.
- Bi-temporal history is available in the same storage adapter used by the CLI.

Tradeoffs:

- Embedded storage lifecycle (path management, backup/restore discipline) is now an operator concern.
- Multi-process coordination and remote access require opting into server mode explicitly.

## Operational Notes

- Back up the full ledger directory when the CLI is not writing.
- Restore by replacing the directory and re-running commands against the same `LOGOS_DB_PATH`.
- Use `ledger aletheia start` and `ledger aletheia status` only when HTTP integration is needed.
