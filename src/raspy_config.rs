use std::collections::HashSet;

use itertools::Itertools;

pub struct RaspyConfigBuilder {
  worker_num: usize,
  save_path: Option<String>,
  urls: Option<Vec<String>>,
  get_exts: Vec<String>,
}

impl RaspyConfigBuilder {
  pub fn builder() -> Self {
    Self {
      worker_num: 1,
      save_path: None,
      urls: None,
      get_exts: [".jpg", ".png", ".gif", ".jpeg"]
        .map(|v| v.to_string())
        .to_vec(),
    }
  }
  pub fn worker(mut self, worker_num: usize) -> Self {
    self.worker_num = worker_num;
    self
  }
  pub fn save_path(mut self, save_path: impl ToString) -> Self {
    self.save_path = Some(save_path.to_string());
    self
  }

  pub fn initial_urls<T: ToString>(mut self, urls: &[T]) -> Self {
    self.urls = Some(urls.into_iter().map(|v| v.to_string()).collect_vec());
    self
  }
  pub fn get_exts<T: ToString>(mut self, get_exts: &[T]) -> Self {
    self.get_exts = get_exts.into_iter().map(|v| v.to_string()).collect_vec();
    self
  }

  pub fn build(self) -> RaspyConfig {
    RaspyConfig {
      worker_num: self.worker_num,
      save_path: self.save_path.expect("Should be input save_path"),
      urls: self.urls.expect("Should be input urls"),
      get_exts: self.get_exts.into_iter().collect(),
    }
  }
}
pub struct RaspyConfig {
  pub worker_num: usize,
  pub save_path: String,
  pub urls: Vec<String>,
  pub get_exts: HashSet<String>,
}

impl RaspyConfig {
  pub fn url_num(&self) -> usize {
    self.urls.len()
  }
}

#[cfg(test)]
mod test {
  use super::RaspyConfigBuilder;

  #[test]
  fn config_test() {
    let config = RaspyConfigBuilder::builder()
      .initial_urls(&["https://www.google.co.jp/"])
      .save_path("./images")
      .build();

    assert_eq!(config.worker_num, 1);
    assert_eq!(config.urls[0], "https://www.google.co.jp/");
    assert_eq!(config.save_path, "./images");
  }
}
