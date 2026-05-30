# ADR 0005: Type-Safe Database Migrations

## Status
Accepted

## Context
In the legacy codebase, database tables were initialized by running a raw multi-query SQL string (`CREATE TABLE IF NOT EXISTS ...`) on every server startup. This approach has several severe drawbacks:
1. **Clustering & Horizontal Scaling Inhibited**: If multiple backend server instances start concurrently (e.g. in a Kubernetes cluster or serverless environment), they will all attempt to run the `CREATE TABLE` queries simultaneously. This can cause lock contention, race conditions, or application launch errors.
2. **Schema Version Tracking & Rollbacks**: Raw inline SQL strings do not support schema version tracking, schema version checks, or rollbacks. 
3. **No Drift Control**: Schema drift is hard to control and audit when table creation is defined directly inside application code.

## Decision
Transition the database schema definition to SQLx's built-in migration management tool.
1. Define the initial tables in a tracked SQL file: [0001_init.sql](../../host-server/migrations/0001_init.sql).
2. Invoke `sqlx::migrate!().run(&pool).await?` inside `Database::new` during connection pool configuration.
3. Remove the inline `create_tables` logic and its explicit call in [main.rs](../../host-server/src/main.rs).

## Consequences

### Positive
- **Automatic Migration Execution**: Database schema migrations are executed automatically on startup, but SQLx guarantees execution synchronization and logs migration history in a `_sqlx_migrations` tracking table.
- **Production-Ready Scaling**: Safe for multi-instance deployments since migrations are executed transactionally and sequentially.
- **Auditability**: Database schema changes are tracked inside the `migrations` directory in Git.

### Negative
- Local/CI testing requires a database configuration (offline queries or schema checks) or we must ensure migrations are run before executing compile-time checks if using SQLx offline mode. (Note: Currently the project uses dynamic queries via standard `sqlx::query` but transitioning to query macros in a future phase will require SQLx offline data compilation).
