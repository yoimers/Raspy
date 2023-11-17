use anyhow::anyhow;
use async_trait::async_trait;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use tokio::io::{AsyncWrite, AsyncWriteExt};

#[async_trait]
trait DownLoadHelper {
  async fn get_contents<T: AsyncWrite + Unpin + Send + Sync>(
    &self,
    client: &Client,
    url: &str,
    writer: &mut T,
  ) -> anyhow::Result<()>;
}

pub struct ImageDownLoader;

#[async_trait]
impl DownLoadHelper for ImageDownLoader {
  async fn get_contents<T: AsyncWrite + Unpin + Send + Sync>(
    &self,
    client: &Client,
    url: &str,
    writer: &mut T,
  ) -> anyhow::Result<()> {
    let res = client
      .get(url)
      .send()
      .await
      .or(Err(anyhow!("Failed to GET from {}", url)))?;
    let total_size = res
      .content_length()
      .ok_or(anyhow!("Failed to get content length from {}", url))?;
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
  .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap()
  .progress_chars("#>-"));
    pb.set_message(format!("Downloading {}", url));
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();
    while let Some(item) = stream.next().await {
      let chunk = item.or(Err(anyhow!("Error while downloading file")))?;
      writer
        .write_all(&chunk)
        .await
        .or(Err(anyhow!("Error while writing to file")))?;
      let new = total_size.min(downloaded + chunk.len() as u64);
      downloaded = new;
      pb.set_position(new);
    }
    pb.finish_with_message(format!("Downloaded {}", url));
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use reqwest::header::{HeaderMap, HeaderValue};
  use tokio::fs::{self, File};

  use super::{DownLoadHelper, ImageDownLoader};

  #[tokio::test]
  async fn download_test() {
    let downloader = ImageDownLoader;
    let mut headers = HeaderMap::new();
    headers.insert(
      reqwest::header::USER_AGENT,
      HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36"),
    );

    let client = reqwest::Client::builder()
      .default_headers(headers)
      .build()
      .unwrap();
    let mut file = File::create("./test.jpg").await.unwrap();
    let result = downloader
      .get_contents(&client, "https://www.google.co.jp/", &mut file)
      .await;
    fs::remove_file("./test.jpg").await.unwrap();
    assert!(result.is_err());
    assert_eq!(
      result.unwrap_err().to_string(),
      "Failed to get content length from https://www.google.co.jp/".to_string()
    );
  }
}
