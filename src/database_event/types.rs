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

    // Special marker for unchanged TOAST values in UPDATE operations.
    // When a column has a large TOAST value that wasn't modified, PostgreSQL
    // sends this marker instead of the value. The target should exclude these
    // columns from the UPDATE SET clause.
    ToastUnchanged = 126,

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
            126 => Some(Self::ToastUnchanged),
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

    // Control directives (for coordinating producer/receiver behavior)
    SequenceSync = 30,
    DisableForeignKeys = 31,
    EnableForeignKeys = 32,

    // DDL operations
    Ddl = 33,

    // DDL phase completed (control event from scry-backfill)
    DdlComplete = 34,

    // Backfill phase completed (control event from scry-backfill)
    // Contains table statistics for verification
    BackfillComplete = 35,

    // Backfill phase starting (control event from scry-backfill)
    // Contains estimated table/row counts for progress tracking
    BackfillStart = 36,
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
            30 => Some(Self::SequenceSync),
            31 => Some(Self::DisableForeignKeys),
            32 => Some(Self::EnableForeignKeys),
            33 => Some(Self::Ddl),
            34 => Some(Self::DdlComplete),
            35 => Some(Self::BackfillComplete),
            36 => Some(Self::BackfillStart),
            _ => None,
        }
    }

    /// Check if this is a control directive (not a data operation).
    pub fn is_control_directive(&self) -> bool {
        matches!(
            self,
            Self::SequenceSync | Self::DisableForeignKeys | Self::EnableForeignKeys
                | Self::BackfillComplete | Self::BackfillStart
        )
    }

    /// Check if this is a DDL operation.
    pub fn is_ddl(&self) -> bool {
        matches!(self, Self::Ddl)
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
            Self::SequenceSync => write!(f, "SEQUENCE_SYNC"),
            Self::DisableForeignKeys => write!(f, "DISABLE_FOREIGN_KEYS"),
            Self::EnableForeignKeys => write!(f, "ENABLE_FOREIGN_KEYS"),
            Self::Ddl => write!(f, "DDL"),
            Self::DdlComplete => write!(f, "DDL_COMPLETE"),
            Self::BackfillComplete => write!(f, "BACKFILL_COMPLETE"),
            Self::BackfillStart => write!(f, "BACKFILL_START"),
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

/// A PostgreSQL sequence value for synchronization.
///
/// Used to transmit current sequence state from source to target
/// so that sequences can be synchronized after snapshot completion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SequenceValue {
    /// Schema name containing the sequence.
    pub schema: String,

    /// Sequence name.
    pub name: String,

    /// Current last_value from pg_sequences.
    pub last_value: i64,

    /// Whether setval's is_called parameter should be true.
    /// If true: next nextval() returns last_value + increment_by
    /// If false: next nextval() returns last_value
    pub is_called: bool,

    /// Increment value for this sequence.
    pub increment_by: i64,

    /// Minimum value for this sequence.
    pub min_value: i64,

    /// Maximum value for this sequence.
    pub max_value: i64,
}

impl SequenceValue {
    /// Create a new sequence value.
    pub fn new(
        schema: impl Into<String>,
        name: impl Into<String>,
        last_value: i64,
    ) -> Self {
        Self {
            schema: schema.into(),
            name: name.into(),
            last_value,
            is_called: true,
            increment_by: 1,
            min_value: 1,
            max_value: i64::MAX,
        }
    }

    /// Get the fully qualified sequence name.
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.schema, self.name)
    }
}

/// Control directive for coordinating producer/receiver behavior.
///
/// Control directives are sent as special batches to signal the receiver
/// to perform administrative operations like disabling/enabling constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ControlDirective {
    /// Disable foreign key constraints on the target.
    /// The receiver should:
    /// 1. Query all FK constraints
    /// 2. Store definitions in _scry_admin.foreign_keys
    /// 3. Drop all FK constraints
    DisableForeignKeys,

    /// Enable foreign key constraints on the target.
    /// The receiver should:
    /// 1. Read FK definitions from _scry_admin.foreign_keys
    /// 2. Recreate all FK constraints
    /// 3. Clean up the admin table
    EnableForeignKeys,

    /// Synchronize sequence values.
    /// The batch will contain sequence_values with the current state.
    SyncSequences,
}

