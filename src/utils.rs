use reqwest::IntoUrl;
use std::fs::File;
use std::io::Write;

pub async fn download_file_from_url<T>(
    url: Option<T>,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: IntoUrl,
{
    if url.is_none() {
        println!("Cannot Download!");
    } else {
        println!("Download started!");
        let mut resp = reqwest::get(url.unwrap()).await?;
        resp.content_length();
        let mut f = File::create(name)?;
        while let Some(chunk) = resp.chunk().await? {
            f.write_all(&chunk[..])?;
        }
        println!("Download completed!");
    }
    Ok(())
}
