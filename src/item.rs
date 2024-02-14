use crate::finite_item::FiniteItem;
use arcstr::ArcStr;
use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    name: ArcStr,
    emoji: ArcStr,
    is_new: bool,
    parents: Vec<(ArcStr, ArcStr)>,
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
    #[must_use]
    pub fn new(name: &str, emoji: &str, is_new: bool) -> Self {
        Self {
            name: ArcStr::from(name),
            emoji: ArcStr::from(emoji),
            is_new,
            parents: vec![],
        }
    }

    #[must_use]
    pub fn new_with_parents(
        name: &str,
        emoji: &str,
        is_new: bool,
        first: &str,
        second: &str,
    ) -> Self {
        Self {
            name: ArcStr::from(name),
            emoji: ArcStr::from(emoji),
            is_new,
            parents: vec![(first.into(), second.into())],
        }
    }

    #[must_use]
    pub fn name(&self) -> ArcStr {
        self.name.clone()
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn is_new(&self) -> bool {
        self.is_new
    }

    #[must_use]
    pub fn parents(&self) -> &[(ArcStr, ArcStr)] {
        &self.parents
    }

    #[must_use]
    pub fn is_nothing(&self) -> bool {
        self.name() == "Nothing"
    }

    pub fn push_parents(&mut self, first: ArcStr, second: ArcStr) {
        self.parents.push((first, second));
    }

    #[must_use]
    pub fn contains_parents(&self, first: &str, second: &str) -> bool {
        self.parents.iter().any(|(item1, item2)| {
            item1 == first && item2 == second || item1 == second && item2 == first
        })
    }

    #[must_use]
    pub fn to_finite(&self) -> FiniteItem {
        FiniteItem::new(
            &self.name,
            &self.emoji,
            self.is_new,
            self.parents
                .iter()
                .map(|(f, s)| (f.as_str(), s.as_str()))
                .collect_vec(),
        )
    }
}
