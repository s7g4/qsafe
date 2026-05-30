# 9. Asynchronous WebSocket Registry & Offline Message Queues

## Status

Accepted

## Context

In the initial prototype, WebSocket connections were stored using a standard `HashMap` wrapped in a synchronized mutex (`Arc<Mutex<HashMap<String, SplitSink<WebSocket, Message>>>>`). This pattern blocks the main executing thread when writing or reading from connections, leading to high-contention scenarios, possible deadlocks under heavy load, and poor vertical scalability.

Additionally, if a user is offline when a message is routed to them, the gateway has no mechanism to buffer or queue the message. Real-time messages are silently dropped or fail to deliver, leading to loss of cryptographic messages and communication context.

## Decision

We will:
1. Replace the blocking mutex-based WebSocket map with an asynchronous connection registry utilizing the **Actor Model** and Tokio channels (`tokio::sync::mpsc`).
   - The connection registry (`WebSocketRegistry`) will communicate with a background actor loop (`WebSocketRegistryActor`) using non-blocking channel messages.
   - When a WebSocket connection starts, it registers an unbounded sender (`mpsc::UnboundedSender<Message>`) to which other connections can push messages.
2. Introduce a persistent offline message queue using a new PostgreSQL table `offline_messages`.
   - If a message routing request returns `false` (meaning the recipient is not registered / offline), the gateway will persist the message in the `offline_messages` table.
   - On WebSocket connection (`Join` command), the gateway will query the database for any pending offline messages for the connecting user, deliver them in order, and remove them from the database queue.

## Consequences

- **Concurrency**: Lock contention on connection maps is eliminated, as all mutation and lookup operations are serialized within the single-threaded background actor loop.
- **Reliability**: Messages are guaranteed to be buffered in the database if the recipient is offline and delivered as soon as they rejoin the network.
- **Portability**: Codebase retains zero external dependency modifications, relying entirely on the standard features of `tokio` and `sqlx`.
