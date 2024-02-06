use derive_new::new;
use parking_lot::RwLock;
use serde::Deserialize;
use std::sync::Arc;

use crate::item::Item;

#[derive(Debug, Deserialize, Clone, new)]
pub struct Response {
    result: String,
    emoji: Option<String>,
    #[serde(rename = "isNew")]
    is_new: bool,
}

impl Response {
    #[must_use]
    pub fn to_item(self) -> Arc<RwLock<Item>> {
        let emoji = self.emoji.unwrap_or_else(|| {
            tracing::warn!("combination returned no emoji!");
            "❓️".into()
        });
        Item::new(&self.result, emoji, self.is_new)
    }
}
