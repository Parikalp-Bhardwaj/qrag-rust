use crate::qdrant_store::RetrievedChunk;

use anyhow::{Context, Result};
use rig::{
    client::{CompletionClient, ProviderClient},
    completion::Prompt,
    providers::openrouter
};

#[derive(Clone)]
pub struct LlmService{
    client: openrouter::Client
}

impl LlmService{
    pub fn new() -> Result<Self>{
        let client = openrouter::Client::from_env()
                    .context("Failed to create Rig OpenRouter client from OPENROUTER_API_KEY")?;
        Ok(Self { client })

    }

    pub async fn answer_question(
        &self,
        question: &str,
        chunks: &[RetrievedChunk],
        ) -> Result<String>{
        
        if chunks.is_empty(){
            return Ok(format!("could not find relevant content for: {}", question));
        }

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
                    .agent("openai/gpt-4o-mini")
                    .preamble("You answer questions using retrieved chunks as grounded context.")
                    .build();

        let respone = agent
                                .prompt(prompt)
                                .await
                                .context("Failed to generate answer with Rig agent")?;

        Ok(respone)
    }
}



fn build_context(chunks: &[RetrievedChunk]) -> String{
    let mut context = String::new();

    for chunk in chunks{
        context.push_str(&format!(
            "\nSource: {}\nChunk: {}\nScore: {:.4}\nText: {}\n",
            chunk.file_path,
            chunk.chunk_index,
            chunk.score,
            chunk.text
        ));
    }

    context
}