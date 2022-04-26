use std::{
	borrow::Cow,
	path::{Path, PathBuf},
	str::FromStr,
};

/// Generates an XML sitemap (according to the sitemaps.org protocol) from a collection of files.
#[derive(clap::Parser)]
pub struct Cmd {
	/// Write sitemap to the given file (`-` means standard output), ignoring the configuration file's `sitemap_path`.
	#[clap(short, long)]
	pub output: Option<OutputTo<'static>>,

	/// Explain why files are excluded from the sitemap.
	#[clap(short, long)]
	pub verbose: bool,

	/// Path to the configuration file.
	pub config_file: PathBuf,
}

impl Cmd {
	pub fn output<'a>(&'a self, cfg: &'a crate::config::Config) -> OutputTo<'a> {
		if let Some(output) = &self.output {
			match output {
				OutputTo::Stdout => OutputTo::Stdout,
				OutputTo::File(path) => OutputTo::File(Cow::Borrowed(&**path)),
			}
		}
		else if let Some(sitemap_path) = &cfg.sitemap_path {
			OutputTo::File(Cow::Borrowed(sitemap_path.as_path()))
		}
		else {
			OutputTo::Stdout
		}
	}
}

pub enum OutputTo<'p> {
	Stdout,
	File(Cow<'p, Path>),
}

impl FromStr for OutputTo<'static> {
	type Err = <PathBuf as FromStr>::Err;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s == "-" {
			Ok(Self::Stdout)
		}
		else {
			Ok(Self::File(Cow::Owned(PathBuf::from_str(s)?)))
		}
	}
}
