// TODO: remove, temp
#![allow(clippy::cast_precision_loss)]
use arcstr::ArcStr;
use color_eyre::Result;
use dashmap::DashMap;
use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, StatusCode,
};
use spinners::{Spinner, Spinners};
use std::{fmt::Display, fs::File, io::Write};
use tracing::info;
use zstd::bulk::decompress;

use crate::{
    finite_collection::FiniteCollection,
    finite_item::FiniteItem,
    item::Item,
    response::{Response, ResponseFailure, ResponseSuccess},
};

const URL: &str = "https://neal.fun/api/infinite-craft/pair?first=FIRST&second=SECOND";
const REFERER: &str = "https://neal.fun/infinite-craft/";
const COOLDOWN: f64 = 0.3;

#[derive(Debug, Clone)]
pub struct Collection {
    pub items: DashMap<String, Item>,
    client: Client,
}

impl Default for Collection {
    fn default() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("Referer", HeaderValue::from_static(REFERER));
        Self {
            items: DashMap::from_iter([
                ("Water".into(), Item::new("Water", "💧", false)),
                ("Fire".into(), Item::new("Fire", "🔥", false)),
                ("Wind".into(), Item::new("Wind", "🌬️", false)),
                ("Earth".into(), Item::new("Earth", "🌍️", false)),
            ]),
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
    pub fn from_items(items: Vec<Item>) -> Self {
        let items = items
            .into_iter()
            // TODO: not unwrap!!!!!
            .map(|item| (item.as_str().into(), item))
            .collect_vec();

        Self {
            items: DashMap::from_iter(items),
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
        let collection = finite_collection.to_collection();
        Ok(collection)
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
        self.items
            .insert(name.into(), Item::new(name, emoji, false));
    }

    // TODO: write
    /// # Errors
    /// # Panics
    pub async fn get(&self, url: &str) -> Result<ResponseSuccess, ResponseFailure> {
        let response = self.client.get(url).send().await?;
        dbg!(&response.status());
        match response.status() {
            StatusCode::FORBIDDEN => Err(ResponseFailure::NotAllowed),
            StatusCode::TOO_MANY_REQUESTS => Err(ResponseFailure::Timeout),
            StatusCode::OK => {
                let text = response.text().await?;
                let response = serde_json::from_str::<Response>(&text)?;
                if response.name() == "Nothing" {
                    Ok(ResponseSuccess::Nothing)
                } else {
                    Ok(ResponseSuccess::Ok(response))
                }
            }
            _ => todo!(),
        }
    }

    // TODO: write
    /// # Errors
    pub async fn combine(
        &self,
        first: &str,
        second: &str,
    ) -> Result<ResponseSuccess, ResponseFailure> {
        let url = URL.replace("FIRST", first).replace("SECOND", second);
        let response = self.get(&url).await?;
        Ok(response)
    }

    // TODO: write
    /// # Errors
    pub async fn scan(&self) -> Result<usize> {
        let mut index = 0;

        let mut total = Vec::with_capacity(self.items.len().pow(2));

        let time_now = std::time::Instant::now();
        info!(
            "the list will start off with {} combinations!",
            self.items.len().pow(2)
        );

        let total_items = self.items.len().pow(2);
        let mut second_index = 0;
        // TODO: instead of adding everything to a list and then
        // removing items from it, filter items before adding them!!!
        // for first in &self.items {
        //     for second in &self.items {
        //         second_index += 1;
        //         info!("trying {} and {}!", first.clone(), second.clone());
        //         info!("{second_index} out of {total_items}");
        //         if first.name() == "Nothing" || second.name() == "Nothing" {
        //             info!("contained nothing!");
        //             continue;
        //         }
        //         let (first, second) = sort_items(first.name(), second.name());
        //         if self.already_combined(&first, &second) {
        //             info!("already combined!");
        //             continue;
        //         }
        //         info!("success!");
        //         total.push((first.clone(), second.clone()));
        //     }
        // }

        for first in &self.items {
            for second in &self.items {
                total.push((first.clone(), second.clone()));
            }
        }

        info!(
            "gathered all items; took {} seconds!",
            time_now.elapsed().as_secs_f64()
        );

        std::thread::sleep(std::time::Duration::from_secs(3));

        let mut total = total
            .par_iter()
            .map(|(first, second)| {
                // info!("sorting {first} and {second}");
                sort_items(first.name(), second.name())
            })
            .filter(|(first, second)| {
                // info!("checking for combinations for {first} and {second}");
                !self.already_combined(first, second)
            })
            .filter(|(first, second)| {
                // info!("filtering away Nothing for {first} and {second}");
                first != "Nothing" || second != "Nothing"
            })
            .collect::<Vec<(ArcStr, ArcStr)>>();

        total.sort_unstable();
        total.dedup();

        info!(
            "filtering complete! doing {} combinations! eta: {} seconds ({} minutes)",
            total.len(),
            (COOLDOWN + 0.15) * total.len() as f64,
            ((COOLDOWN + 0.15) * total.len() as f64) / 60.0
        );

        for (first, second) in total {
            // as to not ddos neal
            std::thread::sleep(std::time::Duration::from_secs_f64(COOLDOWN));

            let mut spin = Spinner::new(Spinners::Dots, format!("{first} + {second} = ..."));

            let response = self.combine(&first, &second).await;

            let message = match response {
                Err(ResponseFailure::NetworkError(why)) => todo!(),
                Err(ResponseFailure::ParsingError(why)) => todo!(),
                Err(ResponseFailure::CloudflareError) => todo!(),
                Err(ResponseFailure::Timeout) => todo!(),
                Err(ResponseFailure::NotAllowed) => todo!(),
                Ok(ResponseSuccess::Nothing) => format!("{first} + {second} = ❌ Nothing"),
                Ok(ResponseSuccess::Ok(result)) => {
                    let new = if result.is_new() { " ✨" } else { "" };

                    let discovery = self.items.get_mut(result.name()).map_or_else(
                        || {
                            // TODO: not clone
                            let item = result.clone().to_item_with_parents(&first, &second);
                            self.items
                                // TODO: FIX FIX FIX!!!!!!!!
                                .insert(item.as_str().into(), item);
                            " 🔎"
                        },
                        |mut already_found| {
                            if !already_found.contains_parents(&first, &second) {
                                already_found.push_parents(first.clone(), second.clone());
                            }
                            ""
                        },
                    );
                    format!("{first} + {second} = {result}{discovery}{new}")
                }
            };

            spin.stop_and_persist("✓", message);

            if index % 1000 == 0 {
                info!("1000");
                self.save("collection.ron")?;
            }
            index += 1;
        }

        Ok(index)
    }

    #[must_use]
    pub fn already_combined(&self, first: &str, second: &str) -> bool {
        self.items
            .iter()
            .any(|item| item.contains_parents(first, second))
    }

    #[must_use]
    pub fn to_finite(&self) -> FiniteCollection {
        FiniteCollection::new(self.to_finite_items())
    }

    #[must_use]
    pub fn to_names(&self) -> Vec<String> {
        self.items
            .iter()
            .map(|item| format!("{}", item.to_finite()))
            .collect()
    }

    #[must_use]
    pub fn to_finite_items(&self) -> Vec<FiniteItem> {
        self.items
            .clone()
            .into_iter()
            .map(|(_, item)| item.to_finite())
            .collect()
    }
}

fn sort_items(first: ArcStr, second: ArcStr) -> (ArcStr, ArcStr) {
    // TODO: improve
    let mut names = [first.as_str(), second.as_str()];
    names.sort_unstable();
    if *names[0] == first {
        (first, second)
    } else {
        (second, first)
    }
}
