
use anyhow::Result;
use tonic::transport::Server;
use tracing::info;


use anyhow;
mod document_loader;
mod config;
mod chunker;
mod qdrant_store;
mod llm;
mod rag;
mod grpc_service;


pub mod rag_proto {
    tonic::include_proto!("rag");
}

use crate::{
    config::Config,
    grpc_service::RagGrpcService,
    llm::LlmService,
    qdrant_store::QdrantStore,
    rag::RagEngine,
    rag_proto::rag_service_server::RagServiceServer,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error>{

    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let config = Config::from_env();
    let addr = config.server_addr();

    let qdrant_store = QdrantStore::new(
        &config.qdrant_url,
        &config.qdrant_collection,
    )?;

    let llm = LlmService::new()?;

    let engine = RagEngine::new(
        "./docs",
        qdrant_store,
        llm,
    );

    engine.initialize().await?;

    let grpc_service = RagGrpcService::new(engine);

    info!("CodeRAG-rs gRPC server running on {}", addr);


    Server::builder()
        .add_service(RagServiceServer::new(grpc_service))
        .serve(addr.parse()?)
        .await?;


    Ok(())
}
