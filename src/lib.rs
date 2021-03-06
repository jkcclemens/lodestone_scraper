#![feature(crate_visibility_modifier)]

#[macro_use] extern crate failure;

use lazy_static::lazy_static;

use lodestone_parser::models::{
  character::Character,
  free_company::FreeCompany,
};

use reqwest::{Client, StatusCode};

use url::Url;

use std::str::FromStr;

pub mod builder;
pub mod error;
pub mod models;
crate mod util;

use crate::error::*;

#[derive(Debug)]
pub struct LodestoneScraper {
  client: Client,
}

impl Default for LodestoneScraper {
  fn default() -> Self {
    let client = Client::new();
    LodestoneScraper { client }
  }
}

lazy_static! {
  static ref LODESTONE_URL: Url = Url::from_str("https://na.finalfantasyxiv.com/lodestone/").unwrap();
}

impl LodestoneScraper {
  fn route(s: &str) -> Result<Url> {
    LODESTONE_URL.join(s).map_err(Error::Url)
  }

  crate async fn text(&self, url: Url) -> Result<String> {
    let res = self.client
      .get(url)
      .send()
      .await
      .map_err(Error::Net)?;
    match res.status() {
      StatusCode::OK => {},
      StatusCode::NOT_FOUND => return Err(Error::NotFound),
      x => return Err(Error::UnexpectedResponse(x)),
    }
    res
      .text()
      .await
      .map_err(Error::Net)
  }

  pub async fn character(&self, id: u64) -> Result<Character> {
    let url = LodestoneScraper::route(&format!("character/{}", id))?;
    let text = self.text(url).await?;
    lodestone_parser::parse_character(id, &text).map_err(Error::Parse)
  }

  pub fn character_search(&self) -> builder::CharacterSearchBuilder {
    builder::CharacterSearchBuilder::new(self)
  }

  pub async fn free_company(&self, id: u64) -> Result<FreeCompany> {
    let url = LodestoneScraper::route(&format!("freecompany/{}", id))?;
    let text = self.text(url).await?;
    lodestone_parser::parse_free_company(id, &text).map_err(Error::Parse)
  }

  pub fn free_company_search(&self) -> builder::FreeCompanySearchBuilder {
    builder::FreeCompanySearchBuilder::new(self)
  }

  pub fn linkshell(&self, id: u64) -> builder::LinkshellBuilder {
    builder::LinkshellBuilder::new(self, id)
  }

  pub fn linkshell_search(&self) -> builder::LinkshellSearchBuilder {
    builder::LinkshellSearchBuilder::new(self)
  }
}
