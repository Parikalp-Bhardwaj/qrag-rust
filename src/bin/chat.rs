use anyhow::{Context, Result};
use std::io::{self, BufRead, Write};
// use tonic::transport::Channel;

pub mod rag_proto {
    tonic::include_proto!("rag");
}

use rag_proto::{
    rag_service_client::RagServiceClient, AskQuestionRequest, ReindexRequest,
};

#[tokio::main]
async fn main() -> Result<()> {
    let addr = std::env::var("RAG_SERVER")
        .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());

    let mut client = RagServiceClient::connect(addr.clone())
        .await
        .with_context(|| format!("could not reach RAG server at {addr} — is it running?"))?;

    println!("Connected to {addr}");
    println!("Type a question and press Enter. Commands: /reindex, /quit");
    println!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input = stdin.lock();
    let mut line = String::new();

    loop {
        print!("you > ");
        stdout.flush().ok();

        line.clear();
        let n = input.read_line(&mut line)?;
        if n == 0 {
            println!();
            break;
        }

        let q = line.trim();
        if q.is_empty() {
            continue;
        }
        if q == "/quit" || q == "/exit" {
            break;
        }
        if q == "/reindex" {
            match client.reindex(ReindexRequest {}).await {
                Ok(resp) => {
                    let r = resp.into_inner();
                    println!("indexed {} chunks — {}\n", r.chunks_indexed, r.message);
                }
                Err(e) => eprintln!("reindex failed: {}\n", e.message()),
            }
            continue;
        }

        match client
            .ask_question(AskQuestionRequest {
                question: q.to_string(),
            })
            .await
        {
            Ok(resp) => {
                let r = resp.into_inner();
                println!("\nbot > {}\n", r.answer);
                if !r.sources.is_empty() {
                    println!("  sources:");
                    for s in r.sources {
                        let preview: String = s.preview.chars().take(80).collect();
                        println!(
                            "    - {}  (score {:.3})\n      {}…",
                            s.file_path, s.score, preview
                        );
                    }
                    println!();
                }
            }
            Err(e) => eprintln!("error: {}\n", e.message()),
        }
    }

    Ok(())
}