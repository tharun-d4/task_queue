INSERT INTO workers 
(id, pid, started_at, last_heartbeat)
VALUES 
(
  '019bfe1d-228e-7938-8678-3798f454c236',
  54321,
  NOW() AT TIME ZONE 'UTC',
  NOW() AT TIME ZONE 'UTC'
),
(
  '019bfe1e-4450-76d4-ad0e-bf5d38280c16',
  12345,
  NOW() AT TIME ZONE 'UTC',
  NOW() AT TIME ZONE 'UTC'
);
