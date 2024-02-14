use derive_new::new;
use serde::Deserialize;
use std::fmt::Display;
use thiserror::Error;

use crate::{finite_item::FiniteItem, item::Item};

#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub enum ResponseSuccess {
    Ok(Response),
    Nothing,
}

#[derive(Debug, Error)]
#[allow(clippy::module_name_repetitions)]
pub enum ResponseFailure {
    #[error("a network error occured")]
    NetworkError(#[from] reqwest::Error),
    #[error("a parsing error occured")]
    ParsingError(#[from] serde_json::Error),
    #[error("a cloudflare error occured")]
    CloudflareError,
    #[error("a timeout occured")]
    Timeout,
    #[error("not allowed was returned")]
    NotAllowed,
}

#[derive(Debug, Deserialize, Clone, new)]
pub struct Response {
    result: String,
    emoji: Option<String>,
    #[serde(rename = "isNew")]
    is_new: bool,
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(emoji) = &self.emoji {
            write!(f, "{} {}", emoji, self.result)
        } else {
            write!(f, "❌ {}", self.result)
        }
    }
}

impl Response {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.result
    }

    #[must_use]
    pub const fn is_new(&self) -> bool {
        self.is_new
    }

    #[must_use]
    pub fn to_finite_item(self) -> FiniteItem {
        if let Some(emoji) = self.emoji {
            FiniteItem::new(&self.result, &emoji, self.is_new, Vec::new())
        } else {
            FiniteItem::new(&self.result, "❓️", self.is_new, vec![])
        }
    }

    #[must_use]
    pub fn to_item_with_parents(self, first: &str, second: &str) -> Item {
        if let Some(emoji) = self.emoji {
            Item::new_with_parents(&self.result, &emoji, self.is_new, first, second)
        } else {
            Item::new_with_parents(&self.result, "❓️", self.is_new, first, second)
        }
    }
}
