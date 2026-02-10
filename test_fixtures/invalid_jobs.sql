INSERT INTO jobs 
(
  id,
  job_type,
  payload,
  status,
  priority,
  max_retries,
  created_at
)
VALUES
(
  '019c481e-25ce-79be-9f03-1d081e6a52bd',
  'invalid_job_type',
  '{
    "event": "This is a sample of invalid job"
  }',
  'pending',
  4,
  2,
  NOW() AT TIME ZONE 'UTC'
);
