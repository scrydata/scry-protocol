//! Database event types matching the FlatBuffers schema.

use serde::{Deserialize, Serialize};

/// Type tag for fast dispatch without parsing OID.
/// Maps to PostgreSQL type categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum TypeTag {
    // Special
    Null = 0,

    // Basic types
    Bool = 1,
    Int16 = 2,
    Int32 = 3,
    Int64 = 4,
    Float32 = 5,
    Float64 = 6,
    Text = 7,
    Bytea = 8,

    // JSON types
    Json = 9,
    Jsonb = 10,

    // Other common types
    Uuid = 11,
    Timestamp = 12,
    TimestampTz = 13,
    Date = 14,
    Time = 15,
    TimeTz = 16,
    Interval = 17,
    Numeric = 18,

    // Bit types
    Bit = 20,
    Varbit = 21,

    // Geometric types
    Point = 30,
    Line = 31,
    Lseg = 32,
    Box = 33,
    Path = 34,
    Polygon = 35,
    Circle = 36,

    // Network types
    Inet = 40,
    Cidr = 41,
    MacAddr = 42,
    MacAddr8 = 43,

    // Range types
    Int4Range = 50,
    Int8Range = 51,
    NumRange = 52,
    TsRange = 53,
    TsTzRange = 54,
    DateRange = 55,

    // Money
    Money = 60,

    // XML
    Xml = 61,

    // Arrays (element type encoded in type_oid)
    Array = 100,

    // Extension point for custom/unknown types
    Custom = 127,
}

impl TypeTag {
    /// Convert from byte value.
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Null),
            1 => Some(Self::Bool),
            2 => Some(Self::Int16),
            3 => Some(Self::Int32),
            4 => Some(Self::Int64),
            5 => Some(Self::Float32),
            6 => Some(Self::Float64),
            7 => Some(Self::Text),
            8 => Some(Self::Bytea),
            9 => Some(Self::Json),
            10 => Some(Self::Jsonb),
            11 => Some(Self::Uuid),
            12 => Some(Self::Timestamp),
            13 => Some(Self::TimestampTz),
            14 => Some(Self::Date),
            15 => Some(Self::Time),
            16 => Some(Self::TimeTz),
            17 => Some(Self::Interval),
            18 => Some(Self::Numeric),
            20 => Some(Self::Bit),
            21 => Some(Self::Varbit),
            30 => Some(Self::Point),
            31 => Some(Self::Line),
            32 => Some(Self::Lseg),
            33 => Some(Self::Box),
            34 => Some(Self::Path),
            35 => Some(Self::Polygon),
            36 => Some(Self::Circle),
            40 => Some(Self::Inet),
            41 => Some(Self::Cidr),
            42 => Some(Self::MacAddr),
            43 => Some(Self::MacAddr8),
            50 => Some(Self::Int4Range),
            51 => Some(Self::Int8Range),
            52 => Some(Self::NumRange),
            53 => Some(Self::TsRange),
            54 => Some(Self::TsTzRange),
            55 => Some(Self::DateRange),
            60 => Some(Self::Money),
            61 => Some(Self::Xml),
            100 => Some(Self::Array),
            127 => Some(Self::Custom),
            _ => None,
        }
    }
}

/// Database operation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum OperationType {
    // DML operations
    Insert = 0,
    Update = 1,
    Delete = 2,
    Truncate = 3,

    // Transaction boundaries
    Begin = 10,
    Commit = 11,
    Rollback = 12,

    // Snapshot markers (for COPY rows)
    SnapshotRow = 20,
    SnapshotBegin = 21,
    SnapshotEnd = 22,
}

impl OperationType {
    /// Convert from byte value.
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Insert),
            1 => Some(Self::Update),
            2 => Some(Self::Delete),
            3 => Some(Self::Truncate),
            10 => Some(Self::Begin),
            11 => Some(Self::Commit),
            12 => Some(Self::Rollback),
            20 => Some(Self::SnapshotRow),
            21 => Some(Self::SnapshotBegin),
            22 => Some(Self::SnapshotEnd),
            _ => None,
        }
    }
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Insert => write!(f, "INSERT"),
            Self::Update => write!(f, "UPDATE"),
            Self::Delete => write!(f, "DELETE"),
            Self::Truncate => write!(f, "TRUNCATE"),
            Self::Begin => write!(f, "BEGIN"),
            Self::Commit => write!(f, "COMMIT"),
            Self::Rollback => write!(f, "ROLLBACK"),
            Self::SnapshotRow => write!(f, "SNAPSHOT_ROW"),
            Self::SnapshotBegin => write!(f, "SNAPSHOT_BEGIN"),
            Self::SnapshotEnd => write!(f, "SNAPSHOT_END"),
        }
    }
}

