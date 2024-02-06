use collection::Collection;
use color_eyre::Result;
use inquire::{MultiSelect, Select, Text};
use itertools::Itertools;
use tracing::info;

pub mod collection;
pub mod finite_collection;
pub mod finite_item;
pub mod item;
pub mod response;

const INFO: &str = "OVERVIEW\nthe ♾️📕 infinite cookbook is a helper program for ♾️🛠️ infinite craft by neal agarwal. the original game can be found and played at https://neal.fun/infinite-craft/\n\nVOCABULARY\n\nITEM: a named item/concept/person/etc. and its accompanying emoji as per the website, e.g. 🔥 Fire or 👊 Goku\n\nCOMBINE: to combine is to... combine two items through its recipe\n\nRESEARCH: to research is to discover an item through combining the items of one of its recipes, e.g. to research 💨 Steam through 🔥 Fire and 💧 Water\n\nRECIPE: two items used to research a given item, e.g. 🔥 Fire and 💧 Water is a recipe for 💨 Steam\n\nFEATURES\ncurrently, it supports scanning (iterating over every researched item and combining them), adding custom items, scanning just a list of items, viewing this help message... and displaying this message :)";

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    let mut collection = Collection::open("collection.ron").unwrap_or_default();
    let choices = vec!["scan", "craft", "add", "help", "view", "quit"];

    loop {
        match Select::new("what do?", choices.clone()).prompt()? {
            "scan" => loop {
                let now = std::time::Instant::now();
                let found = collection.scan_all().await?;
                tracing::info!("found {found} new items!");
                tracing::info!(
                    "scan finished in {} seconds ({} minutes)",
                    now.elapsed().as_secs(),
                    now.elapsed().as_secs() / 60
                );
                collection.save("collection.ron")?;
            },
            "craft" => craft(&mut collection).await?,
            "add" => add(&mut collection)?,
            "help" => println!("{INFO}"),
            "view" => view(&collection)?,
            _ => break,
        };
    }

    Ok(())
}

async fn craft(collection: &mut Collection) -> Result<()> {
    let finite_items = collection.to_finite_items();
    let selection = MultiSelect::new("first", finite_items)
        .with_page_size(20)
        .prompt()?;
    let items = selection
        .iter()
        .filter_map(|item| collection.find_by_finite_item(item))
        .collect_vec();
    let now = std::time::Instant::now();
    let found = collection.scan(&items).await?;
    info!("found {found} new items!");
    info!(
        "scan finished in {} seconds ({} minutes)",
        now.elapsed().as_secs(),
        now.elapsed().as_secs() / 60
    );
    collection.save("collection.ron")?;
    Ok(())
}

fn view(collection: &Collection) -> Result<()> {
    let items = collection.to_finite_items();
    let _ = MultiSelect::new("view", items)
        .with_page_size(20)
        .with_vim_mode(true)
        .prompt_skippable()?;
    Ok(())
}

fn add(collection: &mut Collection) -> Result<()> {
    let name = Text::new("name?").prompt()?;
    let emoji = Text::new("emoji?")
        .with_help_message("❓️")
        .prompt_skippable()?
        .unwrap_or_else(|| "❓️".into());
    collection.add_item(&name, &emoji);
    Ok(())
}
