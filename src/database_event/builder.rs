//! Builders for constructing database events.

use super::types::*;
use std::time::{SystemTime, UNIX_EPOCH};

/// Builder for constructing a single DatabaseEvent.
pub struct DatabaseEventBuilder {
    event: DatabaseEvent,
}

impl DatabaseEventBuilder {
    /// Create a new builder for a DML operation.
    pub fn new(operation: OperationType, schema: impl Into<String>, table: impl Into<String>) -> Self {
        Self {
            event: DatabaseEvent {
                event_id: None,
                timestamp_us: current_timestamp_us(),
                operation,
                schema: schema.into(),
                table: table.into(),
                position: 0,
                transaction_id: 0,
                new_row: None,
                old_row: None,
                columns: Vec::new(),
                relation_meta: None,
                ddl_sql: None,
                ddl_object_type: None,
            },
        }
    }

    /// Create a builder for an INSERT event.
    pub fn insert(schema: impl Into<String>, table: impl Into<String>) -> Self {
        Self::new(OperationType::Insert, schema, table)
    }

    /// Create a builder for an UPDATE event.
    pub fn update(schema: impl Into<String>, table: impl Into<String>) -> Self {
        Self::new(OperationType::Update, schema, table)
    }

    /// Create a builder for a DELETE event.
    pub fn delete(schema: impl Into<String>, table: impl Into<String>) -> Self {
        Self::new(OperationType::Delete, schema, table)
    }

    /// Create a builder for a snapshot row.
    pub fn snapshot_row(schema: impl Into<String>, table: impl Into<String>) -> Self {
        Self::new(OperationType::SnapshotRow, schema, table)
    }

    /// Create a builder for BEGIN transaction.
    pub fn begin() -> Self {
        Self::new(OperationType::Begin, "", "")
    }

    /// Create a builder for COMMIT transaction.
    pub fn commit() -> Self {
        Self::new(OperationType::Commit, "", "")
    }

    /// Create a builder for a DDL event.
    pub fn ddl(sql: impl Into<String>, object_type: impl Into<String>) -> Self {
        let mut builder = Self::new(OperationType::Ddl, "", "");
        builder.event.ddl_sql = Some(sql.into());
        builder.event.ddl_object_type = Some(object_type.into());
        builder
    }

    /// Set the event ID.
    pub fn event_id(mut self, id: impl Into<String>) -> Self {
        self.event.event_id = Some(id.into());
        self
    }

    /// Set the timestamp in microseconds.
    pub fn timestamp_us(mut self, ts: u64) -> Self {
        self.event.timestamp_us = ts;
        self
    }

    /// Set the timestamp from SystemTime.
    pub fn timestamp(mut self, ts: SystemTime) -> Self {
        self.event.timestamp_us = systemtime_to_us(ts);
        self
    }

    /// Set the replication position (LSN).
    pub fn position(mut self, pos: u64) -> Self {
        self.event.position = pos;
        self
    }

    /// Set the transaction ID.
    pub fn transaction_id(mut self, xid: u64) -> Self {
        self.event.transaction_id = xid;
        self
    }

    /// Set the new row data.
    pub fn new_row(mut self, row: Row) -> Self {
        self.event.new_row = Some(row);
        self
    }

    /// Set the old row data.
    pub fn old_row(mut self, row: Row) -> Self {
        self.event.old_row = Some(row);
        self
    }

    /// Set column names.
    pub fn columns(mut self, columns: Vec<String>) -> Self {
        self.event.columns = columns;
        self
    }

    /// Set relation metadata.
    pub fn relation_meta(mut self, meta: RelationMeta) -> Self {
        self.event.relation_meta = Some(meta);
        self
    }

    /// Build the event.
    pub fn build(self) -> DatabaseEvent {
        self.event
    }
}

/// Builder for constructing batches of events with efficient serialization.
pub struct BatchBuilder {
    events: Vec<DatabaseEvent>,
    relations: Vec<RelationMeta>,
    source_id: Option<String>,
    batch_seq: u64,
    current_bytes: usize,
    max_events: usize,
    max_bytes: usize,
}

impl BatchBuilder {
    /// Default maximum events per batch.
    pub const DEFAULT_MAX_EVENTS: usize = 1000;
    /// Default maximum bytes per batch (1MB).
    pub const DEFAULT_MAX_BYTES: usize = 1_000_000;

