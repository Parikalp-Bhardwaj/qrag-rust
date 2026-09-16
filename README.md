# qrag-rust 🦀

A small, complete **Retrieval-Augmented Generation (RAG)** system in Rust — document loader, chunker, vector store, LLM service, gRPC API, and a terminal chat client.

Built with **Qdrant** for vector search, **Rig** for the AI application layer, **Tonic** for gRPC, and **OpenRouter** for embeddings and completion.

> If you've ever wondered what's underneath `langchain.create_retrieval_chain(...)` — this project is the answer, written in ~700 lines of Rust you can read end to end.

---

## ✨ Features

- 📄 Loads Markdown, plain text, and PDF documents from a local folder
- 🪓 Word-based chunking (~120 words per chunk) with UUID-tracked sources
- 🧠 Embeddings via OpenAI's `text-embedding-3-small` (1536-dim) through OpenRouter
- 🔎 Vector search powered by Qdrant with cosine similarity
- 💬 Grounded answers from `gpt-4o-mini` with explicit source attribution
- ⚡ **Auto-indexes `./docs` on startup** when the collection is empty — no manual step on first run
- 🚀 gRPC service exposing `AskQuestion` and `Reindex` endpoints
- 🖥️ Terminal chat client that talks to the server over gRPC
- 🧩 **Cargo workspace** — the RAG engine is a reusable `qrag-core` library
- 🐳 One-command Qdrant via Docker Compose

---

## 🏗️ Architecture

Two phases share a single `RagEngine`:

**Indexing** (runs automatically on startup when the collection is empty; or on demand via `Reindex`):

```
./docs → load → chunk → embed (OpenRouter) → store (Qdrant)
```

**Query** (every time someone asks):

```
question → embed → search Qdrant → top-k chunks → prompt → LLM → answer + sources
```

The same embedding model is used on both sides — that's what makes vector distances meaningful.

---

## 📂 Project layout

The project is a **Cargo workspace**. The engine lives in a reusable library
crate (`qrag-core`) so multiple frontends can share it; the gRPC server and
chat client live in `qrag-server`.

```
qrag-rust/
├── Cargo.toml                   # workspace root + shared dependency versions
├── docker-compose.yaml          # Qdrant container
├── proto/
│   └── rag.proto                # gRPC service definition
├── docs/                        # your knowledge base lives here
│   ├── grpc.md
│   ├── rust.md
│   ├── tokio.md
│   └── Rust-for-Network-Programming-and-Automation.pdf
└── crates/
    ├── qrag-core/               # reusable RAG engine (library)
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs           # re-exports the public API
    │       ├── config.rs        # env-var configuration
    │       ├── document_loader.rs  # reads .md, .txt, .pdf
    │       ├── chunker.rs       # splits into ~120-word chunks
    │       ├── qdrant_store.rs  # embeddings + vector storage
    │       ├── llm.rs           # prompt + completion
    │       └── rag.rs           # orchestration (+ auto-index)
    └── qrag-server/             # gRPC server + chat client
        ├── Cargo.toml
        ├── build.rs             # compiles .proto → Rust at build time
        └── src/
            ├── main.rs          # boots the gRPC server
            ├── grpc_service.rs  # tonic handlers
            └── bin/
                └── chat.rs      # terminal chat client
```

---

## 🛠️ Prerequisites

### Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version
```

### System packages (Ubuntu/Debian)

```bash
sudo apt update
sudo apt install -y \
    build-essential pkg-config libssl-dev \
    clang cmake protobuf-compiler poppler-utils
```

| Package | Why |
|---|---|
| `build-essential` | GCC/G++ for native crate compilation |
| `pkg-config` | Locates system libraries |
| `libssl-dev` | OpenSSL headers for HTTPS crates |
| `clang` | LLVM toolchain (bindgen, ML runtime crates) |
| `cmake` | Used by several native ML libraries |
| `protobuf-compiler` | The `protoc` binary for gRPC code generation |
| `poppler-utils` | Provides `pdftotext`, used for PDF ingestion |

### Docker

```bash
sudo apt install -y docker.io docker-compose-plugin
sudo systemctl enable --now docker
docker --version
```

### macOS

```bash
brew install rustup protobuf poppler
brew install --cask docker
rustup-init
```

---

## 🚀 Quick start

### 1. Clone and enter the project

```bash
git clone https://github.com/Parikalp-Bhardwaj/qrag-rust
cd qrag-rust
```

### 2. Create your `.env` file

```bash
cat > .env << 'EOF'
# Required — get one at https://openrouter.ai/keys
OPENROUTER_API_KEY=sk-or-v1-paste-your-key-here

