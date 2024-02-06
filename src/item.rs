use crate::finite_item::FiniteItem;
use parking_lot::RwLock;
use std::sync::Arc;

type Parents = Vec<(Arc<RwLock<Item>>, Arc<RwLock<Item>>)>;

#[derive(Debug, Default)]
pub struct Item {
    name: Arc<String>,
    emoji: Arc<String>,
    is_new: bool,
    parents: Parents,
}

impl std::fmt::Display for Item {
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

impl Item {
    pub fn new(
        name: impl Into<String>,
        emoji: impl Into<String>,
        is_new: bool,
    ) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            name: Arc::new(name.into()),
            emoji: Arc::new(emoji.into()),
            is_new,
            ..Default::default()
        }))
    }

    #[must_use]
    pub fn name(&self) -> String {
        self.name.to_string()
    }

    #[must_use]
    pub const fn is_new(&self) -> bool {
        self.is_new
    }

    #[must_use]
    pub const fn parents(&self) -> &Parents {
        &self.parents
    }

    #[must_use]
    pub fn is_nothing(&self) -> bool {
        &self.name() == "Nothing"
    }

    pub fn push_parents(&mut self, first: Arc<RwLock<Self>>, second: Arc<RwLock<Self>>) {
        self.parents.push((first, second));
    }

    pub fn contains_parents(&self, first: &Arc<RwLock<Self>>, second: &Arc<RwLock<Self>>) -> bool {
        self.parents.iter().any(|(item1, item2)| {
            let item1 = &item1.read().name;
            let item2 = &item2.read().name;
            let first = &first.read().name;
            let second = &second.read().name;
            item1 == first && item2 == second || item1 == second && item2 == first
        })
    }

    #[must_use]
    pub fn to_finite(&self) -> FiniteItem {
        FiniteItem::new(
            self.name.to_string(),
            self.emoji.to_string(),
            self.is_new,
            self.parents
                .iter()
                .map(|(first, second)| (first.read().name(), second.read().name()))
                .collect(),
        )
    }
}
