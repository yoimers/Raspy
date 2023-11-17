use std::sync::Arc;

use itertools::Itertools;
use scraper::Selector;
use url::{ParseError, Url};

use crate::router::PageRouter;

use super::next_page::NextPage;

pub struct AddNextUrls {
  router: Arc<PageRouter>,
}

impl AddNextUrls {
  pub fn new(router: Arc<PageRouter>) -> Self {
    Self { router }
  }
}

impl NextPage for AddNextUrls {
  fn next_page(&self, base_url: &str, html: &scraper::Html) -> anyhow::Result<Vec<String>> {
    let base_url = Url::parse(base_url).unwrap();
    let a_selector = Selector::parse("a").unwrap();
    let a_list = html
      .select(&a_selector)
      .into_iter()
      .filter_map(|e| e.value().attr("href"));

    let img_selector = Selector::parse("img").unwrap();
    let img_list = html
      .select(&img_selector)
      .into_iter()
      .filter_map(|e| e.value().attr("src"));

    let urls = a_list
      .chain(img_list)
      .filter_map(|e| match Url::parse(e) {
        Ok(e) => Some(e),
        Err(ParseError::RelativeUrlWithoutBase) => Some(base_url.join(e).unwrap()),
        Err(_) => None,
      })
      .filter(|e| ["https", "http"].contains(&e.scheme()))
      .filter(|v| self.router.at(v.as_str()).is_ok())
      .map(|v| v.to_string())
      .collect_vec();
    Ok(urls)
  }
}

#[cfg(test)]
mod test {
  use std::{collections::HashMap, sync::Arc};

  use async_trait::async_trait;
  use itertools::Itertools;
  use matchit::Params;
  use scraper::Html;
  use url::Url;

  use crate::{
    page_process::{
      next_page::NextPage,
      page_process::{Contents, Metainfo, PageProcess},
    },
    router::PageRouter,
  };

  use super::AddNextUrls;
  #[derive(Clone)]
  struct Test {}
  impl PageProcess for Test {
    fn contents(&self, html: &Html, params: &Params) -> anyhow::Result<Contents> {
      Ok(Contents(vec![]))
    }
    fn metainfo(&self, html: &Html, params: &Params) -> anyhow::Result<Metainfo> {
      Ok(Metainfo(HashMap::new()))
    }
  }
  #[test]
  fn atag_parse_absolute_test() {
    let router = PageRouter::new()
      .insert("https://www.google.co.jp/", Test {})
      .insert("https://www.google.co.jp/:id", Test {})
      .insert("https://www.google.co.jp/users/:id", Test {})
      .insert("https://www.yahoo.co.jp/shops/:id", Test {});

    let next_url_process = AddNextUrls {
      router: Arc::new(router),
    };
    let html = r#"
      <html>
        <body>
          <p>xxx</p>
          <a href="https://www.google.co.jp/" title="struct reqwest::blocking::RequestBuilder"><code>RequestBuilder</code></a>
          <a href="https://www.google.co.jp/123" title="struct reqwest::blocking::RequestBuilder"><code>RequestBuilder</code></a>
          <a href="https://www.google.co.jp/123/123" title="struct reqwest::blocking::RequestBuilder"><code>RequestBuilder</code></a>
          <a href="https://www.google.co.jp/users/123" title="struct reqwest::blocking::RequestBuilder"><code>RequestBuilder</code></a>
          <a href="https://www.yahoo.co.jp/shops/321" title="struct reqwest::blocking::RequestBuilder"><code>RequestBuilder</code></a>
          <a href="https://www.yahoo.co.jp/user/123123" title="struct reqwest::blocking::RequestBuilder"><code>RequestBuilder</code></a>
        </body>
      </html>
    "#;
    let a_list =
      next_url_process.next_page("https://www.google.co.jp/", &Html::parse_document(html));
    assert!(a_list.is_ok());
    let a_list = a_list.unwrap();
    assert_eq!(
      a_list,
      vec![
        "https://www.google.co.jp/",
        "https://www.google.co.jp/123",
        "https://www.google.co.jp/users/123",
        "https://www.yahoo.co.jp/shops/321"
      ]
      .iter()
      .map(|v| v.to_string())
      .collect_vec()
    );
  }

  #[test]
  fn atag_parse_relative_test() {
    let router = PageRouter::new()
      .insert("https://www.google.co.jp/", Test {})
      .insert("https://www.google.co.jp/:id", Test {})
      .insert("https://www.google.co.jp/users/:id", Test {})
      .insert("https://www.yahoo.co.jp/shops/:id", Test {});

    let next_url_process = AddNextUrls {
      router: Arc::new(router),
    };
    let html = r#"
      <html>
        <body>
          <p>xxx</p>
          <a href="/" title="struct reqwest::blocking::RequestBuilder"><code>RequestBuilder</code></a>
          <a href="/123" title="struct reqwest::blocking::RequestBuilder"><code>RequestBuilder</code></a>
          <a href="/123/123" title="struct reqwest::blocking::RequestBuilder"><code>RequestBuilder</code></a>
          <a href="/users/123" title="struct reqwest::blocking::RequestBuilder"><code>RequestBuilder</code></a>
        </body>
      </html>
    "#;
    let a_list =
      next_url_process.next_page("https://www.google.co.jp/", &Html::parse_document(html));
    assert!(a_list.is_ok());
    let a_list = a_list.unwrap();
    assert_eq!(
      a_list,
      vec![
        "https://www.google.co.jp/",
        "https://www.google.co.jp/123",
        "https://www.google.co.jp/users/123",
      ]
      .iter()
      .map(|v| v.to_string())
      .collect_vec()
    );
  }
}