/// PostgreSQL replica identity setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum ReplicaIdentity {
    #[default]
    Default = 0, // 'd' - Use primary key
    Nothing = 1, // 'n' - No old row data
    Full = 2,    // 'f' - Full old row
    Index = 3,   // 'i' - Use specific index
}

impl ReplicaIdentity {
    /// Convert from PostgreSQL replica identity character.
    pub fn from_pg_char(c: u8) -> Self {
        match c {
            b'd' => Self::Default,
            b'n' => Self::Nothing,
            b'f' => Self::Full,
            b'i' => Self::Index,
            _ => Self::Default,
        }
    }

    /// Convert to PostgreSQL replica identity character.
    pub fn to_pg_char(self) -> u8 {
        match self {
            Self::Default => b'd',
            Self::Nothing => b'n',
            Self::Full => b'f',
            Self::Index => b'i',
        }
    }
}

/// A single column value.
/// Uses raw PostgreSQL binary format bytes for zero-copy efficiency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnValue {
    /// Type tag for fast dispatch.
    pub type_tag: TypeTag,

    /// PostgreSQL type OID (for arrays, custom types, and disambiguation).
    pub type_oid: u32,

    /// Raw PostgreSQL binary format bytes.
    /// None represents NULL.
    pub data: Option<Vec<u8>>,
}

impl ColumnValue {
    /// Create a NULL column value.
    pub fn null() -> Self {
        Self {
            type_tag: TypeTag::Null,
            type_oid: 0,
            data: None,
        }
    }

    /// Create a column value from raw PostgreSQL binary data.
    pub fn from_pg_binary(type_tag: TypeTag, type_oid: u32, data: Vec<u8>) -> Self {
        Self {
            type_tag,
            type_oid,
            data: Some(data),
        }
    }

    /// Check if this value is NULL.
    pub fn is_null(&self) -> bool {
        self.data.is_none()
    }

    /// Get the raw data bytes.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        self.data.as_deref()
    }

    /// Estimate the size of this value in bytes.
    pub fn size_bytes(&self) -> usize {
        // 1 (tag) + 4 (oid) + data length
        5 + self.data.as_ref().map_or(0, |d| d.len())
    }
}

/// A row of column values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Row {
    pub values: Vec<ColumnValue>,
}

impl Row {
    /// Create a new row from values.
    pub fn new(values: Vec<ColumnValue>) -> Self {
        Self { values }
    }

    /// Create an empty row.
    pub fn empty() -> Self {
        Self { values: Vec::new() }
    }

    /// Get a value by index.
    pub fn get(&self, index: usize) -> Option<&ColumnValue> {
        self.values.get(index)
    }

    /// Estimate the size of this row in bytes.
    pub fn size_bytes(&self) -> usize {
        self.values.iter().map(|v| v.size_bytes()).sum()
    }
}

/// Column metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnMeta {
    /// Column name.
    pub name: String,

    /// PostgreSQL type OID.
    pub type_oid: u32,

    /// Type modifier (e.g., varchar length, numeric precision).
    pub type_modifier: i32,

    /// Whether this column is part of the key.
    pub is_key: bool,
}

/// Relation (table) metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationMeta {
    /// Relation ID (PostgreSQL OID).
    pub rel_id: u32,

    /// Schema name.
    pub schema: String,

    /// Table name.
    pub table: String,

    /// Column definitions.
    pub columns: Vec<ColumnMeta>,

    /// Replica identity setting.
    pub replica_identity: ReplicaIdentity,
}

impl RelationMeta {
    /// Get the fully qualified table name.
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.schema, self.table)
    }

    /// Get column names in order.
    pub fn column_names(&self) -> Vec<&str> {
        self.columns.iter().map(|c| c.name.as_str()).collect()
    }
}

/// A single database event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatabaseEvent {
    /// Unique event identifier (UUID v4).
    pub event_id: Option<String>,

    /// Event timestamp (Unix timestamp in microseconds).
    pub timestamp_us: u64,

    /// Operation type.
    pub operation: OperationType,

    /// Schema name.
    pub schema: String,

    /// Table name.
    pub table: String,

    /// Replication position (PostgreSQL LSN as u64).
    pub position: u64,

    /// Transaction ID.
    pub transaction_id: u64,

    /// New row data (for INSERT, UPDATE, SnapshotRow).
    pub new_row: Option<Row>,

    /// Old row data (for UPDATE with REPLICA IDENTITY FULL, DELETE).
    pub old_row: Option<Row>,

    /// Column names in order.
    pub columns: Vec<String>,

    /// Relation metadata (sent once per table per stream).
    pub relation_meta: Option<RelationMeta>,
}

