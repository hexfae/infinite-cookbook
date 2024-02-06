use color_eyre::Result;
use parking_lot::RwLock;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use spinners::{Spinner, Spinners};
use std::{fmt::Display, fs::File, io::Write, sync::Arc};
use zstd::bulk::decompress;

use crate::{
    finite_collection::FiniteCollection, finite_item::FiniteItem, item::Item, response::Response,
};

const URL: &str = "https://neal.fun/api/infinite-craft/pair?first=FIRST&second=SECOND";
const REFERER: &str = "https://neal.fun/infinite-craft/";

#[derive(Debug)]
pub struct Collection {
    items: Vec<Arc<RwLock<Item>>>,
    client: Client,
}

impl Default for Collection {
    fn default() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("Referer", HeaderValue::from_static(REFERER));
        Self {
            items: vec![
                Item::new("Water", "💧", false),
                Item::new("Fire", "🔥", false),
                Item::new("Wind", "🌬️", false),
                Item::new("Earth", "🌍️", false),
            ],
            client: Client::builder()
                .default_headers(headers)
                .build()
                .expect("valid header"),
        }
    }
}

impl Display for Collection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.len();
        let names = self.to_names().join(", ");
        write!(f, "{len} items: {names}")
    }
}

impl Collection {
    #[must_use]
    pub fn from_items(items: Vec<Arc<RwLock<Item>>>) -> Self {
        Self {
            items,
            ..Default::default()
        }
    }

    // TODO: write
    /// # Errors
    pub fn save(&self, path: &str) -> Result<()> {
        let path = if path.is_empty() {
            "collection.ron"
        } else {
            path
        };
        let mut file = File::create(path)?;
        let finite_collection = self.to_finite();
        let encoded = ron::to_string(&finite_collection)?;
        let compressed = zstd::bulk::compress(encoded.as_bytes(), 5)?;
        file.write_all(&compressed)?;
        Ok(())
    }

    // TODO: write
    /// # Errors
    pub fn open(path: &str) -> Result<Self> {
        let path = if path.is_empty() {
            "collection.ron"
        } else {
            path
        };
        // reasonable decompress buffer size?
        let capacity = usize::try_from(std::fs::metadata(path)?.len() * 10)?;
        let decoded = std::fs::read(path)?;
        let decompressed = decompress(&decoded, capacity)?;
        let string = String::from_utf8(decompressed)?;
        let finite_collection: FiniteCollection = ron::from_str(&string)?;
        Ok(finite_collection.to_collection())
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn add_item(&mut self, name: &str, emoji: &str) {
        self.items.push(Item::new(name, emoji, false));
    }

    // TODO: write
    /// # Errors
    /// # Panics
    pub async fn get(&self, url: &str) -> Result<Response> {
        let text = self.client.get(url).send().await?.text().await?;
        let response = serde_json::from_str(&text);
        match response {
            Ok(response) => Ok(response),
            Err(why) => {
                tracing::error!("{why}");
                tracing::error!("failed parsing {text}");
                // return fire instead of panicking for now
                Ok(Response::new("Fire".into(), Some("🔥".into()), false))
                // self.save("collection.ron")?;
                // Err(why.into())
            }
        }
        //         Ok(serde_json::from_str(
        //     &self.client.get(url).send().await?.text().await?,
        // )?)
    }

    // TODO: write
    /// # Errors
    pub async fn combine(
        &self,
        first: Arc<RwLock<Item>>,
        second: Arc<RwLock<Item>>,
    ) -> Result<Arc<RwLock<Item>>> {
        let url = URL
            .replace("FIRST", &first.read().name())
            .replace("SECOND", &second.read().name());
        let response = self.get(&url).await?;
        Ok(response.to_item())
    }

    // TODO: write
    /// # Errors
    pub async fn scan_all(&mut self) -> Result<usize> {
        let list = self.items.clone();
        self.scan(&list).await
    }

    // TODO: write
    /// # Errors
    pub async fn scan(&mut self, list: &[Arc<RwLock<Item>>]) -> Result<usize> {
        let mut index = 0;
        // let list = self.items.clone();

        for first in list {
            for second in list {
                // as to not ddos neal
                if self.already_combined(first, second) {
                    continue;
                }
                if first.read().is_nothing() || second.read().is_nothing() {
                    continue;
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
                let (first, second) = sort_items(first.clone(), second.clone());
                let mut spin = Spinner::new(
                    Spinners::Dots,
                    format!("{} + {} = ...", first.read(), second.read()),
                );
                let result = self.combine(first.clone(), second.clone()).await?;
                let discovery = if let Some(already_found) = self.find(&result) {
                    if !already_found.read().contains_parents(&first, &second) {
                        already_found
                            .write()
                            .push_parents(first.clone(), second.clone());
                    }
                    ""
                } else {
                    result.write().push_parents(first.clone(), second.clone());
                    self.items.push(result.clone());
                    " 🔎"
                };
                let new = if result.read().is_new() { " ✨" } else { "" };
                let message = format!(
                    "{} + {} = {}{}{}",
                    first.read(),
                    second.read(),
                    result.read(),
                    discovery,
                    new,
                );
                spin.stop_and_persist("✓", message);

                index += 1;
            }
        }
        Ok(index)
    }

    pub fn already_combined(&self, first: &Arc<RwLock<Item>>, second: &Arc<RwLock<Item>>) -> bool {
        self.items
            .iter()
            .any(|item| item.read().contains_parents(first, second))
    }

    pub fn find(&self, input: &Arc<RwLock<Item>>) -> Option<Arc<RwLock<Item>>> {
        self.items
            .iter()
            .find(|item| item.read().name() == input.read().name())
            .cloned()
    }

    #[must_use]
    pub fn find_by_name(&self, input: &str) -> Option<Arc<RwLock<Item>>> {
        self.items
            .iter()
            .find(|item| item.read().name() == input)
            .cloned()
    }

    #[must_use]
    pub fn find_by_finite_item(&self, input: &FiniteItem) -> Option<Arc<RwLock<Item>>> {
        self.items
            .iter()
            .find(|item| item.read().name() == input.name())
            .cloned()
    }

    #[must_use]
    pub fn to_finite(&self) -> FiniteCollection {
        FiniteCollection::new(self.to_finite_items())
    }

    #[must_use]
    pub fn to_names(&self) -> Vec<String> {
        self.items
            .iter()
            .map(|item| format!("{}", item.read()))
            .collect()
    }

    #[must_use]
    pub fn to_finite_items(&self) -> Vec<FiniteItem> {
        self.items
            .iter()
            .map(|item| item.read().to_finite())
            .collect()
    }

    #[must_use]
    pub fn tenth(&self) -> Arc<RwLock<Item>> {
        self.items[10].clone()
    }
}

fn sort_items(
    first: Arc<RwLock<Item>>,
    second: Arc<RwLock<Item>>,
) -> (Arc<RwLock<Item>>, Arc<RwLock<Item>>) {
    let first_name = first.read().name();
    let second_name = second.read().name();
    let mut names = [&first_name, &second_name];
    names.sort();
    if *names[0] == first_name {
        (first, second)
    } else {
        (second, first)
    }
}
