use scry_protocol::{FlexBuffersDeserializer, FlatBuffersSerializer, QueryEventBuilder};
use std::time::Duration;

fn main() {
    println!("Scry Protocol - Deserialization Example\n");

    // First, create and serialize some events
    // (In a real scenario, these bytes would come from the network)
    println!("Creating sample batch...");
    let events = vec![
        QueryEventBuilder::new("SELECT * FROM users WHERE id = 1")
            .connection_id("conn-001")
            .database("production")
            .duration(Duration::from_millis(5))
            .rows(1)
            .build(),
        QueryEventBuilder::new("UPDATE users SET last_login = NOW() WHERE id = 1")
            .connection_id("conn-001")
            .database("production")
            .duration(Duration::from_millis(8))
            .rows(1)
            .build(),
        QueryEventBuilder::new("SELECT COUNT(*) FROM orders WHERE user_id = 1")
            .connection_id("conn-001")
            .database("production")
            .duration(Duration::from_millis(12))
            .rows(1)
            .build(),
    ];

    let bytes = FlatBuffersSerializer::serialize_batch(&events, "proxy-demo", 42);
    println!("Serialized {} events into {} bytes\n", events.len(), bytes.len());

    // Now deserialize the batch
    println!("Deserializing batch...");
    match FlexBuffersDeserializer::deserialize_batch(&bytes) {
        Ok(batch) => {
            println!("Deserialization successful!\n");

            println!("Batch metadata:");
            println!("  Proxy ID: {}", batch.proxy_id);
            println!("  Batch sequence: {}", batch.batch_seq);
            println!("  Event count: {}\n", batch.events.len());

            println!("Events:");
            for (i, event) in batch.events.iter().enumerate() {
                println!("  Event {}:", i + 1);
                println!("    ID: {}", event.event_id);
                println!("    Query: {}", event.query);
                println!("    Database: {}", event.database);
                println!("    Connection: {}", event.connection_id);
                println!("    Duration: {:?}", event.duration);
                println!("    Rows: {:?}", event.rows);
                println!("    Success: {}", event.success);
                if let Some(err) = &event.error {
                    println!("    Error: {}", err);
                }
                println!();
            }

            println!("All events deserialized successfully!");
        }
        Err(e) => {
            eprintln!("Deserialization failed: {}", e);
            std::process::exit(1);
        }
    }
}
