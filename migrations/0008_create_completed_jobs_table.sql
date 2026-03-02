CREATE TABLE completed_jobs (
  id UUID PRIMARY KEY,
  job_type VARCHAR(100) NOT NULL,
  payload JSONB NOT NULL,
  priority SMALLINT NOT NULL,
  max_retries SMALLINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  run_at TIMESTAMPTZ NOT NULL,
  started_at TIMESTAMPTZ NOT NULL,
  completed_at TIMESTAMPTZ NOT NULL,
  worker_id UUID NOT NULL,
  lease_expires_at TIMESTAMPTZ NOT NULL,
  attempts SMALLINT NOT NULL,
  error_message TEXT,
  result JSONB
);
