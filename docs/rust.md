# Rust Ownership

Rust ownership is a memory management model where each value has a single owner.
When the owner goes out of scope, the value is dropped automatically.

Borrowing allows references to data without taking ownership.
Rust supports immutable references and mutable references, but not both at the same time.

This prevents data races and memory safety bugs at compile time.