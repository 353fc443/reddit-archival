use std::env;

use api::fetch_all_posts;

mod api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    for i in fetch_all_posts(env::args().nth(1).unwrap().as_str()).await? {
        i.display().await;
    }
    Ok(())
}
