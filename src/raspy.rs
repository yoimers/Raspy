use std::sync::Arc;

use itertools::Itertools;
use tokio::sync::mpsc;

use crate::{
  page_process::{add_next_urls::AddNextUrls, next_page::NextPage, page_process::PageProcess},
  raspy_config::RaspyConfig,
  router::PageRouter,
  worker::Worker,
};

pub struct Raspy {
  router: Arc<PageRouter>,
  config: RaspyConfig,
}

impl Raspy {
  pub fn new(config: RaspyConfig, router: PageRouter) -> Self {
    Self {
      router: Arc::new(router),
      config,
    }
  }

  pub async fn run(&self) {
    let (tx, rx) = mpsc::channel::<usize>(4);
    let tx = Arc::new(tx);
    // let router = Arc::new(self.router);
    let workers = (0..self.config.worker_num)
      .map(|id| Worker::new(id, self.config.urls[id % self.config.url_num()].to_string()))
      .map(|worker| {
        worker.run(
          self.router.clone(),
          AddNextUrls::new(self.router.clone()),
          tx.clone(),
        )
      })
      .collect_vec();
    for w in workers {
      let _ = w.await;
    }
  }
}

#[cfg(test)]
mod test {
  use std::collections::HashMap;

  use super::Raspy;
  use crate::page_process::page_process::Contents;
  use crate::router::PageRouter;
  use crate::{
    page_process::page_process::{Metainfo, PageProcess},
    raspy_config::RaspyConfigBuilder,
  };
  #[derive(Clone)]
  struct Test {}
  impl PageProcess for Test {
    fn contents(&self, html: &scraper::Html, params: &matchit::Params) -> anyhow::Result<Contents> {
      Ok(Contents(vec![]))
    }
    fn metainfo(&self, html: &scraper::Html, params: &matchit::Params) -> anyhow::Result<Metainfo> {
      Ok(Metainfo(HashMap::new()))
    }
  }
  #[tokio::test]
  async fn usecase_test() {
    let raspy_config = RaspyConfigBuilder::builder()
      .worker(4)
      .initial_urls(&[
        "https://www.google.co.jp/",
        "https://www.google.co.jp/:num/",
        "https://www.google.co.jp/user/:id",
      ])
      .get_exts(&[".png", ".jpg", ".jpeg", ".gif", ".mp4"])
      .save_path("./images")
      .build();
    let router = PageRouter::new()
      .insert("https://www.google.co.jp/", Test {})
      .insert("https://www.google.co.jp/:num", Test {})
      .insert("https://www.google.co.jp/user/:id", Test {});
    // let raspy = Raspy::new(raspy_config, router).run().await;
  }
}
