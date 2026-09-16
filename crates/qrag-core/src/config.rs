use std::env;

#[derive(Debug, Clone)]
pub struct Config{
    pub addr: String,
    pub port: u16,
    // pub doc_dir: String,
    pub qdrant_url: String,
    pub qdrant_collection: String
}

impl Config{
    pub fn from_env() -> Self{
        Self{

            addr: env::var("SERVER_ADDR")
                        .unwrap_or_else(|_|"127.0.0.1".to_string()),
            
            port: env::var("PORT")
                    .unwrap_or_else(|_|"50051".to_string())
                    .parse::<u16>()
                    .expect("SERVER_PORT must be a valid number"),

            // doc_dir: env::var("DOC")
            
            qdrant_url: env::var("QDRANT_URL")
                    .unwrap_or_else(|_|"http://127.0.0.1:6334".to_string()),
            

            qdrant_collection: env::var("QDRANT_COLLECTION")
                    .unwrap_or_else(|_|"question".to_string())

        }
    }

    pub fn server_addr(&self) -> String{
        format!("{}:{}", self.addr, self.port)
    }
}