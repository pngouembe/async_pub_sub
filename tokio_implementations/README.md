# Tokio Implementations for Async Pub Sub

This crate provides concrete implementations of the `async_pub_sub` traits using Tokio primitives. It includes a `Publisher` implementation based on `tokio::sync::mpsc`.

## Features

*   **`MpscPublisher`**: A `Publisher` implementation using Tokio's multi-producer, single-consumer channel (`mpsc`).  This allows publishing messages to a single subscriber.

## Usage

1.  Add `tokio_implementations` to your `Cargo.toml`:

```toml
[dependencies]
tokio_implementations = "0.1.0" # Replace with the latest version
async_pub_sub = "0.1.0" # Replace with the latest version
```

2.  Use the `MpscPublisher` in your code:

```rust
use tokio_implementations::publisher::mpsc::MpscPublisher;
use async_pub_sub::Publisher;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let publisher = MpscPublisher::<i32>::new("my_publisher", 10);

    // Publish a message
    publisher.publish(42).await?;

    Ok(())
}
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
