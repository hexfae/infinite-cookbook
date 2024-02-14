use arcstr::ArcStr;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::item::Item;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub fn new(name: &str, emoji: &str, is_new: bool, parents: Vec<(&str, &str)>) -> Self {
        Self {
            name: name.into(),
            emoji: emoji.into(),
            is_new,
            parents: parents
                .into_iter()
                .map(|(f, s)| (f.into(), s.into()))
                .collect(),
        }
    }

    #[must_use]
    pub fn to_item(&self) -> Item {
        Item::new(&self.name, &self.emoji, self.is_new)
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn has_parents(&self) -> bool {
        !self.parents.is_empty()
    }

    #[must_use]
    pub fn parents(&self) -> Vec<(&str, &str)> {
        self.parents
            .iter()
            .map(|(f, s)| (f.as_str(), s.as_str()))
            .collect_vec()
    }
}
