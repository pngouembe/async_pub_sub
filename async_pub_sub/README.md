# Async Pub Sub

A library that aims at making the pub/sub pattern easy to use in asynchronous Rust.

## Overview

This crate provides a flexible and efficient foundation for building publish-subscribe systems in Rust using asynchronous programming. It includes:

*   **Core Abstractions:**  [`Publisher`](src/publisher/mod.rs), [`Subscriber`](src/subscriber/mod.rs) traits for defining publishers and subscribers.
*   **Derive Macros:**  Convenient macros available using the `macros` features to automatically generate publisher and subscriber implementations (see [async_pub_sub_macros](../async_pub_sub_macros/) for more details).
*   **Extensibility:**  Middleware layers for publishers and subscribers to add custom logic like logging or debugging.
*   **Example Implementations:**  Ready-to-use implementations for common use cases.

## Features

*   **Asynchronous:** Built for async rust.
*   **Flexible:** Generic implementation allowing to use custom messages.
*   **Extensible:** Easily add custom middleware layers.
*   **Macro Support:**  Simplify implementation with derive macros.

## Getting Started

Add `async_pub_sub` to your `Cargo.toml`:

```toml
[dependencies]
async_pub_sub = { version = "0.1.0", features = ["macros"] } # Replace with the latest version
```

See the [examples/](examples/) directory for usage examples.


## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## Note

This project is inspired by the [Tower](https://github.com/tower-rs/tower/tree/master) project