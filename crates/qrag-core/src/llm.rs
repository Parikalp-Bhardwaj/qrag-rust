use crate::qdrant_store::RetrievedChunk;

use crate::Config;
use anyhow::{Context, Result};
use futures::StreamExt;
use rig::{
    agent::MultiTurnStreamItem,
    client::{CompletionClient, ProviderClient},
    providers::openrouter,
    streaming::{StreamedAssistantContent, StreamingPrompt},
};

#[derive(Clone)]
pub struct LlmService {
    client: openrouter::Client,
}

impl LlmService {
    pub fn new() -> Result<Self> {
        let client = openrouter::Client::from_env()
            .context("Failed to create Rig OpenRouter client from OPENROUTER_API_KEY")?;
        Ok(Self { client })
    }

    pub async fn answer_question_streamed(
        &self,
        question: &str,
        chunks: &[RetrievedChunk],
        mut on_token: impl FnMut(&str),
    ) -> Result<String> {
        dotenvy::dotenv().ok();
        if chunks.is_empty() {
            let msg = format!("could not find relevant content for: {}", question);
            on_token(&msg);
            return Ok(msg);
        }

        let config = Config::from_env();
        let model = &config.model;
        let context = build_context(chunks);

        let prompt = format!(
            r#"
                You are a helpful Rust AI assistant.

                Answer the question using only the provided document context.

                Rules:
                - Be clear and concise.
                - If the context is not enough, say so.
                - Mention the source file when useful.
                - Do not invent facts outside the context.

                Context:
                {}

                Question:
                {}

                Answer:
                "#,
            context, question
        );

        let agent = self
            .client
            // .agent("openai/gpt-4o-mini")
            .agent(model)
            .preamble("You answer questions using retrieved chunks as grounded context.")
            .build();

        let mut stream = agent.stream_prompt(prompt).await;
        let mut full = String::new();
        while let Some(item) = stream.next().await {
            let item = item.map_err(|e| anyhow::anyhow!("streaming failed: {e}"))?;
            if let MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(t)) =
                item
            {
                full.push_str(&t.text);
                on_token(&t.text);
            }
        }
        Ok(full)
    }
}

fn build_context(chunks: &[RetrievedChunk]) -> String {
    let mut context = String::new();

    for chunk in chunks {
        context.push_str(&format!(
            "\nSource: {}\nChunk: {}\nScore: {:.4}\nText: {}\n",
            chunk.file_path, chunk.chunk_index, chunk.score, chunk.text
        ));
    }

    context
}
