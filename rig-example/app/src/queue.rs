use std::path::PathBuf;
use std::sync::Arc;
use dashmap::DashMap;
use serde::Serialize;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", content = "detail")]
pub enum JobStatus {
    Queued,
    Processing,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct QueueJob {
    pub id: Uuid,
    pub file_path: PathBuf,
}

pub struct Queue {
    tx: mpsc::Sender<QueueJob>,
    pub(crate) jobs: Arc<DashMap<Uuid, JobStatus>>,
}

impl Queue {
    pub fn new(tx: mpsc::Sender<QueueJob>) -> Self {
        Self {
            tx,
            jobs: Arc::new(DashMap::new()),
        }
    }

    pub async fn enqueue(
        &self,
        file_path: PathBuf,
        id: Uuid,
    ) -> anyhow::Result<Uuid, mpsc::error::TrySendError<QueueJob>> {
        //        let id = Uuid::new_v4();

        let job = QueueJob { id, file_path };

        self.jobs.insert(id, JobStatus::Queued);
        if let Err(err) = self.tx.try_send(job) {
            self.jobs.remove(&id);
            return Err(err);
        }

        Ok(id)
    }

    pub fn status(&self, id: Uuid) -> Option<JobStatus> {
        self.jobs.get(&id).map(|entry| entry.clone())
    }

    pub fn list(&self) -> Vec<(Uuid, JobStatus)> {
        self.jobs.iter().map(|entry| (*entry.key(), entry.value().clone())).collect()
    }
}


