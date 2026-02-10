CREATE TYPE job_status AS ENUM ('pending', 'running', 'completed');

CREATE TABLE jobs (
  id UUID PRIMARY KEY,
  job_type VARCHAR(100) NOT NULL,
  payload JSONB NOT NULL,
  status job_status NOT NULL,
  priority SMALLINT NOT NULL,
  max_retries SMALLINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  started_at TIMESTAMPTZ,
  completed_at TIMESTAMPTZ,
  worker_id UUID,
  attempts SMALLINT,
  error_message TEXT,
  result JSONB
);