impl DatabaseEvent {
    /// Get the fully qualified table name.
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.schema, self.table)
    }

    /// Estimate the size of this event in bytes.
    pub fn size_bytes(&self) -> usize {
        let mut size = 64; // Base overhead
        size += self.schema.len();
        size += self.table.len();
        if let Some(ref row) = self.new_row {
            size += row.size_bytes();
        }
        if let Some(ref row) = self.old_row {
            size += row.size_bytes();
        }
        size += self.columns.iter().map(|c| c.len()).sum::<usize>();
        size
    }
}

/// Batch of events for efficient transport.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatabaseEventBatch {
    /// Events in this batch.
    pub events: Vec<DatabaseEvent>,

    /// Source identifier (proxy/connector ID).
    pub source_id: Option<String>,

    /// Batch sequence number (for ordering/deduplication).
    pub batch_seq: u64,

    /// Cached relation metadata for this batch.
    pub relations: Vec<RelationMeta>,
}

impl DatabaseEventBatch {
    /// Create a new empty batch.
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            source_id: None,
            batch_seq: 0,
            relations: Vec::new(),
        }
    }

    /// Create a batch with events.
    pub fn with_events(events: Vec<DatabaseEvent>) -> Self {
        Self {
            events,
            source_id: None,
            batch_seq: 0,
            relations: Vec::new(),
        }
    }

    /// Set the source ID.
    pub fn with_source_id(mut self, source_id: impl Into<String>) -> Self {
        self.source_id = Some(source_id.into());
        self
    }

    /// Set the batch sequence number.
    pub fn with_batch_seq(mut self, seq: u64) -> Self {
        self.batch_seq = seq;
        self
    }

    /// Estimate the size of this batch in bytes.
    pub fn size_bytes(&self) -> usize {
        let mut size = 32; // Base overhead
        size += self.events.iter().map(|e| e.size_bytes()).sum::<usize>();
        size
    }
}

impl Default for DatabaseEventBatch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_tag_roundtrip() {
        for tag in [
            TypeTag::Null,
            TypeTag::Bool,
            TypeTag::Int32,
            TypeTag::Text,
            TypeTag::Uuid,
            TypeTag::Array,
            TypeTag::Custom,
        ] {
            let byte = tag as u8;
            let recovered = TypeTag::from_byte(byte).unwrap();
            assert_eq!(tag, recovered);
        }
    }

    #[test]
    fn test_operation_type_display() {
        assert_eq!(OperationType::Insert.to_string(), "INSERT");
        assert_eq!(OperationType::Update.to_string(), "UPDATE");
        assert_eq!(OperationType::Delete.to_string(), "DELETE");
        assert_eq!(OperationType::Begin.to_string(), "BEGIN");
        assert_eq!(OperationType::Commit.to_string(), "COMMIT");
        assert_eq!(OperationType::SnapshotRow.to_string(), "SNAPSHOT_ROW");
    }

    #[test]
    fn test_replica_identity_pg_char() {
        assert_eq!(ReplicaIdentity::from_pg_char(b'd'), ReplicaIdentity::Default);
        assert_eq!(ReplicaIdentity::from_pg_char(b'f'), ReplicaIdentity::Full);
        assert_eq!(ReplicaIdentity::Full.to_pg_char(), b'f');
    }

    #[test]
    fn test_column_value_null() {
        let null = ColumnValue::null();
        assert!(null.is_null());
        assert_eq!(null.as_bytes(), None);
    }

    #[test]
    fn test_column_value_with_data() {
        let data = vec![0x00, 0x01, 0x02, 0x03];
        let value = ColumnValue::from_pg_binary(TypeTag::Int32, 23, data.clone());
        assert!(!value.is_null());
        assert_eq!(value.as_bytes(), Some(data.as_slice()));
    }

    #[test]
    fn test_row_size_bytes() {
        let row = Row::new(vec![
            ColumnValue::null(),
            ColumnValue::from_pg_binary(TypeTag::Int32, 23, vec![0, 0, 0, 42]),
        ]);
        assert!(row.size_bytes() > 0);
    }

    #[test]
    fn test_database_event_batch() {
        let batch = DatabaseEventBatch::new()
            .with_source_id("test-source")
            .with_batch_seq(42);

        assert_eq!(batch.source_id, Some("test-source".to_string()));
        assert_eq!(batch.batch_seq, 42);
    }
}
