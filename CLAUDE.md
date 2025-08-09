# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust workspace containing an asynchronous publish-subscribe library with four main crates:

- **async_pub_sub**: Core pub/sub library with Publisher/Subscriber traits and implementations
- **async_pub_sub_macros**: Procedural macros for derive implementations and RPC interface generation
- **async_pub_sub_ipc**: IPC extensions (new crate)
- **tokio_implementations**: Tokio-specific implementations (excluded from releases)

## Development Commands

### Build and Test
```bash
# Build all workspace members
cargo build

# Build specific crate
cargo build -p async_pub_sub

# Run all tests
cargo test

# Run tests for specific crate
cargo test -p async_pub_sub

# Run specific test
cargo test test_pub_sub_i32

# Run example
cargo run --example client_server --features macros
```

### Development Workflow
```bash
# Check format
cargo fmt --check

# Format code
cargo fmt

# Check with clippy
cargo clippy

# Check documentation
cargo doc --no-deps --open
```

## Architecture

### Core Traits
- `Publisher<Message>`: Publishes messages asynchronously via `publish()` method
- `Subscriber<Message>`: Subscribes to publishers via `subscribe_to()` and receives with `receive()`
- `Forwarder`: Utility for forwarding messages between publishers/subscribers

### Key Components
- **PublisherImpl/SubscriberImpl**: Basic implementations of the core traits
- **Middleware System**: Builder pattern with layers for logging, debugging
- **RPC System**: Generated client/server code from `#[rpc_interface]` macro
- **Route Macros**: `route!()` and `routes!()` for connecting publishers to subscribers

### Macro System (`async_pub_sub_macros`)
- `#[derive(DerivePublisher)]`: Auto-implement Publisher trait
- `#[derive(DeriveSubscriber)]`: Auto-implement Subscriber trait  
- `#[rpc_interface]`: Generate RPC message enums and client/server traits
- `route!(publisher -> subscriber: MessageType)`: Connect single pub/sub pair
- `routes! { ... }`: Connect multiple pub/sub pairs

### Testing Patterns
Tests are organized numerically in `async_pub_sub/tests/`:
- `01_*`: Basic usage patterns
- `02_*-03_*`: Middleware/decorators
- `04_*`: Concurrency
- `05_*-18_*`: Advanced features (RPC, forwarders, etc.)
- `19_*`: Internal communication

Tests use `#[tokio::test]` and follow Setup/Exec/Check pattern.

## Key Files
- `async_pub_sub/src/lib.rs`: Main library exports
- `async_pub_sub/src/publisher/publisher_trait.rs`: Core Publisher trait
- `async_pub_sub/src/subscriber/subscriber_trait.rs`: Core Subscriber trait
- `async_pub_sub_macros/src/lib.rs`: All procedural macros
- `examples/client_server_pub_sub_example/`: Complete working example

## Release Configuration
Uses `release-plz.toml` for automated releases. The `tokio_implementations` crate is excluded from publishing (`publish = false`).