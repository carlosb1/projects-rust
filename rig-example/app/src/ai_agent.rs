use rig::Agent;
use rig::client::{AgentClientExt, CompletionClient};
use rig::completion::Prompt;
use rig::core::client;
use rig::http_client::ReqwestClient;
use rig::providers::openai;
use rig::providers::openai::OpenAICompletionsExt;

#[derive(Clone)]
pub struct AIAgent  {
    client: client::Client<OpenAICompletionsExt, ReqwestClient>,
    agent: Agent,
}

const AGENT_DEFINITION: &str = "You are an assistant that analyses documents and produces concise, evidence-based conclusions.";

impl AIAgent {
    pub fn new(api_key: String, base_url: String, model: String) -> anyhow::Result<Self> {
        let client = openai::Client::builder()
            .api_key(api_key)
            .base_url(base_url)
            .build()?
            .completions_api();

        let agent = client.agent(&model)
            .preamble(
                AGENT_DEFINITION
            )
            .build();


        Ok(AIAgent {client, agent})
    }
    pub async fn prompt(&self, prompt: &str) -> anyhow::Result<String> {
        let response = self.agent
            .prompt(prompt)
            .await?;
        Ok(response)
    }
}