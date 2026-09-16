# qrag-rust 🦀

A small, complete **Retrieval-Augmented Generation (RAG)** system in Rust — document loader, chunker, vector store, LLM service, gRPC API, and an interactive terminal UI.

Built with **Qdrant** for vector search, **Rig** for the AI application layer, **Tonic** for gRPC, **ratatui** for the terminal UI, and **OpenRouter** for embeddings and completion.

> If you've ever wondered what's underneath `langchain.create_retrieval_chain(...)` — this project is the answer, written in readable Rust you can follow end to end.

<!-- ![demo](docs/demo.gif)  -->

---

## ✨ Features

- 📄 Loads Markdown, plain text, and PDF documents from a local folder
- 🪓 Word-based chunking (~120 words per chunk) with UUID-tracked sources
- 🧠 Embeddings via OpenAI's `text-embedding-3-small` (1536-dim) through OpenRouter
- 🔎 Vector search powered by Qdrant with cosine similarity
- 💬 Grounded answers with explicit source attribution
- ⚡ **Streaming answers** — replies type out token-by-token
- 🖥️ **Interactive ratatui TUI** — chat + live sources panel, spinner and timer
- 🔁 **Auto-indexes `./docs` on startup** when the collection is empty
- 🔧 **Configurable LLM model** via the `MODEL` env var
- 🚀 gRPC service exposing `AskQuestion` and `Reindex` endpoints
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
question → embed → search Qdrant → top-k chunks → prompt → LLM (streamed) → answer + sources
```

The same embedding model is used on both sides — that's what makes vector distances meaningful.

---

## 📂 Project layout

A **Cargo workspace**: the engine is a reusable library (`qrag-core`); the
gRPC server + chat client live in `qrag-server`; the terminal UI is `qrag-tui`.

```
qrag-rust/
├── Cargo.toml                   # workspace root + shared dependency versions
├── docker-compose.yaml          # Qdrant container
├── proto/
│   └── rag.proto                # gRPC service definition
├── docs/                        # your knowledge base lives here
└── crates/
    ├── qrag-core/               # reusable RAG engine (library)
    │   └── src/
    │       ├── lib.rs           # re-exports the public API
    │       ├── config.rs        # env-var configuration
    │       ├── document_loader.rs  # reads .md, .txt, .pdf
    │       ├── chunker.rs       # splits into ~120-word chunks
    │       ├── qdrant_store.rs  # embeddings + vector storage
    │       ├── llm.rs           # prompt + streamed completion
    │       └── rag.rs           # orchestration (+ auto-index, streaming)
    ├── qrag-server/             # gRPC server + chat client
    │   ├── build.rs             # compiles .proto → Rust at build time
    │   └── src/
    │       ├── main.rs          # boots the gRPC server
    │       ├── grpc_service.rs  # tonic handlers
    │       └── bin/chat.rs      # simple terminal chat client
    └── qrag-tui/                # interactive ratatui UI (embeds qrag-core)
        └── src/
            ├── main.rs          # boots engine + opens the UI
            ├── app.rs           # state + async event loop
            └── ui.rs            # layout / rendering
```

---

## 🛠️ Prerequisites

### Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version   # needs a recent toolchain (edition 2024)
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
| `clang` | LLVM toolchain (bindgen) |
| `cmake` | Used by several native libraries |
| `protobuf-compiler` | The `protoc` binary for gRPC code generation |
| `poppler-utils` | Provides `pdftotext`, used for PDF ingestion |

### Docker

```bash
sudo apt install -y docker.io docker-compose-plugin
sudo systemctl enable --now docker
```

### macOS

```bash
brew install rustup protobuf poppler
brew install --cask docker
rustup-init
```

---

## 🚀 Quick start

### 1. Clone

```bash
git clone https://github.com/Parikalp-Bhardwaj/qrag-rust
cd qrag-rust
```

### 2. Create your `.env`

```bash
cat > .env << 'EOF'
# Required — get one at https://openrouter.ai/keys
OPENROUTER_API_KEY=sk-or-v1-paste-your-key-here

# Optional — defaults shown
SERVER_ADDR=127.0.0.1
PORT=50051
QDRANT_URL=http://127.0.0.1:6334
QDRANT_COLLECTION=question
MODEL=openai/gpt-4o-mini
EOF
```

| Variable | Default | What it controls |
|---|---|---|
| `OPENROUTER_API_KEY` | *(required)* | Authenticates embedding and LLM calls |
| `SERVER_ADDR` | `127.0.0.1` | Host the gRPC server binds to |
| `PORT` | `50051` | gRPC port for clients |
| `QDRANT_URL` | `http://127.0.0.1:6334` | Qdrant's gRPC endpoint (not 6333) |
| `QDRANT_COLLECTION` | `question` | Name of the vector collection |
| `MODEL` | `openai/gpt-4o-mini` | OpenRouter model used for answers |

⚠️ Never commit `.env`. Make sure it's in `.gitignore`.

