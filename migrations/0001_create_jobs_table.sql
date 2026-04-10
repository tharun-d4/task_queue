CREATE TYPE job_status AS ENUM ('pending', 'running', 'completed', 'failed', 'cancelled');

CREATE TABLE jobs (
  id UUID PRIMARY KEY,
  job_type VARCHAR(100) NOT NULL,
  payload JSONB NOT NULL,
  status job_status NOT NULL,
  priority SMALLINT NOT NULL,
  max_retries SMALLINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  run_at TIMESTAMPTZ NOT NULL,
  worker_id UUID,
  lease_expires_at TIMESTAMPTZ,
  started_at TIMESTAMPTZ,
  finished_at TIMESTAMPTZ,
  attempts SMALLINT NOT NULL DEFAULT 0,
  error_message TEXT,
  result JSONB
);
