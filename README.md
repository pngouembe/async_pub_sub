# Async Pub Sub

A library that aims at making the pub/sub pattern easy to use in asynchronous Rust.

## Overview

This crate provides a flexible and efficient foundation for building publish-subscribe systems in Rust using asynchronous programming with Tokio. It includes:

*   **Core Abstractions:**  [`Publisher`](async_pub_sub/src/publisher/mod.rs), [`Subscriber`](async_pub_sub/src/subscriber/mod.rs) traits for defining publishers and subscribers.
*   **Derive Macros:**  Convenient macros in the `async_pub_sub_macros` crate to automatically generate publisher and subscriber implementations.
*   **Extensibility:**  Middleware layers for publishers and subscribers to add custom logic like logging or debugging.
*   **Example Implementations:**  Ready-to-use implementations for common use cases.

### Core Components

- **Publisher:** Define how messages are published using the [`Publisher`](async_pub_sub/src/publisher/mod.rs) trait.
- **Subscriber:** Define how messages are consumed using the [`Subscriber`](async_pub_sub/src/subscriber/mod.rs) trait.

## Features

*   **Asynchronous:** Built on Tokio for high concurrency and responsiveness.
*   **Flexible:** Supports various message types and communication patterns.
*   **Extensible:** Easily add custom middleware layers.
*   **Macro Support:**  Simplify implementation with derive macros.
*   **Testable:** Includes comprehensive tests to ensure reliability.

## Getting Started

Add `async_pub_sub` to your `Cargo.toml`:

```toml
[dependencies]
async_pub_sub = "0.1.0" # Replace with the latest version
async_pub_sub_macros = "0.1.0" # Replace with the latest version
```

See the [async_pub_sub/examples/](async_pub_sub/examples/) directory for usage examples.


## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
