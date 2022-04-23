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
	/// A regular expression that must match the URL-path for this rule to be applied.
	///
	/// URL-paths never begin with a slash and always end with the file name. For example, if:
	///
	/// * you're running this program on Windows,
	/// * the `root_dir` is `C:\Users\Mister Rogers\Documents\My Site`, and
	/// * the file being considered is `C:\Users\Mister Rogers\Documents\My Site\neighbors\you.html`,
	///
	/// then this regex will be matched against the string `neighbors/you.html`.
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
