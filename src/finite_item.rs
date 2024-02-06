use std::sync::Arc;

use derive_new::new;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::item::Item;

#[derive(Serialize, Deserialize, new, Clone)]
pub struct FiniteItem {
    name: String,
    emoji: String,
    is_new: bool,
    parents: Vec<(String, String)>,
}

impl std::fmt::Display for FiniteItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}{}",
            self.emoji,
            self.name,
            if self.is_new { " ✨" } else { "" }
        )
    }
}

impl FiniteItem {
    #[must_use]
    pub fn to_item(&self) -> Arc<RwLock<Item>> {
        Item::new(&self.name, &self.emoji, self.is_new)
    }

    #[must_use]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[must_use]
    pub fn has_parents(&self) -> bool {
        !self.parents.is_empty()
    }

    #[must_use]
    pub const fn parents(&self) -> &Vec<(String, String)> {
        &self.parents
    }
}
