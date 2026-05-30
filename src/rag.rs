use crate::{
    document_loader::loader_documents_from_dir,
    chunker::chunk_retrieve,
    llm::LlmService,
    qdrant_store::{QdrantStore, RetrievedChunk}
};

use anyhow::Result;

#[derive(Clone)]
pub struct RagEngine{
    file_dir: String,
    qdrant_store: QdrantStore,
    llm: LlmService
}

#[derive(Debug)]
pub struct RagAnswer{
    pub answer: String,
    pub sources: Vec<RetrievedChunk>
}


impl RagEngine{
    pub fn new(
        file_dir: impl Into<String>,
        qdrant_store: QdrantStore,
        llm: LlmService
    ) -> Self{

        Self { 
            file_dir: file_dir.into(), 
            qdrant_store, 
            llm 
        }
    }

    pub async fn initialize(&self) -> Result<()>{
        self.qdrant_store.ensure_collection().await
    }

    pub async fn reindex_docs(&self) -> Result<usize>{
        let load = loader_documents_from_dir(&self.file_dir)?;
        let chunks = chunk_retrieve(load, 120);
        self.qdrant_store.reset_collection().await?;
        let indexed = self.qdrant_store.upsert_chunks(&chunks).await?;
        Ok(indexed)
    }

    pub async fn ask_question(&self, question: String) -> Result<RagAnswer>{
        let chunk = self.qdrant_store.search(&question, 3).await?;
        
        let answer = self
                .llm
                .answer_question(&question, &chunk)
                .await?;

        Ok(RagAnswer { 
            answer, 
            sources: chunk 
        })
    }
}