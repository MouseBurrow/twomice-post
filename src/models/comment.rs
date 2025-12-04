use chrono::{DateTime, Local, Utc};
use colored::*;
use serde::{Deserialize, Serialize};
use std::fmt;
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Comment {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub session_token: String,
    pub last_used_at: DateTime<Local>,
    pub expires_at: DateTime<Local>,
}
