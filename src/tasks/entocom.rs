/*
Task 1 - Stuff related to transfers from enwiki and commons.
To take over part of FastilyBot's operations, which have ceased since the retirement of its owner.
*/

use futures::stream::{FuturesUnordered, StreamExt};
use mwbot::{
    self, Bot, SaveOptions,
    generators::{EmbeddedIn, Generator},
};
use regex::Regex;

pub async fn main(bot: Bot) {
    let commons = Bot::from_path(std::path::Path::new(".config/commons.toml"))
        .await
        .unwrap();

    now_commons(bot.clone(), commons.clone()).await;
    nominated_for(bot.clone(), commons.clone()).await;
    copy_to(bot.clone(), commons.clone()).await
}

async fn now_commons(bot: Bot, commons: Bot) {
    let mut tasks = FuturesUnordered::new();
    let mut feed = EmbeddedIn::new("Template:Now Commons")
        .namespace([6])
        .generate(&bot);

    let re =
        Regex::new(r"(?im)^\{\{(Now Commons|db-f8|db-commons|NC)(\|([^|}\n]+))?([^}\n]+)?\}\}$")
            .unwrap(); // dont you just love regex? i sure do

    while let Some(item) = feed.recv().await {
        let page = item.unwrap();
        let title = page.title().to_string();
        let wikitext = page.wikitext().await.unwrap();

        let re = re.clone();
        let commons = commons.clone();

        tasks.push(tokio::spawn(async move {
            if !re.is_match(&wikitext) {
                panic!("Unable to find Now Commons template on {title}")
            }

            let destination = "File:".to_string()
                + &re // i have no idea why this works. if it aint broke dont fix it, amirite?
                    .captures(&wikitext)
                    .and_then(|m| m.get(3))
                    .map(|m| m.as_str())
                    .unwrap_or(&title)
                    .replace("File:", "");

            let mirror = commons
                .page(&html_escape::decode_html_entities(&destination))
                .unwrap();
            if !mirror.exists().await.unwrap() {
                println!("{destination} no longer exists")
            } else {
                let full = re.captures(&wikitext).unwrap().get(0).unwrap().as_str();
                let mirrortext = mirror.wikitext().await.unwrap().to_lowercase();

                let (replacement, summary) = if mirrortext.contains("{{delete") {
                    (
                        format!("{{{{Nominated for deletion on Commons|{destination}}}}}"),
                        "Task 1.1) (File is currently nominated for deletion on Commons. Replacing template.) (TRIAL",
                    )
                } else if mirrortext.contains("{{keep local") {
                    (
                        "".to_string(),
                        "Task 1.2) ({{Keep local}} also present. Removing {{Now Commons}}.) (TRIAL",
                    )
                } else {
                    return;
                };

                page.save(
                    wikitext.replace(full, &replacement),
                    &SaveOptions::summary(summary)
                ).await.unwrap();

                println!("{replacement} {summary}")
            }
        }));

        while tasks.len() >= 1000 {
            let _ = tasks.next().await;
        }
    }

    while (tasks.next().await).is_some() {}
}

async fn nominated_for(bot: Bot, commons: Bot) {
    let mut tasks = FuturesUnordered::new();
    let mut feed = EmbeddedIn::new("Template:Nominated for deletion on Commons")
        .namespace([6])
        .generate(&bot);

    let re =
        Regex::new(r"(?im)^\{\{(Nominated for deletion on Commons)(\|([^|}\n]+))?([^}\n]+)?\}\}$")
            .unwrap();

    while let Some(item) = feed.recv().await {
        let page = item.unwrap();
        let title = page.title().to_string();
        let wikitext = page.wikitext().await.unwrap();

        let re = re.clone();
        let commons = commons.clone();

        tasks.push(tokio::spawn(async move {
            if !re.is_match(&wikitext) {
                panic!("Unable to find nominated template on {title}")
            }

            let destination = "File:".to_string()
                + &re // i have no idea why this works. if it aint broke dont fix it, amirite?
                    .captures(&wikitext)
                    .and_then(|m| m.get(3))
                    .map(|m| m.as_str())
                    .unwrap_or(&title)
                    .replace("1=", "")
                    .replace("File:", "");

            let full = re.captures(&wikitext).unwrap().get(0).unwrap().as_str();
            let mirror = commons.page(&html_escape::decode_html_entities(&destination)).unwrap();

            let (replacement, summary) = if !mirror.exists().await.unwrap() {
                (
                    format!("{{{{Deleted on Commons|{destination}}}}}"),
                    "Task 2.1) (File has been deleted on Commons. Replacing template.) (TRIAL",
                )
            } else if {
                let mirrortext = mirror.wikitext().await.unwrap().to_lowercase();
                !(mirrortext.contains("{{delete") || mirrortext.contains("since|"))
            } {
                (
                    "".to_string(),
                    "Task 2.2) (File no longer nominated for deletion on Commons. Removing template.) (TRIAL",
                )
            } else {
                return;
            };

            page.save(
                wikitext.replace(full, &replacement),
                &SaveOptions::summary(summary)
            ).await.unwrap();

            println!("{title} {replacement} {summary}")
        }));

        while tasks.len() >= 1000 {
            let _ = tasks.next().await;
        }
    }

    while (tasks.next().await).is_some() {}
}

async fn copy_to(bot: Bot, commons: Bot) {
    let mut tasks = FuturesUnordered::new();
    let mut feed = EmbeddedIn::new("Template:Copy to Wikimedia Commons")
        .namespace([6])
        .generate(&bot);

    let re =
        Regex::new(r"(?im)\{\{(Copy to Wikimedia Commons)(\|([^|}\n]+))?([^}\n]+)?\}\}").unwrap();

    while let Some(item) = feed.recv().await {
        let page = item.unwrap();
        let title = page.title().to_string();
        let wikitext = page.wikitext().await.unwrap();

        let re = re.clone();
        let commons = commons.clone();

        tasks.push(tokio::spawn(async move {
            if !re.is_match(&wikitext) {
                panic!("Unable to find copy to template on {title}")
            }
            println!("{title}");

            let destination = "File:".to_string()
                + &re // i have no idea why this works. if it aint broke dont fix it, amirite?
                    .captures(&wikitext)
                    .and_then(|m| m.get(3))
                    .map(|m| m.as_str())
                    .unwrap_or(&title)
                    .replace("1=", "")
                    .replace("File:", "");

            let full = re.captures(&wikitext).unwrap().get(0).unwrap().as_str();
            let mirror = commons.page(&html_escape::decode_html_entities(&destination)).unwrap();

            let (replacement, summary) = if mirror.exists().await.unwrap() {
                (
                    format!("{{{{Now Commons|{destination}|date={{{{subst:#time:Y-m-d}}}}|bot=CanonNiBot}}}}"),
                    "Task 3.1) (File already on Commons. Replacing template.) (TRIAL",
                )
            } else if wikitext.to_lowercase().contains("non-free") {
                (
                    "".to_string(),
                    "Task 3.2) (Non-free file illegible for transfer. Removing template.) (TRIAL",
                )
            } else {
                return;
            };

            page.save(
                wikitext.replace(full, &replacement),
                &SaveOptions::summary(summary)
            ).await.unwrap();

            println!("{title} {replacement} {summary}")
        }));

        while tasks.len() >= 1000 {
            let _ = tasks.next().await;
        }
    }

    while (tasks.next().await).is_some() {}
}
