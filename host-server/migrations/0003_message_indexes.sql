-- get_messages_between_users queries:
--   WHERE (sender_id = $1 AND recipient_id = $2) OR (sender_id = $2 AND recipient_id = $1)
--   ORDER BY timestamp DESC LIMIT $3
-- With no supporting index this was a full sequential scan on every
-- conversation fetch. A single composite index covers both branches of the
-- OR: Postgres's planner treats equality predicates on composite indexes as
-- order-agnostic, so (recipient_id, sender_id) also satisfies lookups
-- phrased as (sender_id, recipient_id) - verified via EXPLAIN that a second,
-- differently-ordered index adds write overhead with zero query benefit.
CREATE INDEX IF NOT EXISTS idx_messages_recipient_sender_timestamp
    ON messages (recipient_id, sender_id, timestamp DESC);
