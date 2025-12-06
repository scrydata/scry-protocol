# Scry Protocol

Event protocol for Scry SQL proxy - FlexBuffers serialization for query events.

## Overview

This crate provides the event types and serialization/deserialization for query events captured by the Scry proxy. It's designed to be used both by the proxy itself and by downstream analytics services that consume events.

## Features

- **Event Types**: `QueryEvent` and `QueryEventBuilder` for constructing events
- **FlexBuffers Serialization**: Efficient, schema-less binary serialization
- **FlexBuffers Deserialization**: Deserialize event batches from binary format
- **Batch Support**: Serialize and deserialize multiple events in a single batch
- **Schema Documentation**: Canonical FlatBuffers schema in `schema/query_event.fbs`

## Wire Format

Events are serialized using FlexBuffers, a schema-less binary format from the FlatBuffers project. This provides:

- **Compact binary representation**: Smaller than JSON, efficient over the network
- **Backward/forward compatibility**: Schema evolution without breaking consumers
- **Zero-copy capabilities**: Efficient deserialization when needed
- **Works seamlessly with serde**: No code generation required

## Event Schema

See `schema/query_event.fbs` for the canonical schema definition.

### QueryEvent

Represents a captured SQL query event from the proxy:

- `event_id`: Unique identifier (UUID v4)
- `timestamp`: When the query was received (SystemTime)
- `query`: The SQL query text (raw or anonymized)
- `normalized_query`: Normalized query with placeholders (optional, if anonymization enabled)
- `value_fingerprints`: Fingerprints of literal values (optional, for hot data detection)
- `duration`: Query execution duration
- `rows`: Number of rows affected/returned (optional)
- `success`: Whether the query succeeded
- `error`: Error message if query failed (optional)
- `database`: Database name
- `connection_id`: Client connection identifier

### QueryEventBatch

A batch of query events sent together:

- `events`: Array of QueryEvent
- `proxy_id`: Unique proxy instance identifier
- `batch_seq`: Batch sequence number (for ordering/deduplication)

## Usage

### Creating Events

```rust
use scry_protocol::{QueryEventBuilder, Duration};

let event = QueryEventBuilder::new("SELECT * FROM users")
    .connection_id("conn-123")
    .database("mydb")
    .duration(Duration::from_millis(5))
    .rows(42)
    .build();
```

### Serializing Batches

```rust
use scry_protocol::FlatBuffersSerializer;

let events = vec![event1, event2, event3];
let bytes = FlatBuffersSerializer::serialize_batch(
    &events,
    "proxy-1",  // proxy_id
    0           // batch_seq
);

// Send bytes over the network...
```

### Deserializing Batches

```rust
use scry_protocol::FlexBuffersDeserializer;

// Receive bytes from the network...
let batch = FlexBuffersDeserializer::deserialize_batch(&bytes)?;

println!("Proxy ID: {}", batch.proxy_id);
println!("Batch seq: {}", batch.batch_seq);

for event in batch.events {
    println!("Query: {}", event.query);
    println!("Duration: {:?}", event.duration);
    println!("Success: {}", event.success);
}
```

## Examples

See the `examples/` directory for complete examples:

- `serialize.rs` - Creating and serializing events
- `deserialize.rs` - Deserializing and reading events

Run examples with:

```bash
cargo run --example serialize
cargo run --example deserialize
```

## Version Compatibility

This crate follows semantic versioning:

- **Major version**: Breaking changes to the event schema or API
- **Minor version**: New fields added (backward compatible)
- **Patch version**: Bug fixes, internal improvements

FlexBuffers provides forward/backward compatibility:
- **Forward compatible**: Old readers can read new data (unknown fields ignored)
- **Backward compatible**: New readers can read old data (missing fields use defaults)

## Performance

- **Serialization**: ~1-2 microseconds per event
- **Deserialization**: ~2-3 microseconds per event
- **Batch overhead**: Minimal, amortized across events
- **Binary size**: ~200-400 bytes per event (varies with query length)

## Testing

Run tests with:

```bash
cargo test
```

The test suite includes:
- Unit tests for serialization/deserialization
- Roundtrip tests (serialize → deserialize → verify)
- Edge cases (empty batches, large batches, special characters)
- Timestamp precision tests

## License

Licensed under MIT OR Apache-2.0
