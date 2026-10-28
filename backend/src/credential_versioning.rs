use crate::error::ApiError;
use chrono::{DateTime, Utc};

pub struct VersioningSystem;

impl VersioningSystem {
    pub fn track_version(_credential_id: &str, _version: u32) -> Result<(), ApiError> {
        Ok(())
    }
}
