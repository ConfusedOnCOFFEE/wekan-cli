#[cfg(not(test))]
use super::config::FileWriter;
use crate::config::{ConfigRequester, UserConfig};
use async_trait::async_trait;
#[cfg(not(test))]
use chrono::prelude::*;
#[cfg(not(test))]
use log::{info, trace};
use serde::{Deserialize, Serialize};
use wekan_common::artifact::common::StoreTrait;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Entry<T> {
    pub age: String,
    pub parent: String,
    pub payload: T,
}

#[cfg(not(test))]
#[async_trait]
pub trait Store: ConfigRequester<UserConfig> {
    async fn write_into_context<'de, T: StoreTrait + Deserialize<'de>>(
        &self,
        partial_context: T,
        id: &str,
    ) {
        let entry = {
            Entry {
                age: Utc::now().to_string(),
                parent: id.to_string(),
                payload: partial_context,
            }
        };
        trace!("Raw full entry: {:?}", entry);
        let mut path = entry.payload.get_type().to_string().to_owned();
        if entry.parent.to_string().is_empty() {
            path.push('s');
        } else {
            path.push('_');
            path.push_str(&entry.parent);
        }
        info!("Write to file: {}", path);
        self.get_config().write(path.to_string(), entry).await;
    }
}

#[cfg(test)]
#[async_trait]
pub trait Store: ConfigRequester<UserConfig> {
    async fn write_into_context<'de, T: StoreTrait + Deserialize<'de>>(
        &self,
        _partial_context: T,
        _id: &str,
    ) {
    }
}
