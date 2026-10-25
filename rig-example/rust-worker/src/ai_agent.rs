use rig::Agent;
use rig::prelude::*;
use rig::providers::openai::{OpenAI, OpenAIConfig};

#[derive(Clone)]
pub struct AIAgent {
    client: OpenAI,
    agent: Agent,
}

const AGENT_DEFINITION: &str =
    "You are an assistant that analyses documents \
     and produces concise, evidence-based conclusions.";

impl AIAgent {
    pub fn new(
        api_key: String,
        base_url: String,
        model: String,
    ) -> anyhow::Result<Self> {
        let client = OpenAIConfig::new(api_key)
            .with_base_url(base_url)
            .client();

        let agent = AgentBuilder::new(client.chat(model))
            .preamble(AGENT_DEFINITION)
            .build();

        Ok(Self { client, agent })
    }

    pub async fn prompt(
        &self,
        message: &str,
    ) -> anyhow::Result<String> {
        let response = self.agent.prompt(message).await?;
        Ok(response.output().to_string())
    }
}