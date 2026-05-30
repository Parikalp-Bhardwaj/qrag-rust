use crate::chunker::DocumentChunk;
use anyhow::{Context, Result};

use qdrant_client::{
    qdrant::{
        CreateCollectionBuilder,
        Distance,
        PointStruct,
        QueryPointsBuilder,
        UpsertPointsBuilder,
        VectorParamsBuilder,
    },
    Payload,
    Qdrant,
};

use rig::{
    client::{EmbeddingsClient, ProviderClient},
    embeddings::EmbeddingsBuilder,
    providers::openrouter::Client as OpenRouterAiClient,
    vector_store::{VectorSearchRequest, VectorStoreIndex},
    Embed,
};


const EMBEDDING_MODEL: &str = "openai/text-embedding-3-small";

use rig_qdrant::QdrantVectorStore;
use serde::{Serialize, Deserialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedChunk{
    pub file_path: String,
    pub chunk_index: usize,
    pub text: String,
    pub score: f32
}

#[derive(Clone)]
pub struct QdrantStore{
    client: Qdrant,
    openrouter_client: OpenRouterAiClient,
    collection_name: String
}

#[derive(Embed, Clone)]
struct ChunkEmbedding {
    id: String,
    file_path: String,
    chunk_index: usize,
    #[embed]
    text: String,
}


impl QdrantStore{
    pub fn new(qdrant_url: &str, collection_name: &str) -> Result<Self, anyhow::Error>{
        dotenvy::dotenv().ok();

        let client = Qdrant::from_url(qdrant_url)
                .build()
                .context("Falied to create Qdrant client")?;

        let openrouter_client = OpenRouterAiClient::from_env()?;

        Ok(Self{
            client,
            openrouter_client,
            collection_name: collection_name.to_string()
        })
    }

    pub async fn ensure_collection(&self) -> Result<()>{
        let exits = self
                .client
                .collection_exists(&self.collection_name)
                .await.context("Failed to check Qdrant collection")?;

        if exits{
            return Ok(())
        }

        self
            .client
            .create_collection(CreateCollectionBuilder::new(&self.collection_name)
                .vectors_config(VectorParamsBuilder::new(1536, Distance::Cosine))
            )
            .await
            .context("Failed to create Qdrant collection")?;

        Ok(())
    }

    pub async fn reset_collection(&self) -> Result<()>{
        let exists = self
                    .client
                    .collection_exists(&self.collection_name)
                    .await
                    .context("Failed to check Qdrant collection")?;

        if exists{
            self
                .client
                .delete_collection(&self.collection_name)
                .await
                .context("Falied to delete existing Qdrant collection")?;
        }
        self.ensure_collection().await
    }

    pub async fn upsert_chunks(&self, 
        chunks: &[DocumentChunk]) -> Result<usize>{
        
        if chunks.is_empty(){
            return Ok(0)
        }

        let embedding_model = self.openrouter_client.embedding_model(EMBEDDING_MODEL);
        let mut builder = EmbeddingsBuilder::new(embedding_model);
        
        for chunk in chunks {
            builder = builder.document(ChunkEmbedding {
                id: chunk.id.clone(),
                file_path: chunk.file_path.clone(),
                chunk_index: chunk.chunk_index,
                text: chunk.text.clone(),
            })?;
        }
        

        let embedded_doc = builder
                            .build()
                            .await
                            .context("Failed to create embedding with Rig")?;


                            let points: Vec<PointStruct> = embedded_doc
                            .into_iter()
                            .map(|(doc, embeddings)| {
                                let vector: Vec<f32> = embeddings
                                    .first()        // OneOrMany<Embedding> -> Embedding
                                    .vec            // Vec<f64>
                                    .iter()
                                    .map(|&value| value as f32)
                                    .collect();
                        
                                PointStruct::new(
                                    doc.id.clone(),
                                    vector,
                                    Payload::try_from(json!({
                                        "file_path": doc.file_path,
                                        "chunk_index": doc.chunk_index,
                                        "text": doc.text,
                                    }))
                                    .expect("valid Qdrant payload"),
                                )
                            })
                            .collect();

        self
            .client
            .upsert_points(UpsertPointsBuilder::new(&self.collection_name, points))
            .await
            .context("Falied to upsert points into Qdrant")?;

        Ok(chunks.len())
    }

    pub async fn search(&self, question: &str, top_k: usize) -> Result<Vec<RetrievedChunk>>{

        let embedding_model = self
            .openrouter_client
            .embedding_model(EMBEDDING_MODEL);

        let query = QueryPointsBuilder::new(&self.collection_name)
            .with_payload(true)
            .limit(top_k as u64)
            .build();

        let vector_store = QdrantVectorStore::new(
            self.client.clone(),
            embedding_model,
            query,
        );

        let request = VectorSearchRequest::builder()
            .query(question)
            .samples(top_k as u64)
            .build();

        let results = vector_store
            .top_n::<serde_json::Value>(request)
            .await
            .context("Failed to search Qdrant through Rig")?;

        let mut chunk = Vec::new();

        for (score, _id, payload) in results {
            let file_path = payload
                .get("file_path")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
        
            let chunk_index = payload
                .get("chunk_index")
                .and_then(|v| v.as_u64())
                .unwrap_or_default() as usize;
        
            let text = payload
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
        
            chunk.push(RetrievedChunk {
                file_path,
                chunk_index,
                text,
                score: score as f32,
            });
        }
        

        

        Ok(chunk)
    }
}




