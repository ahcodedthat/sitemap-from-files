mod cmd;
mod config;
mod rules;
mod scan;

const MAX_SITEMAP_URLS: usize = 50_000;
const MAX_SITEMAP_BYTES: usize = 52_428_800;

fn main() -> anyhow::Result<()> {
	use anyhow::Context as _;
	use self::cmd::OutputTo;
	use std::{
		fs,
		io::{self, Write as _},
		path::PathBuf,
	};

	let mut cmd = <self::cmd::Cmd as clap::Parser>::parse();

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

	match cmd.output(&cfg) {
		OutputTo::Stdout => {
			let stdout = io::stdout();
			let mut stdout = stdout.lock();
			stdout.write_all(sitemap.as_slice())
			.and_then(|_| stdout.flush())
		}

		OutputTo::File(path) => fs::write(&*path, sitemap.as_slice()),
	}
	.context("couldn't write sitemap file")?;

	Ok(())
}
