use lettre::{
    address::AddressError, error::Error as LettreError, transport::smtp::Error as SmtpError,
};
use reqwest::Error as ReqwestError;
use sqlx::Error as SqlxError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkerError {
    #[error("Database error")]
    Database(SqlxError),
    #[error("Email error: {0}")]
    Email(String),
    #[error("Invalid Job error")]
    InvalidJob,
    #[error("Webhook error: {0}")]
    Webhook(String),
    #[error("Reqwest error")]
    Request(ReqwestError),
}

impl From<SqlxError> for WorkerError {
    fn from(err: SqlxError) -> Self {
        WorkerError::Database(err)
    }
}

impl From<LettreError> for WorkerError {
    fn from(err: LettreError) -> Self {
        WorkerError::Email(err.to_string())
    }
}

impl From<AddressError> for WorkerError {
    fn from(err: AddressError) -> Self {
        WorkerError::Email(err.to_string())
    }
}

impl From<SmtpError> for WorkerError {
    fn from(err: SmtpError) -> Self {
        WorkerError::Email(err.to_string())
    }
}

impl From<ReqwestError> for WorkerError {
    fn from(err: ReqwestError) -> Self {
        WorkerError::Request(err)
    }
}
