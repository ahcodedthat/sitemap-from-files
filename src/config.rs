use anyhow::Context as _;
use regex::Regex;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use url::Url;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
	pub root_dir: PathBuf,
	pub root_url: Url,
	pub sitemap_path: Option<PathBuf>,
	#[serde(rename = "rule")]
	pub rules: Vec<Rule>,
}

impl Config {
	pub fn resolve_paths(&mut self, config_file_path: &Path) -> anyhow::Result<()> {
		let parent =
			config_file_path.parent()
			.context("configuration file path doesn't have a parent")?;

		for path in [Some(&mut self.root_dir), self.sitemap_path.as_mut()] {
		if let Some(path) = path {
		if !path.is_absolute() {
			*path = parent.join(&*path);
		}}}

		Ok(())
	}
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
	#[serde(with = "serde_regex")]
	pub r#match: Regex,
	pub replace: Option<String>,
	#[serde(default = "Rule::default_replace_limit")]
	pub replace_limit: usize,
	pub include: Option<bool>,
}

impl Rule {
	fn default_replace_limit() -> usize {
		0
	}
}
