use serde_json::Value;

#[derive(Default, Debug)]
pub struct Post {
    title: String,
    permalink: String,
    dest_url: String,
    is_video: bool,
    is_gif: bool,
    is_image: bool,
}

impl Post {
    pub async fn display(&self) {
        println!("Title: {}", &self.title);
        println!("Url: {}", &self.permalink);
        println!("Is Video: {}", &self.is_video);
        println!("Destination Url: {}", &self.dest_url);
        println!("Is Gif: {}", &self.is_gif);
        println!("Is Image: {}", &self.is_image);
    }
}

pub async fn fetch_all_posts(username: &str) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
    let url = format!("https://www.reddit.com/user/{}.json", username);
    let client = reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:90.0) Gecko/20100101 Firefox/90.0",
        )
        .build()?;
    let resp = client.get(url).send().await?.text().await?;
    let repo: Value = serde_json::from_str(&resp)?;
    let posts = repo["data"]["children"].as_array().expect("Not Found");
    let posts_count = posts.len();
    let mut fetched_posts: Vec<Post> = Vec::new();
    for j in 0..posts_count {
        if posts[j]["data"]["is_submitter"].as_bool().is_none() {
            let permalink = format!(
                "https://www.reddit.com{}",
                posts[j]["data"]["permalink"].to_string().replace('"', "")
            );
            let title = posts[j]["data"]["title"].to_string();
            let is_video = posts[j]["data"]["is_video"].as_bool().expect("Not found");
            let dest_url = posts[j]["data"]["url_overridden_by_dest"].to_string();
            let is_gif = posts[j]["data"]["preview"]["reddit_video_preview"]["is_gif"]
                .as_bool()
                .is_some();
            let is_image = check_if_image(posts[j]["data"]["post_hint"].to_string());
            fetched_posts.push(Post {
                title,
                permalink,
                is_video,
                dest_url,
                is_gif,
                is_image,
            });
        }
    }
    Ok(fetched_posts)
}

#[inline]
fn check_if_image(hint: String) -> bool {
    let hint = hint.replace('"', "");
    if hint == "image".to_string() {
        true
    } else {
        false
    }
}