# Optional — defaults shown
SERVER_ADDR=127.0.0.1
PORT=50051
QDRANT_URL=http://127.0.0.1:6334
QDRANT_COLLECTION=question
EOF
```

| Variable | Default | What it controls |
|---|---|---|
| `OPENROUTER_API_KEY` | *(required)* | Authenticates embedding and LLM calls |
| `SERVER_ADDR` | `127.0.0.1` | Host the gRPC server binds to |
| `PORT` | `50051` | gRPC port for clients |
| `QDRANT_URL` | `http://127.0.0.1:6334` | Qdrant's gRPC endpoint (not 6333) |
| `QDRANT_COLLECTION` | `question` | Name of the vector collection |

⚠️ Never commit `.env`. Make sure it's in `.gitignore`.

### 3. Start Qdrant

```bash
docker compose up -d
```

Verify:

```bash
docker ps                          # coderag-qdrant should be listed
curl http://localhost:6333/healthz # → healthz check passed
```

The Qdrant dashboard is at **http://localhost:6333/dashboard**.

### 4. Add your documents

Drop `.md`, `.txt`, or `.pdf` files into `./docs/`. The repo ships with a few Rust notes to get you started.

### 5. Build and run the server

```bash
cargo run -p qrag-server --bin qrag-server
```

On first run (empty collection) the server indexes `./docs` automatically, so you'll see something like:

```
CodeRAG-rs gRPC server running on 127.0.0.1:50051
auto-indexed 386 chunks from ./docs
```

On later runs, if the collection already has data it skips re-indexing:

```
index already populated with 386 chunks — skipping auto-index
```

First build pulls a lot of crates and is slow. Subsequent builds are quick.

### 6. Ask a question

Because indexing happens automatically, you can ask right away. Interactive client:

```bash
cargo run -p qrag-server --bin chat
```

Or with `grpcurl`:

```bash
grpcurl -plaintext -d '{"question":"What is tokio?"}' \
  -import-path proto -proto rag.proto \
  127.0.0.1:50051 rag.RagService/AskQuestion
```

### 7. Re-index after adding documents (optional)

Auto-index only runs when the collection is empty, so after adding new files to `./docs/` trigger a manual re-index:

```bash
grpcurl -plaintext -d '{}' \
  -import-path proto -proto rag.proto \
  127.0.0.1:50051 rag.RagService/Reindex
```

Response:

```json
{
  "chunksIndexed": "386",
  "message": "Indexed 386 chunk into qdrant"
}
```

---

## 💬 The chat client

For an interactive terminal experience, run the chat binary instead of `grpcurl`:

```bash
cargo run -p qrag-server --bin chat
```

```
Connected to http://127.0.0.1:50051
Type a question and press Enter. Commands: /quit

you > what is tokio?

bot > Tokio is an asynchronous runtime for Rust that allows programs
      to run many async tasks concurrently...

  - ./docs/tokio.md  (score 0.581)
    # Tokio Runtime Tokio is an asynchronous runtime for Rust...
```

**Built-in commands:**

| Command | What it does |
|---|---|
| `/quit` or `/exit` | Exits the client |
| `Ctrl-D` | Same as `/quit` |

The server has to be running for the client to connect. To refresh the index after changing documents, call the `Reindex` RPC (see Quick start step 7).

---

## 🔌 gRPC API

Defined in [`proto/rag.proto`](proto/rag.proto):

```protobuf
service RagService {
  rpc AskQuestion(AskQuestionRequest) returns (AskQuestionResponse);
  rpc Reindex(ReindexRequest) returns (ReindexResponse);
}
```

### `AskQuestion`

**Request:**
```json
{ "question": "What is tokio?" }
```

**Response:**
```json
{
  "answer": "Tokio is an asynchronous runtime for Rust...",
  "sources": [
    {
      "filePath": "./docs/tokio.md",
      "chunksIndexed": 0,
      "preview": "# Tokio Runtime Tokio is an asynchronous runtime...",
      "score": 0.581
    }
  ]
}
```

### `Reindex`

