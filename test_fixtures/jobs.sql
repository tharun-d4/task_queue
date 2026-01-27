INSERT INTO jobs 
(id, job_type, payload, status, priority, max_retries, created_at)
VALUES
(
  '019bfadc-28bb-781d-9d22-acf23fe50117',
  'send_email',
  '{
    "from": "job_scheduler@gmail.com",
    "to": "user@gmail.com",
    "subject": "Yay! Welcome to Job scheduler",
    "body": "This is an automated email sent by a job scheduler and its workers."
  }',
  'pending',
  5,
  3,
  NOW() AT TIME ZONE 'UTC'
),
(
  '019bfdd5-cc70-7f37-a02a-1ec5849f25df',
  'send_email',
  '{
    "from": "job_scheduler@gmail.com",
    "to": "user@gmail.com",
    "subject": "Please complete your registration process",
    "body": "Hey user, we would like to complete your registration process immediately to win the reward."
  }',
  'pending',
  1,
  5,
  NOW() AT TIME ZONE 'UTC'
);
