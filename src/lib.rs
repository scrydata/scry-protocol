//! # Scry Protocol
//!
//! Event protocol for Scry SQL proxy. Provides types and serialization
//! for query events captured by the proxy.
//!
//! ## Features
//!
//! - **Event Types**: `QueryEvent` and `QueryEventBuilder` for constructing events
//! - **FlexBuffers Serialization**: Efficient, schema-less binary serialization
//! - **FlexBuffers Deserialization**: Deserialize event batches from binary format
//! - **Batch Support**: Serialize and deserialize multiple events in a single batch
//!
//! ## Wire Format
//!
//! Events are serialized using FlexBuffers, a schema-less binary format from the
//! FlatBuffers project. This provides:
//!
//! - Compact binary representation
//! - Backward/forward compatibility
//! - Zero-copy deserialization capabilities
//! - Works seamlessly with serde
//!
//! ## Example: Serialization
//!
//! ```rust
//! use scry_protocol::{QueryEventBuilder, FlatBuffersSerializer};
//! use std::time::Duration;
//!
//! let event = QueryEventBuilder::new("SELECT * FROM users")
//!     .connection_id("conn-123")
//!     .database("mydb")
//!     .duration(Duration::from_millis(5))
//!     .build();
//!
//! let batch = vec![event];
//! let bytes = FlatBuffersSerializer::serialize_batch(&batch, "proxy-1", 0);
//! ```
//!
//! ## Example: Deserialization
//!
//! ```rust
//! use scry_protocol::FlexBuffersDeserializer;
//!
//! # let bytes: Vec<u8> = vec![]; // From serialized batch
//! let result = FlexBuffersDeserializer::deserialize_batch(&bytes);
//! match result {
//!     Ok(batch) => {
//!         println!("Proxy ID: {}", batch.proxy_id);
//!         println!("Batch seq: {}", batch.batch_seq);
//!         println!("Events: {}", batch.events.len());
//!     }
//!     Err(e) => eprintln!("Deserialization error: {}", e),
//! }
//! ```

mod event;
mod serializer;
mod deserializer;
pub mod database_event;

pub use event::{QueryEvent, QueryEventBuilder};
pub use serializer::FlatBuffersSerializer;
pub use deserializer::{FlexBuffersDeserializer, DeserializedBatch};

// Re-export commonly used types for convenience
pub use std::time::{Duration, SystemTime};