Rebuilds the entire index from `./docs/`. Takes no parameters; returns the chunk count. The server also runs this automatically on startup when the collection is empty.

---

## 🐳 Docker commands cheat sheet

```bash
# Stop the container (data persists)
docker compose down

# Stop AND wipe vectors (use after changing embedding dimensions)
docker compose down -v

# View Qdrant logs
docker compose logs -f qdrant

# Restart
docker compose restart
```

The `-v` flag matters: it deletes the `qdrant_data` volume. You need this after any change to the vector dimension in `qdrant_store.rs`, otherwise the old collection sticks around with the wrong shape and every upsert fails.

---

## ⚙️ Configuration deep dive

All configuration is environment-driven. The relevant struct is in `crates/qrag-core/src/config.rs`:

```rust
pub struct Config {
    pub addr: String,             // SERVER_ADDR
    pub port: u16,                // PORT
    pub qdrant_url: String,       // QDRANT_URL
    pub qdrant_collection: String,// QDRANT_COLLECTION
}
```

To change models or chunk sizes, edit the constants in code:

| What | Where |
|---|---|
| Embedding model | `EMBEDDING_MODEL` in `crates/qrag-core/src/qdrant_store.rs` |
| Vector dimension | `VectorParamsBuilder::new(1536, ...)` in `crates/qrag-core/src/qdrant_store.rs` |
| LLM model | `.agent("openai/gpt-4o-mini")` in `crates/qrag-core/src/llm.rs` |
| Chunk size | `chunk_retrieve(load, 120)` in `crates/qrag-core/src/rag.rs` |
| Top-K retrieval | `self.qdrant_store.search(&question, 3)` in `crates/qrag-core/src/rag.rs` |

If you change the embedding model, **also update the vector dimension** and run `docker compose down -v` to clear the old collection.

---

## ⚠️ Gotchas

- **Vector dimension mismatch.** `text-embedding-3-small` is 1536-dim. Change one without the other and every upsert fails. Wipe the volume to recover.
- **REST vs gRPC port.** Qdrant exposes REST on 6333 and gRPC on 6334. The Rust app needs 6334.
- **PDFs need `pdftotext`.** If `poppler-utils` isn't installed, PDF documents fail to load and startup auto-index logs a warning.
- **Auto-index only runs when empty.** After adding new docs to `./docs/`, run the `Reindex` RPC — a restart alone won't pick them up if the collection already has data.
- **First index is slow.** Hundreds of OpenRouter API calls. Expect 30-60 seconds for the included docs.
- **`.env` is loaded from the current directory.** Always run `cargo` from the workspace root.

---

## 🔭 Roadmap / ideas

In rough order of payoff:

- [ ] **ratatui TUI** — a rich terminal interface in a new `qrag-tui` crate
- [ ] **File-watcher** — auto-reindex when files in `./docs` change
- [ ] **Smarter chunking** — sentence-aware splits, sliding-window overlap, or semantic chunking
- [ ] **Reranking** — pull top-20 from Qdrant, then use a cross-encoder to get top-3
- [ ] **Hybrid retrieval** — combine vector search with BM25 keyword search
- [ ] **Streaming responses** — gRPC server streaming so tokens arrive incrementally
- [ ] **Per-tenant collections** — one Qdrant collection per user/workspace
- [ ] **Observability** — per-step latency tracing (embedding, search, LLM)
- [ ] **HTTP/REST endpoint** — for clients that don't speak gRPC

---

## 🧱 Tech stack

| Layer | Choice |
|---|---|
| Language | Rust (edition 2024) |
| Async runtime | [Tokio](https://tokio.rs) |
| gRPC | [Tonic](https://github.com/hyperium/tonic) + [Prost](https://github.com/tokio-rs/prost) |
| LLM framework | [Rig](https://github.com/0xPlaygrounds/rig) |
| Vector DB | [Qdrant](https://qdrant.tech) (via `qdrant-client`) |
| Vector store glue | [`rig-qdrant`](https://crates.io/crates/rig-qdrant) |
| LLM provider | [OpenRouter](https://openrouter.ai) (embedding + completion) |
| Models | `openai/text-embedding-3-small`, `openai/gpt-4o-mini` |
| PDF text extraction | `pdftotext` (poppler) |



Built to learn. Read the source, break it, fork it. That's where the understanding comes from. 🦀