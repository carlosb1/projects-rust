
use aws_sdk_sqs::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteJob {
    pub job_id: Uuid,
    pub task: String,
    pub document_key: String,
}

#[derive(Clone)]
pub struct JobPublisher {
    client: Client,
    python_queue_url: String,
    rust_queue_url: String,
}

impl JobPublisher {
    pub async fn from_env() -> anyhow::Result<Self> {
        let config = aws_config::load_defaults(
            aws_config::BehaviorVersion::latest(),
        ).await;

        let mut builder =
            aws_sdk_sqs::config::Builder::from(&config);

        if let Ok(endpoint) = std::env::var("SQS_ENDPOINT") {
            builder = builder.endpoint_url(endpoint);
        }

        let client = Client::from_conf(builder.build());

        Ok(Self {
            client,
            python_queue_url: std::env::var("PYTHON_QUEUE_URL")?,
            rust_queue_url: std::env::var("RUST_QUEUE_URL")?,
        })
    }

    pub async fn publish_python(
        &self,
        job: &RemoteJob,
    ) -> anyhow::Result<()> {
        self.publish(&self.python_queue_url, job).await
    }

    pub async fn publish_rust(
        &self,
        job: &RemoteJob,
    ) -> anyhow::Result<()> {
        self.publish(&self.rust_queue_url, job).await
    }

    async fn publish(
        &self,
        queue_url: &str,
        job: &RemoteJob,
    ) -> anyhow::Result<()> {
        let body = serde_json::to_string(job)?;

        self.client
            .send_message()
            .queue_url(queue_url)
            .message_body(body)
            .send()
            .await?;

        Ok(())
    }
}