impl std::fmt::Display for ControlDirective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DisableForeignKeys => write!(f, "DISABLE_FOREIGN_KEYS"),
            Self::EnableForeignKeys => write!(f, "ENABLE_FOREIGN_KEYS"),
            Self::SyncSequences => write!(f, "SYNC_SEQUENCES"),
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

    /// Create a marker for an unchanged TOAST value.
    /// Used in UPDATE operations when a large column value was not modified.
    /// The type_oid is preserved so the target knows the column's type.
    pub fn unchanged(type_oid: u32) -> Self {
        Self {
            type_tag: TypeTag::ToastUnchanged,
            type_oid,
            data: None,
        }
    }

    /// Check if this value is NULL.
    pub fn is_null(&self) -> bool {
        self.data.is_none() && self.type_tag == TypeTag::Null
    }

    /// Check if this value is an unchanged TOAST marker.
    pub fn is_unchanged(&self) -> bool {
        self.type_tag == TypeTag::ToastUnchanged
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

/// Metadata about a backfill operation, sent with `BackfillStart` events.
///
/// Contains estimated table and row counts for progress tracking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackfillMetadata {
    /// Number of tables being backfilled.
    pub table_count: u32,

    /// Estimated total rows across all tables (from pg_class.reltuples).
    pub estimated_total_rows: u64,
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

    /// DDL SQL statement (when operation == Ddl).
    /// Contains the full CREATE/ALTER/DROP statement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ddl_sql: Option<String>,

    /// DDL object type for filtering/logging.
    /// e.g., "extension", "table", "index", "constraint", "function", "trigger"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ddl_object_type: Option<String>,
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

    /// Control directive for this batch (if a control batch).
    /// When present, this batch signals the receiver to perform
    /// an administrative operation rather than applying data changes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub control_directive: Option<ControlDirective>,

    /// Sequence values for synchronization.
    /// Present when control_directive is SyncSequences.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequence_values: Option<Vec<SequenceValue>>,

    /// Backfill metadata for progress tracking.
    /// Present when the batch contains a BackfillStart event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backfill_metadata: Option<BackfillMetadata>,
}

