use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;


#[derive(Debug, Clone)]
pub struct Document{
    pub file_path: String,
    pub content: String
}


pub fn loader_documents_from_dir
        <P: AsRef<Path>>
        (dir: P) -> Result<Vec<Document>>{
        
        let mut documents = Vec::new();

        let entries = fs::read_dir(&dir)
                .with_context(|| format!("Failed to read directory: {:?}", dir.as_ref()))?;

        for entry in entries{
            let entry = entry?;
            let path: PathBuf = entry.path();

            if !path.is_file(){
                continue;
            }

            let Some(extension) = path.extension() else{
                continue;
            };

            let extension = extension.to_string_lossy().to_lowercase();

            let content = match extension.as_str(){
                "md" | "text" | "rs"  =>{
                    fs::read_to_string(&path)
                        .with_context(||format!("Failed to read text file :{:?}",path))?
                }
                "pdf" =>{
                    extract_pdf_text(&path)
                        .with_context(||format!("Failed to extract text from PDF: {:?}",path))?
                }
                _ => continue,
            };

            documents.push(Document{
                file_path: path.to_string_lossy().to_string(),
                content
            });
        }

    Ok(documents)
}



fn extract_pdf_text(path: &Path) -> Result<String>{
    let output = Command::new("pdftotext")
        .arg(path)
        .arg("-")
        .output()
        .with_context(|| format!("Failed to run pdftotext for {:?} ",path))?;

    if !output.status.success(){
        anyhow::bail!(
            "pdftotext failed for {:?}: {}",
            path,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}