    /// Create a new batch builder with default limits.
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            relations: Vec::new(),
            source_id: None,
            batch_seq: 0,
            current_bytes: 0,
            max_events: Self::DEFAULT_MAX_EVENTS,
            max_bytes: Self::DEFAULT_MAX_BYTES,
        }
    }

    /// Set the source ID.
    pub fn source_id(mut self, id: impl Into<String>) -> Self {
        self.source_id = Some(id.into());
        self
    }

    /// Set the batch sequence number.
    pub fn batch_seq(mut self, seq: u64) -> Self {
        self.batch_seq = seq;
        self
    }

    /// Set the maximum number of events per batch.
    pub fn max_events(mut self, max: usize) -> Self {
        self.max_events = max;
        self
    }

    /// Set the maximum bytes per batch.
    pub fn max_bytes(mut self, max: usize) -> Self {
        self.max_bytes = max;
        self
    }

    /// Add an event to the batch.
    /// Returns Some(batch) if the batch is full and was flushed.
    pub fn add_event(&mut self, event: DatabaseEvent) -> Option<DatabaseEventBatch> {
        let event_size = event.size_bytes();

        // Check if adding this event would exceed limits
        if !self.events.is_empty()
            && (self.events.len() >= self.max_events
                || self.current_bytes + event_size > self.max_bytes)
        {
            // Flush current batch
            let batch = self.flush();
            // Add the event to the new batch
            self.current_bytes = event_size;
            self.events.push(event);
            Some(batch)
        } else {
            self.current_bytes += event_size;
            self.events.push(event);
            None
        }
    }

    /// Add relation metadata to the batch.
    pub fn add_relation(&mut self, meta: RelationMeta) {
        // Only add if not already present
        if !self.relations.iter().any(|r| r.rel_id == meta.rel_id) {
            self.relations.push(meta);
        }
    }

    /// Check if the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get the current number of events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Flush the current batch and return it.
    pub fn flush(&mut self) -> DatabaseEventBatch {
        let batch = DatabaseEventBatch {
            events: std::mem::take(&mut self.events),
            source_id: self.source_id.clone(),
            batch_seq: self.batch_seq,
            relations: std::mem::take(&mut self.relations),
            control_directive: None,
            sequence_values: None,
        };

        self.batch_seq += 1;
        self.current_bytes = 0;

        batch
    }

    /// Finish building and return any remaining events as a batch.
    pub fn finish(mut self) -> Option<DatabaseEventBatch> {
        if self.events.is_empty() {
            None
        } else {
            Some(self.flush())
        }
    }
}

impl Default for BatchBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Utility functions

fn current_timestamp_us() -> u64 {
    systemtime_to_us(SystemTime::now())
}

fn systemtime_to_us(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_builder() {
        let event = DatabaseEventBuilder::insert("public", "users")
            .position(12345)
            .transaction_id(100)
            .columns(vec!["id".to_string(), "name".to_string()])
            .new_row(Row::new(vec![
                ColumnValue::from_pg_binary(TypeTag::Int32, 23, vec![0, 0, 0, 1]),
                ColumnValue::from_pg_binary(TypeTag::Text, 25, b"Alice".to_vec()),
            ]))
            .build();

        assert_eq!(event.operation, OperationType::Insert);
        assert_eq!(event.schema, "public");
        assert_eq!(event.table, "users");
        assert_eq!(event.position, 12345);
    }

    #[test]
    fn test_batch_builder() {
        let mut batch = BatchBuilder::new()
            .source_id("test")
            .max_events(2);

        let event1 = DatabaseEventBuilder::insert("public", "users").build();
        let event2 = DatabaseEventBuilder::insert("public", "users").build();
        let event3 = DatabaseEventBuilder::insert("public", "users").build();

        // First two events should not trigger flush
        assert!(batch.add_event(event1).is_none());
        assert!(batch.add_event(event2).is_none());

        // Third event should trigger flush
        let flushed = batch.add_event(event3);
        assert!(flushed.is_some());
        assert_eq!(flushed.unwrap().events.len(), 2);

        // Finish should return remaining event
        let remaining = batch.finish();
        assert!(remaining.is_some());
        assert_eq!(remaining.unwrap().events.len(), 1);
    }
}