### 3. Start Qdrant

```bash
docker compose up -d
```

### 4. Add your documents

Drop `.md`, `.txt`, or `.pdf` files into `./docs/`.

### 5. Run the TUI (recommended)

The terminal UI embeds the engine — no separate server needed:

```bash
cargo run -p qrag-tui
```

On first run it auto-indexes `./docs`, then opens a two-panel interface: a
**Chat** panel on the left and a **Sources** panel (file · similarity score ·
preview) on the right. Type a question, press **Enter**, and the answer streams
in live while the header shows a spinner and elapsed timer.

Keys: **Enter** send · **↑/↓** scroll · **Esc** / **Ctrl-C** quit.

### Alternative: gRPC server + clients

Run the server, then talk to it from the simple chat client or `grpcurl`:

```bash
# terminal 1 — server (auto-indexes on first run)
cargo run -p qrag-server --bin qrag-server

# terminal 2 — chat client
cargo run -p qrag-server --bin chat
```

Or query directly:

```bash
grpcurl -plaintext -d '{"question":"What is tokio?"}' \
  -import-path proto -proto rag.proto \
  127.0.0.1:50051 rag.RagService/AskQuestion
```

### Re-index after adding documents

Auto-index only runs when the collection is empty, so after adding new files
call the `Reindex` RPC:

```bash
grpcurl -plaintext -d '{}' \
  -import-path proto -proto rag.proto \
  127.0.0.1:50051 rag.RagService/Reindex
```

---

## 🔌 gRPC API

Defined in [`proto/rag.proto`](proto/rag.proto):

```protobuf
service RagService {
  rpc AskQuestion(AskQuestionRequest) returns (AskQuestionResponse);
  rpc Reindex(ReindexRequest) returns (ReindexResponse);
}
```

`AskQuestion` returns an answer plus the source chunks (file path, preview,
similarity score). `Reindex` rebuilds the index from `./docs/` and returns the
chunk count; the server also runs it automatically on startup when the
collection is empty.

---

## ⚙️ Configuration deep dive

Configuration is environment-driven (`crates/qrag-core/src/config.rs`). To
change chunking or the embedding model, edit the constants in code:

| What | Where |
|---|---|
| Embedding model | `EMBEDDING_MODEL` in `crates/qrag-core/src/qdrant_store.rs` |
| Vector dimension | `VectorParamsBuilder::new(1536, ...)` in `crates/qrag-core/src/qdrant_store.rs` |
| LLM model | `MODEL` env var (or the default in `config.rs`) |
| Chunk size | `chunk_retrieve(load, 120)` in `crates/qrag-core/src/rag.rs` |
| Top-K retrieval | `self.qdrant_store.search(&question, 3)` in `crates/qrag-core/src/rag.rs` |

If you change the embedding model, **also update the vector dimension** and run `docker compose down -v` to clear the old collection.

---

## ⚠️ Gotchas

- **Vector dimension mismatch.** `text-embedding-3-small` is 1536-dim. Change one without the other and every upsert fails; wipe the volume (`docker compose down -v`) to recover.
- **REST vs gRPC port.** Qdrant exposes REST on 6333 and gRPC on 6334. The Rust app needs 6334.
- **PDFs need `pdftotext`.** Without `poppler-utils`, PDF documents fail to load and startup auto-index logs a warning.
- **Auto-index only runs when empty.** After adding new docs, call the `Reindex` RPC — a restart alone won't pick them up if the collection already has data.
- **`.env` is loaded from the current directory.** Run `cargo` from the workspace root.

---

## 🔭 Roadmap / ideas

- [x] Cargo workspace with reusable `qrag-core`
- [x] Auto-index on startup
- [x] Interactive ratatui TUI
- [x] Streaming responses
- [x] Configurable LLM model
- [ ] File-watcher to auto-reindex when docs change
- [ ] Reranking (top-20 → cross-encoder → top-3)
- [ ] Hybrid retrieval (vector + BM25)
- [ ] Per-tenant collections
- [ ] HTTP/REST endpoint
- [ ] Tauri desktop app

---

## 🧱 Tech stack

| Layer | Choice |
|---|---|
| Language | Rust (edition 2024) |
| Async runtime | [Tokio](https://tokio.rs) |
| gRPC | [Tonic](https://github.com/hyperium/tonic) + [Prost](https://github.com/tokio-rs/prost) |
| Terminal UI | [ratatui](https://ratatui.rs) + [crossterm](https://github.com/crossterm-rs/crossterm) |
| LLM framework | [Rig](https://github.com/0xPlaygrounds/rig) |
| Vector DB | [Qdrant](https://qdrant.tech) (via `qdrant-client`) |
| Vector store glue | [`rig-qdrant`](https://crates.io/crates/rig-qdrant) |
| LLM provider | [OpenRouter](https://openrouter.ai) (embedding + completion) |
| PDF text extraction | `pdftotext` (poppler) |



Built to learn. Read the source, break it, fork it. 🦀