//! Serialization and deserialization for database events using FlexBuffers.

use super::types::*;
use anyhow::{Context, Result};
use serde::de::DeserializeOwned;

/// Serialize a batch to FlexBuffers format.
pub fn serialize_batch(batch: &DatabaseEventBatch) -> Result<Vec<u8>> {
    let mut serializer = flexbuffers::FlexbufferSerializer::new();
    serde::Serialize::serialize(batch, &mut serializer)
        .context("Failed to serialize DatabaseEventBatch")?;
    Ok(serializer.take_buffer())
}

/// Serialize a single event to FlexBuffers format.
pub fn serialize_event(event: &DatabaseEvent) -> Result<Vec<u8>> {
    let mut serializer = flexbuffers::FlexbufferSerializer::new();
    serde::Serialize::serialize(event, &mut serializer)
        .context("Failed to serialize DatabaseEvent")?;
    Ok(serializer.take_buffer())
}

/// Deserialize from FlexBuffers format.
fn deserialize_flexbuffers<T: DeserializeOwned>(data: &[u8]) -> Result<T> {
    let reader = flexbuffers::Reader::get_root(data)
        .context("Failed to read FlexBuffer root")?;
    T::deserialize(reader).context("Failed to deserialize")
}

/// Deserialize a batch from FlexBuffers format.
pub fn read_batch(data: &[u8]) -> Result<DatabaseEventBatch> {
    deserialize_flexbuffers(data)
}

/// Deserialize a single event from FlexBuffers format.
pub fn read_event(data: &[u8]) -> Result<DatabaseEvent> {
    deserialize_flexbuffers(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database_event::builder::DatabaseEventBuilder;

    #[test]
    fn test_serialize_deserialize_batch() {
        let event = DatabaseEventBuilder::insert("public", "users")
            .position(12345)
            .transaction_id(100)
            .columns(vec!["id".to_string(), "name".to_string()])
            .new_row(Row::new(vec![
                ColumnValue::from_pg_binary(TypeTag::Int32, 23, vec![0, 0, 0, 1]),
                ColumnValue::from_pg_binary(TypeTag::Text, 25, b"Alice".to_vec()),
            ]))
            .build();

        let batch = DatabaseEventBatch::with_events(vec![event])
            .with_source_id("test-source")
            .with_batch_seq(42);

        // Serialize
        let bytes = serialize_batch(&batch).expect("serialize failed");
        assert!(!bytes.is_empty());

        // Deserialize
        let recovered = read_batch(&bytes).expect("deserialize failed");
        assert_eq!(recovered.source_id, batch.source_id);
        assert_eq!(recovered.batch_seq, batch.batch_seq);
        assert_eq!(recovered.events.len(), 1);

        let event = &recovered.events[0];
        assert_eq!(event.schema, "public");
        assert_eq!(event.table, "users");
        assert_eq!(event.position, 12345);
    }

    #[test]
    fn test_serialize_deserialize_event() {
        let event = DatabaseEventBuilder::update("myschema", "orders")
            .position(99999)
            .new_row(Row::new(vec![
                ColumnValue::from_pg_binary(TypeTag::Int64, 20, vec![0, 0, 0, 0, 0, 0, 0, 42]),
            ]))
            .old_row(Row::new(vec![
                ColumnValue::from_pg_binary(TypeTag::Int64, 20, vec![0, 0, 0, 0, 0, 0, 0, 41]),
            ]))
            .build();

        // Serialize
        let bytes = serialize_event(&event).expect("serialize failed");
        assert!(!bytes.is_empty());

        // Deserialize
        let recovered = read_event(&bytes).expect("deserialize failed");
        assert_eq!(recovered.operation, OperationType::Update);
        assert_eq!(recovered.schema, "myschema");
        assert_eq!(recovered.table, "orders");
        assert!(recovered.new_row.is_some());
        assert!(recovered.old_row.is_some());
    }

    #[test]
    fn test_null_values_roundtrip() {
        let event = DatabaseEventBuilder::insert("public", "test")
            .new_row(Row::new(vec![
                ColumnValue::null(),
                ColumnValue::from_pg_binary(TypeTag::Text, 25, b"not null".to_vec()),
                ColumnValue::null(),
            ]))
            .build();

        let bytes = serialize_event(&event).expect("serialize failed");
        let recovered = read_event(&bytes).expect("deserialize failed");

        let row = recovered.new_row.expect("new_row should exist");
        assert!(row.values[0].is_null());
        assert!(!row.values[1].is_null());
        assert!(row.values[2].is_null());
    }

    #[test]
    fn test_relation_meta_roundtrip() {
        let meta = RelationMeta {
            rel_id: 12345,
            schema: "public".to_string(),
            table: "users".to_string(),
            columns: vec![
                ColumnMeta {
                    name: "id".to_string(),
                    type_oid: 23,
                    type_modifier: -1,
                    is_key: true,
                },
                ColumnMeta {
                    name: "name".to_string(),
                    type_oid: 25,
                    type_modifier: -1,
                    is_key: false,
                },
            ],
            replica_identity: ReplicaIdentity::Full,
        };

        let event = DatabaseEventBuilder::insert("public", "users")
            .relation_meta(meta.clone())
            .build();

        let bytes = serialize_event(&event).expect("serialize failed");
        let recovered = read_event(&bytes).expect("deserialize failed");

        let recovered_meta = recovered.relation_meta.expect("relation_meta should exist");
        assert_eq!(recovered_meta.rel_id, meta.rel_id);
        assert_eq!(recovered_meta.schema, meta.schema);
        assert_eq!(recovered_meta.columns.len(), 2);
        assert_eq!(recovered_meta.replica_identity, ReplicaIdentity::Full);
    }
}
