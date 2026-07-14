# Contributing to Q-Safe

## Before you start

Read [docs/HSM_VERIFICATION_STATUS.md](docs/HSM_VERIFICATION_STATUS.md) first. It states plainly what's proven, what's designed-but-unwired, and what doesn't exist yet (the RP2040 firmware). Any PR touching the HSM/crypto path should update that document if it changes the picture.

## Local setup

```bash
docker-compose up -d postgres         # or point DATABASE_URL at any Postgres 16 instance
cp .env.example .env                  # fill in real values; JWT_SECRET must be 32+ chars
cargo run -p qsafe-backend
cargo test -p qsafe-backend -p qsafe-common
```

## Before opening a PR

```bash
cargo fmt --all
cargo clippy -p qsafe-backend -p qsafe-common --all-targets -- -D warnings
cargo clippy -p qsafe-firmware --target thumbv6m-none-eabi -- -D warnings
cargo test -p qsafe-backend -p qsafe-common
```

CI runs all of the above plus a firmware compile check; it will reject a PR that fails any of them.

## Adding a database migration

Add a new `host-server/migrations/NNNN_description.sql` file. `sqlx::migrate!()` embeds these at compile time - **a new migration file alone does not reliably trigger a rebuild of a cached `target/` directory** (verified during this project's engineering audit: a migration was silently not picked up until something forced a real recompile). CI's cache key includes a hash of the migrations directory specifically to avoid this; locally, if a migration doesn't seem to be applying, `touch host-server/src/database.rs` and rebuild.

## Conventions

- **ADRs** ([docs/adr/](docs/adr/)) record significant design decisions in the order they were made, including ones later superseded - they're an append-only log, not living documentation. Add a new ADR rather than editing an old one.
- **[CHANGELOG.md](CHANGELOG.md)** follows [Keep a Changelog](https://keepachangelog.com/) conventions under an `[Unreleased]` heading.
- **[DEVLOG.md](DEVLOG.md)** is an engineering journal (goal, work done, problems hit, lessons learned) - useful context for understanding *why* something is shaped the way it is, not a place to look for current API behavior.
- Prefer adding a real integration test (`host-server/tests/`) over a unit test when the change touches an HTTP handler, the auth flow, or the HSM path - this codebase had a real, previously-shipped bug (a foreign key that made an entire endpoint always fail) that only zero test coverage on that endpoint allowed to go unnoticed.

## Reporting a security issue

See [SECURITY.md](SECURITY.md) - please don't open a public issue for a vulnerability.
