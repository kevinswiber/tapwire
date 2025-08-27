# Repository Guidelines

## Project Structure & Module Organization

- `shadowcat/`: Rust crate and CLI for the MCP proxy.
- `shadowcat/src`: Core modules (e.g., `transport/`, `proxy/`, `auth/`, `cli/`, `recorder/`, `retry/`).
- `shadowcat/tests/` and `shadowcat/standalone_pkce_test.rs`: Integration and standalone tests.
- `plans/`, `specs/`, `reviews/`, `archive/`: Roadmaps, specs, and reviews that guide development.

## Build, Test, and Development Commands

- Build: `cd shadowcat && cargo build` — compiles library and binary.
- Run: `cargo run -- --help` or e.g. `cargo run -- forward stdio -- npx @modelcontextprotocol/server-everything`.
- Test: `cargo test` — runs unit, integration, and doctests.
- Lint: `cargo xtask lint --workspace --all-targets` — lints and fails on warnings.
- Format: `cargo fmt --all` — apply Rustfmt formatting.

## Coding Style & Naming Conventions

- Rust 2021, 4‑space indentation, no tabs; keep imports ordered and minimal.
- Naming: modules/functions `snake_case`, types/traits `PascalCase`, constants `SCREAMING_SNAKE_CASE`.
- Error handling: use typed errors (`thiserror`) in library code; `anyhow` is acceptable in binaries/tests.
- Keep public APIs documented; add examples when feasible (doctests run in CI).

## Testing Guidelines

- Unit tests live next to code under `#[cfg(test)] mod tests { ... }`.
- Integration tests go in `shadowcat/tests/*.rs`; prefer black‑box, behavior‑driven cases.
- Use `#[tokio::test]` for async; keep tests deterministic (control timers, avoid real network when possible).
- Add tests with new features/bug fixes; cover edge cases (e.g., SSE reconnect, rate‑limit headers).

## Commit & Pull Request Guidelines

- Follow Conventional Commits: `feat:`, `fix:`, `docs:`, `refactor:`, `chore:` with optional scopes, e.g. `retry(http): ...`, `transport(sse): ...`.
- Commit subject: imperative mood, ~50 chars; include a concise body for rationale.
- PRs: clear description, linked issues, reproduction steps, and impact. Include config or CLI examples if behavior changes.
- Pre‑PR checks: `cargo fmt --all`, `cargo xtask lint --workspace --all-targets`, `cargo test` all green.

## Security & Configuration Tips

- Don’t commit secrets, tokens, or sensitive tapes; prefer redaction and `.env`/local config.
- Use YAML config (`shadowcat.yaml`) and environment overrides; avoid hard‑coding.
- Large recordings (`shadowcat/tapes/`) should not be versioned unless explicitly required.
