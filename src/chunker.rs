use crate::document_loader::Document;

#[derive(Debug, Clone)]
pub struct DocumentChunk{
    pub id: String,
    pub file_path: String,
    pub chunk_index: usize,
    pub text: String
}

pub fn chunk_retrieve(
    documents: Vec<Document>,
    max_word: usize
) -> Vec<DocumentChunk>{

    let mut chunks = Vec::new();

    for docs in documents{
        let words: Vec<&str> = docs.content.split_whitespace().collect();
        for (chunk_idx, word_chunk) in words.chunks(max_word).enumerate(){
            let text = word_chunk.join(" ");

            if text.trim().is_empty(){
                continue;
            }

            chunks.push(DocumentChunk{
                id: uuid::Uuid::new_v4().to_string(),
                file_path: docs.file_path.clone(),
                chunk_index: chunk_idx,
                text,
            });
        }
    }
    chunks
}

