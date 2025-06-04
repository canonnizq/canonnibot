use mwbot::Bot;

mod tasks;

#[tokio::main]
async fn main() {
    let bot = Bot::from_path(std::path::Path::new(".config/enwiki.toml"))
        .await
        .unwrap();

    tasks::entocom::main(bot).await;
}
