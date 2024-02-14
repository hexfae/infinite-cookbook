use arcstr::ArcStr;
use derive_new::new;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{collection::Collection, finite_item::FiniteItem, item::Item};

#[derive(Debug, Serialize, Deserialize, new, Clone)]
pub struct FiniteCollection {
    items: Vec<FiniteItem>,
}

impl std::fmt::Display for FiniteCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_names().join(", "))
    }
}

impl FiniteCollection {
    // pub fn to_items(&self) -> Vec<Item> {
    //     self.items.iter().map(FiniteItem::to_item).collect_vec()
    // }

    // TODO: write
    /// # Panics
    #[must_use]
    pub fn to_collection(&self) -> Collection {
        let collection = Collection::default();
        for finite_item in &self.items {
            let mut item = finite_item.to_item();
            // insert here needed because an item
            // could have itself as a parent maybe
            collection
                .items
                .insert(finite_item.name().into(), item.clone());
            for (first, second) in finite_item.parents() {
                let first = collection
                    .items
                    .get(first)
                    .map_or_else(|| ArcStr::from(first), |item| item.clone().name());
                let second = collection
                    .items
                    .get(second)
                    .map_or_else(|| ArcStr::from(second), |item| item.clone().name());
                item.push_parents(first, second);
                // insert again because, if an item has itself as its parent
                collection
                    .items
                    .insert(finite_item.name().into(), item.clone());
            }
        }
        collection
    }

    #[must_use]
    pub fn to_names(&self) -> Vec<String> {
        self.items.iter().map(|item| format!("{item}")).collect()
    }
}
