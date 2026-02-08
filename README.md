# Job Scheduler
A job scheduler system built in Rust for reliable background job processing.

## What It Does
```
Client submits job via HTTP
        ↓
      Server
        ↓
    PostgreSQL (queue)
        ↓
  Workers claim jobs
        ↓
Execute & retry on failure
```

## Technologies
- **Server:** Rust (tokio, axum)
- **Worker:** Rust (tokio)
- **Database:** PostgreSQL (sqlx)
