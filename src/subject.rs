//! Subjects for organisation of profiles.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Subject {
    pub uuid: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub name: String,
    pub profiles: HashMap<String, Vec<String>>,
    pub description: Option<String>,
}
