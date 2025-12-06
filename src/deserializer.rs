use crate::event::QueryEvent;
use serde::Deserialize;
use std::time::{Duration, UNIX_EPOCH};
use thiserror::Error;

/// Errors that can occur during deserialization
#[derive(Error, Debug)]
pub enum DeserializationError {
    #[error("FlexBuffers deserialization failed: {0}")]
    FlexBuffersError(String),

    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),

    #[error("Missing required field: {0}")]
    MissingField(&'static str),
}

/// A deserialized batch of query events
#[derive(Debug)]
pub struct DeserializedBatch {
    /// The events in this batch
    pub events: Vec<QueryEvent>,

    /// Proxy instance identifier
    pub proxy_id: String,

    /// Batch sequence number
    pub batch_seq: u64,
}

/// Deserializes batches of QueryEvents from FlexBuffers format
pub struct FlexBuffersDeserializer;

#[derive(Deserialize)]
struct QueryEventBatch {
    events: Vec<DeserializableEvent>,
    proxy_id: String,
    batch_seq: u64,
}

#[derive(Deserialize)]
struct DeserializableEvent {
    event_id: String,
    timestamp_us: u64,
    query: String,
    normalized_query: Option<String>,
    value_fingerprints: Option<Vec<String>>,
    duration_us: u64,
    rows: Option<u64>,
    success: bool,
    error: Option<String>,
    database: String,
    connection_id: String,
}

impl FlexBuffersDeserializer {
    /// Deserialize a batch of events from FlexBuffers binary format
    ///
    /// Returns the deserialized events along with batch metadata
    pub fn deserialize_batch(bytes: &[u8]) -> Result<DeserializedBatch, DeserializationError> {
        // Deserialize from FlexBuffers
        let batch: QueryEventBatch = flexbuffers::from_slice(bytes)
            .map_err(|e| DeserializationError::FlexBuffersError(e.to_string()))?;

        // Convert DeserializableEvent to QueryEvent
        let events: Result<Vec<QueryEvent>, DeserializationError> = batch
            .events
            .into_iter()
            .map(Self::to_query_event)
            .collect();

        Ok(DeserializedBatch {
            events: events?,
            proxy_id: batch.proxy_id,
            batch_seq: batch.batch_seq,
        })
    }

