use messages::Message;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::Mutex;
use thiserror::Error;
use ulid::Ulid;

pub mod messages;
pub mod password;
pub mod sync;
pub mod archive;

pub use self::password::*;
pub use self::sync::*;

/// Shout out to Sylvain Kerkour's [blog post](https://kerkour.com/rust-job-queue-with-postgresql) for code snippets and inspiration!

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("could not acquire the mutex lock")]
    MutexAcquireError,
}

#[async_trait::async_trait]
pub trait Queue: Send + Sync + Debug {
    async fn push(&self, job: Message) -> Result<(), crate::TaskError>;
    async fn pull(&self, number_of_jobs: u32) -> Result<Vec<Job>, crate::TaskError>;
    async fn delete_job(&self, job_id: Ulid) -> Result<(), crate::TaskError>;
    async fn clear(&self) -> Result<(), crate::TaskError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Ulid,
    pub message: Message,
}

#[allow(clippy::from_over_into)]
impl Into<Job> for Message {
    fn into(self) -> Job {
        Job {
            id: Ulid::new(),
            message: self,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct JobQueue {
    jobs: Arc<Mutex<Vec<Job>>>,
}

#[async_trait::async_trait]
impl Queue for JobQueue {
    async fn push(&self, job: Message) -> Result<(), crate::TaskError> {
        let enqueued_job = job.into();
        match self.jobs.lock() {
            Ok(mut jobs) => {
                jobs.push(enqueued_job);
                Ok(())
            }
            Err(e) => {
                eprintln!("{}", e);
                Err(TaskError::MutexAcquireError)
            }
        }
    }
    async fn pull(&self, number_of_jobs: u32) -> Result<Vec<Job>, crate::TaskError> {
        match self.jobs.lock() {
            Ok(mut jobs) => {
                if number_of_jobs as usize > jobs.len() {
                    let job_queue = jobs.drain(..).collect::<Vec<Job>>();
                    Ok(job_queue)
                } else {
                    let job_queue = jobs.drain(..number_of_jobs as usize).collect();
                    Ok(job_queue)
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                Err(TaskError::MutexAcquireError)
            }
        }
    }
    async fn delete_job(&self, job_id: Ulid) -> Result<(), crate::TaskError> {
        match self.jobs.lock() {
            Ok(mut jobs) => {
                *jobs = jobs
                    .iter()
                    .filter(|j| j.id != job_id)
                    .map(|j| j.to_owned())
                    .collect::<Vec<Job>>();
                Ok(())
            }
            Err(e) => {
                eprintln!("{}", e);
                Err(TaskError::MutexAcquireError)
            }
        }
    }
    async fn clear(&self) -> Result<(), crate::TaskError> {
        match self.jobs.lock() {
            Ok(mut jobs) => {
                jobs.clear();
                Ok(())
            }
            Err(e) => {
                eprintln!("{}", e);
                Err(TaskError::MutexAcquireError)
            }
        }
    }
}
