use std::env;

use api::fetch_latest_posts;

mod api;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    for i in fetch_latest_posts(env::args().nth(1).unwrap().as_str()).await? {
        i.download().await?;
    }
    Ok(())
}
