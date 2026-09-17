use anyhow::Result;
use qrag_core::{Config, IndexStatus, LlmService, QdrantStore, RagEngine};

mod app;
mod ui;

use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::from_env();
    let qdrant_core = QdrantStore::new(&config.qdrant_url, &config.qdrant_collection)?;
    let llm = LlmService::new()?;
    let engine = RagEngine::new("./docs", qdrant_core, llm);

    engine.initialize().await?;

    println!("Preparing index…");
    match engine.ensure_indexed().await {
        Ok(IndexStatus::AlreadyPopulated(n)) => println!("Index ready ({n} chunks)."),
        Ok(IndexStatus::Indexed(n)) => println!("Auto-indexed {n} chunks from ./docs."),
        Err(e) => eprintln!("Auto-index failed ({e:#}); starting anyway — fix and restart."),
    }

    let mut terminal = ratatui::init();
    let result = App::new(engine).run(&mut terminal).await;
    ratatui::restore();

    result
}
