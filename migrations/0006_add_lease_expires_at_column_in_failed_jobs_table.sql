ALTER TABLE failed_jobs
ADD COLUMN lease_expires_at TIMESTAMPTZ NOT NULL;