impl DatabaseEventBatch {
    /// Create a new empty batch.
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            source_id: None,
            batch_seq: 0,
            relations: Vec::new(),
            control_directive: None,
            sequence_values: None,
            backfill_metadata: None,
        }
    }

    /// Create a batch with events.
    pub fn with_events(events: Vec<DatabaseEvent>) -> Self {
        Self {
            events,
            source_id: None,
            batch_seq: 0,
            relations: Vec::new(),
            control_directive: None,
            sequence_values: None,
            backfill_metadata: None,
        }
    }

    /// Create a control batch that signals an administrative operation.
    pub fn control(directive: ControlDirective) -> Self {
        Self {
            events: Vec::new(),
            source_id: None,
            batch_seq: 0,
            relations: Vec::new(),
            control_directive: Some(directive),
            sequence_values: None,
            backfill_metadata: None,
        }
    }

    /// Create a sequence sync control batch with sequence values.
    pub fn sequence_sync(sequences: Vec<SequenceValue>) -> Self {
        Self {
            events: Vec::new(),
            source_id: None,
            batch_seq: 0,
            relations: Vec::new(),
            control_directive: Some(ControlDirective::SyncSequences),
            sequence_values: Some(sequences),
            backfill_metadata: None,
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

    /// Set the control directive.
    pub fn with_control_directive(mut self, directive: ControlDirective) -> Self {
        self.control_directive = Some(directive);
        self
    }

    /// Set sequence values.
    pub fn with_sequence_values(mut self, sequences: Vec<SequenceValue>) -> Self {
        self.sequence_values = Some(sequences);
        self
    }

    /// Set backfill metadata for progress tracking.
    pub fn with_backfill_metadata(mut self, metadata: BackfillMetadata) -> Self {
        self.backfill_metadata = Some(metadata);
        self
    }

    /// Check if this is a control batch (no data events, just a directive).
    pub fn is_control_batch(&self) -> bool {
        self.control_directive.is_some()
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
            TypeTag::ToastUnchanged,
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
        assert!(!null.is_unchanged());
        assert_eq!(null.as_bytes(), None);
    }

    #[test]
    fn test_column_value_unchanged() {
        let unchanged = ColumnValue::unchanged(25); // TEXT type OID
        assert!(!unchanged.is_null());
        assert!(unchanged.is_unchanged());
        assert_eq!(unchanged.type_oid, 25);
        assert_eq!(unchanged.as_bytes(), None);
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
        assert!(!batch.is_control_batch());
    }

    #[test]
    fn test_operation_type_control_directives() {
        // Test control directive operation types
        assert_eq!(OperationType::SequenceSync.to_string(), "SEQUENCE_SYNC");
        assert_eq!(OperationType::DisableForeignKeys.to_string(), "DISABLE_FOREIGN_KEYS");
        assert_eq!(OperationType::EnableForeignKeys.to_string(), "ENABLE_FOREIGN_KEYS");

        // Test from_byte roundtrip
        assert_eq!(OperationType::from_byte(30), Some(OperationType::SequenceSync));
        assert_eq!(OperationType::from_byte(31), Some(OperationType::DisableForeignKeys));
        assert_eq!(OperationType::from_byte(32), Some(OperationType::EnableForeignKeys));

        // Test is_control_directive
        assert!(OperationType::SequenceSync.is_control_directive());
        assert!(OperationType::DisableForeignKeys.is_control_directive());
        assert!(OperationType::EnableForeignKeys.is_control_directive());
        assert!(!OperationType::Insert.is_control_directive());
        assert!(!OperationType::SnapshotRow.is_control_directive());
    }

    #[test]
    fn test_sequence_value() {
        let seq = SequenceValue::new("public", "users_id_seq", 100);
        assert_eq!(seq.schema, "public");
        assert_eq!(seq.name, "users_id_seq");
        assert_eq!(seq.last_value, 100);
        assert!(seq.is_called);
        assert_eq!(seq.increment_by, 1);
        assert_eq!(seq.qualified_name(), "public.users_id_seq");
    }

    #[test]
    fn test_control_directive_display() {
        assert_eq!(ControlDirective::DisableForeignKeys.to_string(), "DISABLE_FOREIGN_KEYS");
        assert_eq!(ControlDirective::EnableForeignKeys.to_string(), "ENABLE_FOREIGN_KEYS");
        assert_eq!(ControlDirective::SyncSequences.to_string(), "SYNC_SEQUENCES");
    }

    #[test]
    fn test_control_batch() {
        let batch = DatabaseEventBatch::control(ControlDirective::DisableForeignKeys)
            .with_source_id("backfill-001")
            .with_batch_seq(1);

        assert!(batch.is_control_batch());
        assert_eq!(batch.control_directive, Some(ControlDirective::DisableForeignKeys));
        assert!(batch.events.is_empty());
        assert!(batch.sequence_values.is_none());
    }

    #[test]
    fn test_sequence_sync_batch() {
        let sequences = vec![
            SequenceValue::new("public", "users_id_seq", 100),
            SequenceValue::new("public", "orders_id_seq", 500),
        ];

        let batch = DatabaseEventBatch::sequence_sync(sequences)
            .with_source_id("backfill-001")
            .with_batch_seq(99);

        assert!(batch.is_control_batch());
        assert_eq!(batch.control_directive, Some(ControlDirective::SyncSequences));
        assert!(batch.events.is_empty());

        let seq_values = batch.sequence_values.as_ref().unwrap();
        assert_eq!(seq_values.len(), 2);
        assert_eq!(seq_values[0].qualified_name(), "public.users_id_seq");
        assert_eq!(seq_values[1].last_value, 500);
    }
}
