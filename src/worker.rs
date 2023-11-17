use crate::{
  page_process::{next_page::NextPage, page_process::PageProcess},
  router::PageRouter,
};
use itertools::Itertools;
use reqwest::header::{self, HeaderMap, HeaderValue};
use scraper::Html;
use std::{collections::HashSet, sync::Arc};
use tokio::sync::mpsc::Sender;
use url::{Position, Url};

pub struct Worker {
  pub id: usize,
  init_url: String,
}

impl Worker {
  pub fn new(id: usize, init_url: impl ToString) -> Self {
    Self {
      id,
      init_url: init_url.to_string(),
    }
  }

  pub fn run<T: Send + Sync + 'static>(
    &self,
    router: Arc<PageRouter>,
    next_page: impl NextPage,
    tx: Arc<Sender<T>>,
  ) -> tokio::task::JoinHandle<anyhow::Result<()>> {
    let id = self.id;
    println!("Worker Run !!! id:{id}");
    let mut que = Vec::new();
    que.push(self.init_url.to_string());
    tokio::spawn(async move { Self::_run(id, router, next_page, &mut que).await })
  }

  pub async fn _run(
    id: usize,
    router: Arc<PageRouter>,
    next_page: impl NextPage,
    que: &mut Vec<String>,
  ) -> anyhow::Result<()> {
    let mut headers = HeaderMap::new();
    headers.insert(
      reqwest::header::USER_AGENT,
      HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36"),
    );
    let client = reqwest::Client::builder()
      .default_headers(headers)
      .build()
      .unwrap();
    let mut visited: HashSet<String> = HashSet::new();
    while let Some(url) = que.pop() {
      let text = client.get(&url).send().await?.text().await?;
      let html = &Html::parse_document(&text);
      let base_url = &Url::parse(&url).unwrap()[..Position::BeforePath];
      for url in next_page.next_page(base_url, html)? {
        if !visited.contains(&url) {
          visited.insert(url.clone());
          que.push(url);
        }
      }
      let service = router.at(&url)?;

      let contents = service.value.contents(html, &service.params)?;
      println!("{} {:?}", url, contents.0);
      let metainfo = service.value.metainfo(html, &service.params)?;
    }
    println!("Worker Stop!!! id:{id}");
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::Worker;

  #[test]
  fn worker_test() {
    // let worker = Worker::new(0, "https://www.google.com/");
  }
}
