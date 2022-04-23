mod config;
mod rules;
mod scan;

const MAX_SITEMAP_URLS: u16 = 50_000;
const MAX_SITEMAP_BYTES: usize = 52_428_800;

fn main() -> anyhow::Result<()> {
	use anyhow::Context as _;
	use std::path::PathBuf;

	/// Generates an XML sitemap (according to the sitemaps.org protocol) from a collection of files.
	#[derive(clap::Parser)]
	struct Cmd {
		/// Path to the configuration file.
		config_file: PathBuf,
	}

	let mut cmd = <Cmd as clap::Parser>::parse();

	let cwd: anyhow::Result<PathBuf> =
		std::env::current_dir()
		.context("couldn't get current working directory");

	if !cmd.config_file.is_absolute() {
		cmd.config_file = cwd?.join(cmd.config_file);
	}

	let mut cfg: config::Config = {
		let cfg_bytes =
			std::fs::read(cmd.config_file.as_path())
			.context("couldn't read configuration file")?;

		toml::from_slice(cfg_bytes.as_slice())
		.context("invalid configuration file")?
	};

	cfg.resolve_paths(cmd.config_file.as_path())?;

	if !cfg.rules.iter().any(|rule| matches!(rule.include, Some(true))) {
		anyhow::bail!("the configuration needs to have at least one `[[rule]]` with `include = true`");
	}

	let mut sitemap = Vec::<u8>::new();
	let mut sitemap_writer =
		sitemap::writer::SiteMapWriter::new(&mut sitemap)
		.start_urlset()
		.unwrap();

	self::scan::Scan {
		cfg: &cfg,
		cfg_path: cmd.config_file.as_path(),
		w: &mut sitemap_writer,
	}.scan()?;

	sitemap_writer.end().unwrap();

	anyhow::ensure!(
		sitemap.len() <= MAX_SITEMAP_BYTES,
		"generated sitemap is {} bytes, but the maximum size allowed by the sitemap protocol is {MAX_SITEMAP_BYTES} bytes; please divide the files into multiple sitemaps and join them together in a sitemap index",
		sitemap.len(),
	);

	std::fs::write(
		cfg.sitemap_path.as_path(),
		sitemap.as_slice(),
	)
	.context("couldn't write sitemap file")?;

	Ok(())
}
