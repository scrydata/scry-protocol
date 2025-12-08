//! Database event types and serialization.
//!
//! This module provides types for database replication events (COPY, CDC, proxy)
//! with FlexBuffers serialization for efficient wire format.
//!
//! # Design
//!
//! The key design decision is storing PostgreSQL binary format bytes directly
//! in `ColumnValue.data` to enable zero-copy for the high-throughput COPY path.
//!
//! # Example
//!
//! ```rust
//! use scry_protocol::database_event::{
//!     DatabaseEventBuilder, BatchBuilder, Row, ColumnValue, TypeTag,
//!     serialize_batch, read_batch,
//! };
//!
//! // Build an event
//! let event = DatabaseEventBuilder::insert("public", "users")
//!     .position(12345)
//!     .new_row(Row::new(vec![
//!         ColumnValue::from_pg_binary(TypeTag::Int32, 23, vec![0, 0, 0, 1]),
//!     ]))
//!     .build();
//!
//! // Build a batch
//! let mut builder = BatchBuilder::new().source_id("my-source");
//! builder.add_event(event);
//! let batch = builder.finish().unwrap();
//!
//! // Serialize
//! let bytes = serialize_batch(&batch).unwrap();
//!
//! // Deserialize
//! let recovered = read_batch(&bytes).unwrap();
//! ```

mod types;
mod builder;
mod reader;

pub use types::*;
pub use builder::{DatabaseEventBuilder, BatchBuilder};
pub use reader::{serialize_batch, serialize_event, read_batch, read_event};
