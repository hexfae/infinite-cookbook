use std::sync::Arc;

use derive_new::new;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::{collection::Collection, finite_item::FiniteItem, item::Item};

#[derive(Serialize, Deserialize, new)]
pub struct FiniteCollection {
    items: Vec<FiniteItem>,
}

impl std::fmt::Display for FiniteCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_names().join(", "))
    }
}

impl FiniteCollection {
    pub fn to_items(&self) -> Vec<Arc<RwLock<Item>>> {
        self.items
            .iter()
            .map(FiniteItem::to_item)
            .collect::<Vec<Arc<RwLock<Item>>>>()
    }

    // TODO: write
    /// # Panics
    #[must_use]
    pub fn to_collection(&self) -> Collection {
        let collection = Collection::from_items(self.to_items());
        for finite_item in &self.items {
            for (first_name, second_name) in finite_item.parents() {
                let first = collection
                    .find_by_name(first_name)
                    .expect("valid saved file");
                let second = collection
                    .find_by_name(second_name)
                    .expect("valid saved file");
                let item = collection
                    .find_by_finite_item(finite_item)
                    .expect("valid saved file");
                item.write().push_parents(first.clone(), second.clone());
            }
        }
        collection
    }

    #[must_use]
    pub fn to_names(&self) -> Vec<String> {
        self.items.iter().map(|item| format!("{item}")).collect()
    }
}