    fn to_query_event(event: DeserializableEvent) -> Result<QueryEvent, DeserializationError> {
        // Convert timestamp from microseconds to SystemTime
        let timestamp = UNIX_EPOCH
            + Duration::from_micros(event.timestamp_us);

        // Convert duration from microseconds
        let duration = Duration::from_micros(event.duration_us);

        Ok(QueryEvent {
            event_id: event.event_id,
            timestamp,
            query: event.query,
            normalized_query: event.normalized_query,
            value_fingerprints: event.value_fingerprints,
            duration,
            rows: event.rows,
            success: event.success,
            error: event.error,
            database: event.database,
            connection_id: event.connection_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::QueryEventBuilder;
    use crate::serializer::FlatBuffersSerializer;

    #[test]
    fn test_deserialize_single_event() {
        // Create an event
        let event = QueryEventBuilder::new("SELECT 1")
            .connection_id("conn-123")
            .database("testdb")
            .duration(Duration::from_millis(5))
            .build();

        // Serialize it
        let bytes = FlatBuffersSerializer::serialize_batch(&[event.clone()], "proxy-1", 42);

        // Deserialize it
        let batch = FlexBuffersDeserializer::deserialize_batch(&bytes)
            .expect("Deserialization should succeed");

        assert_eq!(batch.proxy_id, "proxy-1");
        assert_eq!(batch.batch_seq, 42);
        assert_eq!(batch.events.len(), 1);

        let deserialized = &batch.events[0];
        assert_eq!(deserialized.query, "SELECT 1");
        assert_eq!(deserialized.connection_id, "conn-123");
        assert_eq!(deserialized.database, "testdb");
        assert_eq!(deserialized.duration, Duration::from_millis(5));
        assert!(deserialized.success);
        assert!(deserialized.error.is_none());
    }

    #[test]
    fn test_deserialize_batch() {
        let events = vec![
            QueryEventBuilder::new("SELECT 1")
                .connection_id("conn-1")
                .database("db1")
                .duration(Duration::from_millis(5))
                .build(),
            QueryEventBuilder::new("SELECT 2")
                .connection_id("conn-2")
                .database("db2")
                .duration(Duration::from_millis(10))
                .rows(42)
                .build(),
        ];

        let bytes = FlatBuffersSerializer::serialize_batch(&events, "proxy-1", 1);
        let batch = FlexBuffersDeserializer::deserialize_batch(&bytes)
            .expect("Deserialization should succeed");

        assert_eq!(batch.events.len(), 2);
        assert_eq!(batch.events[0].query, "SELECT 1");
        assert_eq!(batch.events[1].query, "SELECT 2");
        assert_eq!(batch.events[1].rows, Some(42));
    }

    #[test]
    fn test_deserialize_with_anonymization() {
        let event = QueryEventBuilder::new("SELECT * FROM users WHERE id = ?")
            .normalized_query("SELECT * FROM users WHERE id = ?")
            .value_fingerprints(vec!["abc123hash".to_string()])
            .connection_id("conn-1")
            .database("db1")
            .duration(Duration::from_millis(5))
            .build();

        let bytes = FlatBuffersSerializer::serialize_batch(&[event], "proxy-1", 1);
        let batch = FlexBuffersDeserializer::deserialize_batch(&bytes)
            .expect("Deserialization should succeed");

        assert_eq!(batch.events.len(), 1);
        assert_eq!(
            batch.events[0].normalized_query,
            Some("SELECT * FROM users WHERE id = ?".to_string())
        );
        assert_eq!(
            batch.events[0].value_fingerprints,
            Some(vec!["abc123hash".to_string()])
        );
    }

    #[test]
    fn test_deserialize_with_error() {
        let event = QueryEventBuilder::new("INVALID SQL")
            .connection_id("conn-1")
            .database("db1")
            .duration(Duration::from_millis(1))
            .success(false)
            .error("syntax error")
            .build();

        let bytes = FlatBuffersSerializer::serialize_batch(&[event], "proxy-1", 1);
        let batch = FlexBuffersDeserializer::deserialize_batch(&bytes)
            .expect("Deserialization should succeed");

        assert_eq!(batch.events.len(), 1);
        assert!(!batch.events[0].success);
        assert_eq!(batch.events[0].error, Some("syntax error".to_string()));
    }

    #[test]
    fn test_roundtrip_timestamp_precision() {
        let event = QueryEventBuilder::new("SELECT 1")
            .connection_id("conn-1")
            .database("db1")
            .duration(Duration::from_micros(12345))
            .build();

        let original_timestamp = event.timestamp;
        let original_duration = event.duration;

        let bytes = FlatBuffersSerializer::serialize_batch(&[event], "proxy-1", 0);
        let batch = FlexBuffersDeserializer::deserialize_batch(&bytes)
            .expect("Deserialization should succeed");

        // Check timestamp precision (microseconds)
        let deserialized_timestamp = batch.events[0].timestamp;
        let diff = if deserialized_timestamp > original_timestamp {
            deserialized_timestamp
                .duration_since(original_timestamp)
                .unwrap()
        } else {
            original_timestamp
                .duration_since(deserialized_timestamp)
                .unwrap()
        };

        // Should be within 1 microsecond
        assert!(diff < Duration::from_micros(1));

        // Check duration precision
        assert_eq!(batch.events[0].duration, original_duration);
    }

    #[test]
    fn test_deserialize_invalid_data() {
        let invalid_bytes = vec![0, 1, 2, 3, 4];
        let result = FlexBuffersDeserializer::deserialize_batch(&invalid_bytes);
        assert!(result.is_err());
    }
}
