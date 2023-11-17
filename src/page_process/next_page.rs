use scraper::Html;

pub trait NextPage: Send + Sync + 'static {
  fn next_page(&self, base_url: &str, html: &Html) -> anyhow::Result<Vec<String>>;
}
