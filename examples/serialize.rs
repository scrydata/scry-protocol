use scry_protocol::{FlatBuffersSerializer, QueryEventBuilder};
use std::time::Duration;

fn main() {
    println!("Scry Protocol - Serialization Example\n");

    // Create some sample events
    let events = vec![
        QueryEventBuilder::new("SELECT * FROM users WHERE id = 1")
            .connection_id("conn-001")
            .database("production")
            .duration(Duration::from_millis(5))
            .rows(1)
            .build(),
        QueryEventBuilder::new("INSERT INTO orders (user_id, total) VALUES (1, 99.99)")
            .connection_id("conn-001")
            .database("production")
            .duration(Duration::from_millis(3))
            .rows(1)
            .build(),
        QueryEventBuilder::new("INVALID SQL QUERY")
            .connection_id("conn-002")
            .database("production")
            .duration(Duration::from_millis(1))
            .success(false)
            .error("syntax error at or near \"INVALID\"")
            .build(),
    ];

    println!("Created {} events:", events.len());
    for (i, event) in events.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, event.query, if event.success { "success" } else { "failed" });
    }

    // Serialize the batch
    let proxy_id = "proxy-demo";
    let batch_seq = 0;
    let bytes = FlatBuffersSerializer::serialize_batch(&events, proxy_id, batch_seq);

    println!("\nSerialized batch:");
    println!("  Proxy ID: {}", proxy_id);
    println!("  Batch seq: {}", batch_seq);
    println!("  Event count: {}", events.len());
    println!("  Binary size: {} bytes", bytes.len());
    println!("  Average bytes/event: {} bytes", bytes.len() / events.len());

    // Show first 32 bytes in hex
    println!("\nFirst 32 bytes (hex):");
    let preview = &bytes[..bytes.len().min(32)];
    for (i, byte) in preview.iter().enumerate() {
        if i > 0 && i % 16 == 0 {
            println!();
        }
        print!("{:02x} ", byte);
    }
    println!();

    println!("\nSerialization complete! These bytes can now be sent over the network.");
}
