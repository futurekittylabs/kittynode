use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogsQuery {
    pub tail: Option<usize>,
}
