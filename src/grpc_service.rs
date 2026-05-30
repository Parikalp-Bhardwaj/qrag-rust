use crate::{
    rag::RagEngine,
    rag_proto::{AskQuestionRequest, AskQuestionResponse, ReindexRequest, ReindexResponse, Source, 
        rag_service_server::RagService
    }
};

use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct RagGrpcService{
    engine: RagEngine
}

impl RagGrpcService{
    pub fn new(engine: RagEngine) -> Self{
        Self { engine }
    }
}

#[tonic::async_trait]
impl RagService for RagGrpcService{
    async fn reindex(
            &self,
            _request: Request<ReindexRequest>,
        ) -> Result<Response<ReindexResponse>, Status>{

            let engine = self.engine.clone();

            let handle = tokio::spawn(async move{
                engine
                    .reindex_docs()
                    .await
                    .map_err(|err|Status::internal(err.to_string()))
            });

            let chunks_indexed = handle
                    .await
                    .map_err(|err|Status::internal(err.to_string()))??;

            Ok(Response::new(ReindexResponse{
                chunks_indexed: chunks_indexed as u64,
                message: format!("Indexed {} chunk into qdrant", chunks_indexed)
            }))

    }

    async fn ask_question(
            &self,
            request: Request<AskQuestionRequest>
        ) -> Result<Response<AskQuestionResponse>, Status>{
        
        let question = request.into_inner().question;

        if question.trim().is_empty(){
            return Err(Status::invalid_argument("question cannot be empty"))
        }

        let rag_answer = self
            .engine
            .ask_question(question)
            .await
            .map_err(|err|Status::internal(err.to_string()))?;

        
        let sources = rag_answer
                    .sources
                    .into_iter()
                    .map(|chunk| Source{
                        file_path: chunk.file_path,
                        chunks_indexed: chunk.chunk_index as u32,
                        preview: chunk.text.chars().take(180).collect(),
                        score: chunk.score
                    })
                    .collect();

        Ok(Response::new(AskQuestionResponse{
            answer: rag_answer.answer,
            sources
        }))

    }
}