# Tokio Runtime

Tokio is an asynchronous runtime for Rust.
It allows programs to run many async tasks concurrently.

tokio::spawn is used to start a new asynchronous task.
The spawned task runs independently on the Tokio runtime.

It is useful for background jobs, concurrent I/O, network servers, and worker tasks.