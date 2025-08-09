# NATS IPC Example

This example demonstrates interprocess communication using the async_pub_sub library with NATS as the transport layer.

## Overview

Two applications (`app1` and `app2`) communicate with each other through:

1. **Pub/Sub Messages**: `PingMeNotification` messages are published to trigger the ping cycle
2. **RPC Calls**: Ping requests and responses are exchanged using dedicated NATS subjects

## Application Flow

1. **app1** (with `SEND_FIRST=true`) starts the cycle by publishing a `PingMeNotification`
2. **app2** receives the notification and makes an RPC call to **app1** 
3. **app1** responds to the RPC and then publishes a new `PingMeNotification`
4. **app2** receives the notification and the cycle continues

## Running the Example

### Prerequisites

- Docker and Docker Compose
- Rust toolchain (if running locally)

### With Docker Compose

```bash
cd async_pub_sub_ipc/examples/nats_ipc
docker-compose up --build
```

This will start:
- A NATS server on port 4222
- app1 with `SEND_FIRST=true`
- app2 with `SEND_FIRST=false`

### Local Development

1. Start a NATS server:
```bash
docker run -p 4222:4222 -p 8222:8222 nats:2.10-alpine -js -m 8222
```

2. Run the applications in separate terminals:

Terminal 1 (app1):
```bash
cd /path/to/async_pub_sub
SEND_FIRST=true RUST_LOG=info cargo run --example nats_ipc
```

Terminal 2 (app2):
```bash  
cd /path/to/async_pub_sub
SEND_FIRST=false RUST_LOG=info cargo run --example nats_ipc
```

## Configuration

Environment variables:
- `SEND_FIRST`: Set to "true" for the app that starts the ping cycle
- `NATS_URL`: NATS server URL (default: "nats://localhost:4222")
- `RUST_LOG`: Log level (default: "info")

## Architecture

The example uses:
- `NatsPublisher<T>`: Publishes messages to NATS subjects with JSON serialization
- `NatsSubscriber<T>`: Subscribes to NATS subjects and deserializes JSON messages
- Custom RPC pattern using request/response message types instead of the built-in RPC macros (to support serialization over NATS)

## NATS Subjects

- `ping_notifications`: Shared subject for ping notifications
- `ping_requests_{app_name}`: Per-app subject for incoming ping requests
- `ping_responses_{app_name}`: Per-app subject for ping responses