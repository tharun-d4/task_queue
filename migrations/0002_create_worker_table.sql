CREATE TABLE workers (
  id UUID PRIMARY KEY,
  pid INT NOT NULL,
  started_at TIMESTAMPTZ NOT NULL,
  last_heartbeat TIMESTAMPTZ NOT NULL,
  shutdown_at TIMESTAMPTZ
);
