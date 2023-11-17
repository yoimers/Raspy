use std::collections::HashMap;

use matchit::{Match, Router};
use url::Url;

use crate::page_process::page_process::PageProcess;

pub struct PageRouter {
  router: HashMap<String, Router<Box<dyn PageProcess>>>,
}

impl PageRouter {
  pub fn new() -> Self {
    Self {
      router: HashMap::new(),
    }
  }
  pub fn insert(mut self, route: &str, value: impl PageProcess + 'static) -> Self {
    let url = Url::parse(route).expect("Failed to parse the URL.");
    let route = self
      .router
      .entry(url.host_str().unwrap().to_string())
      .or_insert(Router::new());

    route
      .insert(url.path(), Box::new(value))
      .expect("Failed to insert the url and value");
    self
  }

  pub fn at<'m, 'p>(
    &'m self,
    path: &'p str,
  ) -> anyhow::Result<Match<'m, 'p, &'m Box<dyn PageProcess>>> {
    let url = Url::parse(path)?;
    let Some(route) = self.router.get(url.host_str().unwrap()) else {
      return Err(matchit::MatchError::NotFound.into())
    };
    let Some(x) = path.find(url.path()) else {
      return Err(matchit::MatchError::NotFound.into());
    };
    if x + url.path().len() > path.len() {
      return Err(matchit::MatchError::NotFound.into());
    }
    let path: &str = &path[x..x + url.path().len()];
    match route.at(path) {
      Ok(e) => Ok(e),
      Err(e) => Err(e.into()),
    }
  }
}

#[cfg(test)]
mod test {
  use std::collections::HashMap;

  use async_trait::async_trait;
  use matchit::Params;
  use scraper::Html;

  use crate::page_process::page_process::{Contents, Metainfo, PageProcess};

  use super::PageRouter;
  struct TestPage {}
  impl PageProcess for TestPage {
    fn contents(&self, html: &Html, params: &Params) -> anyhow::Result<Contents> {
      Ok(Contents(vec![]))
    }
    fn metainfo(&self, html: &Html, params: &Params) -> anyhow::Result<Metainfo> {
      Ok(Metainfo(HashMap::new()))
    }
  }
  struct Test;
  #[async_trait]
  impl PageProcess for Test {
    fn contents(&self, html: &Html, params: &Params) -> anyhow::Result<Contents> {
      Ok(Contents(vec![]))
    }
    fn metainfo(&self, html: &Html, params: &Params) -> anyhow::Result<Metainfo> {
      Ok(Metainfo(HashMap::new()))
    }
  }
  #[test]
  fn router_test() {
    let mut page_router = PageRouter::new()
      .insert("https://www.google.co.jp/", TestPage {})
      .insert("https://www.google.co.jp/:id", TestPage {})
      .insert("https://www.google.co.jp/users/:id", Test)
      .insert("https://www.yahoo.co.jp/shops/:id", Test);
    assert_eq!(
      page_router
        .at("https://www.google.co.jp/users/20")
        .unwrap()
        .params
        .get("id"),
      Some("20")
    );
    assert_eq!(
      page_router
        .at("https://www.google.co.jp/10")
        .unwrap()
        .params
        .get("id"),
      Some("10")
    );
    assert_eq!(
      page_router
        .at("https://www.yahoo.co.jp/shops/10")
        .unwrap()
        .params
        .get("id"),
      Some("10")
    );
    assert!(page_router.at("https://www.yahoo.co.jp/10").is_err(),);
  }
}
