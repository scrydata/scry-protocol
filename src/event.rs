use crate::ParamValue;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// Represents a captured SQL query event from the proxy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryEvent {
    /// Unique identifier for this event
    pub event_id: String,

    /// Timestamp when the query was received
    pub timestamp: SystemTime,

    /// The SQL query text (raw if anonymization disabled, else same as normalized_query)
    pub query: String,

    /// Query parameters from Bind message
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<ParamValue>,

    /// True if params couldn't be fully decoded (cache miss)
    #[serde(default)]
    pub params_incomplete: bool,

    /// Normalized query with placeholders (if anonymization enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalized_query: Option<String>,

    /// Fingerprints of literal values (if anonymization enabled)
    /// Enables hot data detection while protecting PII
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_fingerprints: Option<Vec<String>>,

    /// Query execution duration
    pub duration: Duration,

    /// Number of rows affected/returned (if available)
    pub rows: Option<u64>,

    /// Whether the query succeeded
    pub success: bool,

    /// Error message if query failed
    pub error: Option<String>,

    /// Database name
    pub database: String,

    /// Client connection ID
    pub connection_id: String,
}

/// Builder for creating QueryEvent instances
pub struct QueryEventBuilder {
    event_id: String,
    timestamp: SystemTime,
    query: String,
    params: Vec<ParamValue>,
    params_incomplete: bool,
    normalized_query: Option<String>,
    value_fingerprints: Option<Vec<String>>,
    duration: Option<Duration>,
    rows: Option<u64>,
    success: bool,
    error: Option<String>,
    database: String,
    connection_id: String,
}

impl QueryEventBuilder {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            query: query.into(),
            params: Vec::new(),
            params_incomplete: false,
            normalized_query: None,
            value_fingerprints: None,
            duration: None,
            rows: None,
            success: true,
            error: None,
            database: String::from("unknown"),
            connection_id: String::from("unknown"),
        }
    }

    pub fn params(mut self, params: Vec<ParamValue>) -> Self {
        self.params = params;
        self
    }

    pub fn params_incomplete(mut self, incomplete: bool) -> Self {
        self.params_incomplete = incomplete;
        self
    }

    pub fn normalized_query(mut self, normalized_query: impl Into<String>) -> Self {
        self.normalized_query = Some(normalized_query.into());
        self
    }

    pub fn value_fingerprints(mut self, fingerprints: Vec<String>) -> Self {
        self.value_fingerprints = Some(fingerprints);
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn rows(mut self, rows: u64) -> Self {
        self.rows = Some(rows);
        self
    }

    pub fn success(mut self, success: bool) -> Self {
        self.success = success;
        self
    }

    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.database = database.into();
        self
    }

    pub fn connection_id(mut self, connection_id: impl Into<String>) -> Self {
        self.connection_id = connection_id.into();
        self
    }

    pub fn build(self) -> QueryEvent {
        QueryEvent {
            event_id: self.event_id,
            timestamp: self.timestamp,
            query: self.query,
            params: self.params,
            params_incomplete: self.params_incomplete,
            normalized_query: self.normalized_query,
            value_fingerprints: self.value_fingerprints,
            duration: self.duration.unwrap_or(Duration::from_millis(0)),
            rows: self.rows,
            success: self.success,
            error: self.error,
            database: self.database,
            connection_id: self.connection_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ParamValue;

    #[test]
    fn test_query_event_with_params() {
        let event = QueryEventBuilder::new("SELECT * FROM users WHERE id = $1")
            .params(vec![ParamValue::Int32(42)])
            .build();

        assert_eq!(event.params.len(), 1);
        assert!(!event.params_incomplete);
    }

    #[test]
    fn test_query_event_params_incomplete() {
        let event = QueryEventBuilder::new("SELECT * FROM users WHERE id = $1")
            .params(vec![ParamValue::Unknown { oid: 0, data: vec![0x01] }])
            .params_incomplete(true)
            .build();

        assert!(event.params_incomplete);
    }
}
