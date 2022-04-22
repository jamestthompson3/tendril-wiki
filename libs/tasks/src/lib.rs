use messages::Message;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;
use ulid::Ulid;

pub mod messages;
pub mod password;
pub mod sync;

pub use self::password::*;
pub use self::sync::*;

/// Shout out to Sylvain Kerkour's [blog post](https://kerkour.com/rust-job-queue-with-postgresql) for code snippets and inspiration!

#[derive(Debug, Error)]
pub enum TaskError {}

#[async_trait::async_trait]
pub trait Queue: Send + Sync + Debug {
    async fn push(&self, job: Message) -> Result<(), crate::TaskError>;
    /// pull fetches at most `number_of_jobs` from the queue.
    async fn pull(&self, number_of_jobs: u32) -> Result<Vec<Job>, crate::TaskError>;
    async fn delete_job(&self, job_id: Ulid) -> Result<(), crate::TaskError>;
    async fn fail_job(&self, job_id: Ulid) -> Result<(), crate::TaskError>;
    async fn clear(&self) -> Result<(), crate::TaskError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Ulid,
    pub message: Message,
}
