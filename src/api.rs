use crate::utils;
use reqwest::Url;
use serde_json::Value;

#[derive(Default, Debug)]
pub struct Post {
    title: String,
    permalink: String,
    dest_url: String,
    id: String,
    is_video: bool,
    is_gif: bool,
    is_image: bool,
    is_link: bool,
    video_platform: VideoPlatform,
}

#[derive(Debug)]
pub enum VideoPlatform {
    Imgur,
    RedGifs,
    Unknown,
}

impl Default for VideoPlatform {
    fn default() -> Self {
        VideoPlatform::Unknown
    }
}

impl Post {
    pub async fn download(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_image {
            let url = Url::parse(self.dest_url.as_str())?;
            let name = format!("{}.png", &self.id);
            utils::download_file_from_url(Some(url), &name).await?;
        }
        if self.is_gif {
            let url = fetch_download_url(&self.dest_url, &self.video_platform).await?;
            let name = format!("{}.mp4", &self.id);
            utils::download_file_from_url(url, &name).await?;
        }
        Ok(())
    }
}

async fn fetch_posts_from_value(value: Value) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
    let posts = value["data"]["children"].as_array().expect("Not Found");
    let posts_count = posts.len();
    let mut fetched_posts: Vec<Post> = Vec::new();
    for j in 0..posts_count {
        if posts[j]["data"]["is_submitter"].as_bool().is_none() {
            let permalink = posts[j]["data"]["permalink"].to_string().replace('"', "");
            let data = &posts[j]["data"];
            let title = data["title"].to_string().replace('"', "");
            let is_video = data["is_video"].as_bool().expect("Not found");
            let dest_url = data["url_overridden_by_dest"].to_string().replace('"', "");
            let is_gif = data["preview"]["reddit_video_preview"]["is_gif"]
                .as_bool()
                .is_some();
            let is_image = check_if_image(data);
            let is_link = check_if_link(data);
            let video_platform = find_video_platform(&dest_url);
            let id = data["id"].to_string().replace('"', "");
            fetched_posts.push(Post {
                title,
                permalink,
                is_video,
                dest_url,
                is_gif,
                is_image,
                is_link,
                video_platform,
                id,
            });
        }
    }
    Ok(fetched_posts)
}
pub async fn fetch_latest_posts(username: &str) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
    let url = format!("https://www.reddit.com/user/{}.json", username);
    let client = reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:90.0) Gecko/20100101 Firefox/90.0",
        )
        .build()?;
    let resp = client.get(url).send().await?.text().await?;
    Ok(fetch_posts_from_value(serde_json::from_str(&resp)?).await?)
}

pub async fn fetch_posts_from_after(
    username: String,
    after: String,
) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
    let url = format!(
        "https://www.reddit.com/user/{}.json?after={}",
        username, after
    );
    let client = reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:90.0) Gecko/20100101 Firefox/90.0",
        )
        .build()?;
    let resp = client.get(url).send().await?.text().await?;
    Ok(fetch_posts_from_value(serde_json::from_str(&resp)?).await?)
}

#[inline]
fn check_if_image(post: &Value) -> bool {
    let hint = post["post_hint"].to_string().replace('"', "");
    hint == *"image"
}

#[inline]
fn check_if_link(post: &Value) -> bool {
    let hint = post["post_hint"].to_string().replace('"', "");
    hint == *"link"
}

#[inline]
fn get_after_value(post: &Value) -> Option<String> {
    let after = post["data"]["after"].to_string().replace('"', "");
    if after == "null" {
        None
    } else {
        Some(after)
    }
}

fn find_video_platform(link: &str) -> VideoPlatform {
    if link.contains("imgur.com") {
        return VideoPlatform::Imgur;
    }
    if link.contains("redgifs.com") {
        return VideoPlatform::RedGifs;
    }
    VideoPlatform::Unknown
}

pub async fn fetch_download_url(
    dest_url: &str,
    provider: &VideoPlatform,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    match provider {
        VideoPlatform::Imgur => {
            todo!()
        }
        VideoPlatform::RedGifs => {
            let url = dest_url
                .replace(
                    "https://redgifs.com/watch",
                    "https://api.redgifs.com/v1/gfycats",
                )
                .replace('"', "");
            let client = reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:90.0) Gecko/20100101 Firefox/90.0",
        )
        .build()?;
            let resp = client.get(url).send().await?.text().await?;
            let value: Value = serde_json::from_str(&resp)?;
            let final_url = value["gfyItem"]["content_urls"]["mp4"]["url"]
                .to_string()
                .replace('"', "");
            Ok(Some(final_url))
        }
        VideoPlatform::Unknown => todo!(),
    }
}
