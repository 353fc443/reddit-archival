use reqwest::IntoUrl;
use reqwest::Url;
use serde_json::Value;
use std::fs::File;
use std::io::Write;

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
enum VideoPlatform {
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
            download_file_from_url(url, &name).await?;
        }
        if self.is_gif {
            match self.video_platform {
                VideoPlatform::Imgur => {}
                VideoPlatform::RedGifs => {
                    let url = fetch_download_url_from_redgifs(&self.dest_url)
                        .await?
                        .unwrap();
                    let name = format!("{}.mp4", &self.id);
                    download_file_from_url(url, &name).await?;
                }
                VideoPlatform::Unknown => {
                    println!("Cannot download the GIF")
                }
            }
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

pub async fn fetched_posts_from_after(
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

pub async fn download_file_from_url<T>(url: T, name: &str) -> Result<(), Box<dyn std::error::Error>>
where
    T: IntoUrl,
{
    println!("Download started!");
    let mut resp = reqwest::get(url).await?;
    resp.content_length();
    let mut f = File::create(name)?;
    while let Some(chunk) = resp.chunk().await? {
        f.write_all(&chunk[..])?;
    }
    println!("Download completed!");
    Ok(())
}

pub async fn fetch_download_url_from_redgifs(
    dest_url: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
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
