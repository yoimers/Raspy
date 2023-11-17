use std::{collections::HashMap, sync::Arc};

use matchit::Params;
use scraper::Html;

pub trait PageProcess: Send + Sync {
  fn contents(&self, html: &Html, params: &Params) -> anyhow::Result<Contents>;
  fn metainfo(&self, html: &Html, params: &Params) -> anyhow::Result<Metainfo>;
}

pub struct Contents(pub Vec<String>);

pub struct Metainfo(pub HashMap<String, Vec<String>>);
