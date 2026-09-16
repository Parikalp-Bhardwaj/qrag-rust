use crate::{
    document_loader::loader_documents_from_dir,
    chunker::chunk_retrieve,
    llm::LlmService,
    qdrant_store::{QdrantStore, RetrievedChunk}
};

use anyhow::Result;

#[derive(Clone)]
pub struct RagEngine{
    pub file_dir: String,
    pub qdrant_store: QdrantStore,
    pub llm: LlmService
}

#[derive(Debug)]
pub struct RagAnswer{
    pub answer: String,
    pub sources: Vec<RetrievedChunk>
}

#[derive(Debug)]
pub enum IndexStatus {
    AlreadyPopulated(u64),
    Indexed(usize),
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

    pub async fn ask_question(&self, question: String) -> Result<RagAnswer> {
        let chunk = self.qdrant_store.search(&question, 3).await?;

        let answer = self.llm
            .answer_question_streamed(&question, &chunk, |_| {})
            .await?;

        Ok(RagAnswer { answer, sources: chunk })
    }

    pub async fn retrieve(&self, question: &str) -> Result<Vec<RetrievedChunk>> {
        self.qdrant_store.search(question, 3).await
    }

    pub async fn answer_streamed(
        &self,
        question: &str,
        chunks: &[RetrievedChunk],
        on_token: impl FnMut(&str),
    ) -> Result<String> {
        self.llm.answer_question_streamed(question, chunks, on_token).await
    }

    pub async fn ensure_indexed(&self) -> Result<IndexStatus> {
        let existing = self.qdrant_store.count_points().await?;
        if existing > 0 {
            return Ok(IndexStatus::AlreadyPopulated(existing));
        }
        let indexed = self.reindex_docs().await?;
        Ok(IndexStatus::Indexed(indexed))
    }
}