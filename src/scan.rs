use anyhow::Context as _;
use crate::{
	config::Config,
	MAX_SITEMAP_URLS,
	rules::Rules,
};
use std::{
	cmp::Ordering,
	fs::{self, File},
	io::{self, Write},
	path::{Path, PathBuf},
};
use url::Url;

mod check_html_meta;

pub struct Scan<'c, W: Write> {
	pub cfg: &'c Config,
	pub cfg_path: &'c Path,
	pub w: &'c mut sitemap::writer::UrlSetWriter<W>,
}

impl<'c, W: Write> Scan<'c, W> {
	pub fn scan(self) -> anyhow::Result<()> {
		anyhow::ensure!(
			!self.cfg.root_url.cannot_be_a_base(),
			"the configured `root_url`, `{}`, is unusable as it cannot serve as a base URL",
			self.cfg.root_url,
		);

		let robots_path: PathBuf = self.cfg.root_dir.join("robots.txt");

		let robot: Option<texting_robots::Robot> = match fs::read(robots_path.as_path()) {
			Err(error) if error.kind() == io::ErrorKind::NotFound => None,

			Err(error) => return Err(
				anyhow::Error::new(error)
				.context("couldn't read `robots.txt`")
			),

			Ok(robots_bytes) => Some(
				texting_robots::Robot::new("*", robots_bytes.as_slice())
				.context("`robots.txt` is invalid")?
			),
		};

		let root_dir_url: Url =
			Url::from_directory_path(self.cfg.root_dir.as_path())
			.map_err(|()| anyhow::anyhow!("`{}` is not a valid `root_dir`", self.cfg.root_dir.display()))?;

		let rules = Rules::new(self.cfg);

		let mut scanner = Scanner {
			s: self,
			root_dir_url: &root_dir_url,
			robot: &robot,
			rules: &rules,
			urls: Vec::new(),
		};
		scanner.scan_dir(scanner.s.cfg.root_dir.as_path())?;

		scanner.urls.sort_by(|a, b| {
			use sitemap::structs::Location;

			match (&a.loc, &b.loc) {
				(Location::None, Location::None)
				| (Location::ParseErr(_), Location::ParseErr(_))
				=> Ordering::Equal,

				(Location::None, _)
				| (_, Location::ParseErr(_))
				=> Ordering::Less,

				(_, Location::None)
				| (Location::ParseErr(_), _)
				=> Ordering::Greater,

				(Location::Url(a), Location::Url(b))
				=> a.cmp(b),
			}
		});

		for url in scanner.urls {
			scanner.s.w.url(url)
			.context("couldn't write sitemap entry")?;
		}

		Ok(())
	}
}

struct Scanner<'a, W: Write> {
	robot: &'a Option<texting_robots::Robot>,
	root_dir_url: &'a Url,
	rules: &'a Rules<'a>,
	s: Scan<'a, W>,
	urls: Vec<sitemap::structs::UrlEntry>,
}

impl<'a, W: Write> Scanner<'a, W> {
	fn scan_dir(&mut self, dir: &Path) -> anyhow::Result<()> {
		let read_dir =
			std::fs::read_dir(dir)
			.with_context(|| format!("couldn't open folder `{}`", dir.display()))?;

		for dent in read_dir {
			let dent = dent.with_context(|| format!("couldn't read folder `{}`", dir.display()))?;
			let dent_path = dent.path();

			let dent_type =
				dent.file_type()
				.with_context(|| format!("couldn't get file type of `{}`", dent_path.display()))?;

			if dent_type.is_dir() {
				self.scan_dir(dent_path.as_path())?;
				continue;
			}

			let mut fd =
				File::open(dent_path.as_path())
				.with_context(|| format!("couldn't open file `{}`", dent_path.display()))?;

			let md =
				fd.metadata()
				.with_context(|| format!("couldn't get file system metadata for file `{}`", dent_path.display()))?;

			if !md.is_file() {
				continue;
			}

			// An absolute `file:` URL.
			let file_url =
				Url::from_file_path(dent_path.as_path())
				.map_err(|()| anyhow::format_err!("path `{}` couldn't be converted into a URL", dent_path.display()))?;

			// A relative URL (just the path).
			let url_rel =
				self.root_dir_url.make_relative(&file_url)
				.unwrap_or_else(|| panic!("the URL `{file_url}` could not be made relative to the URL `{}`", self.root_dir_url));

			// A relative URL, with replacements applied.
			let applied_rules = match self.rules.apply(url_rel.as_str()) {
				Some(ok) => ok,
				None => continue,
			};

			// The absolute URL of the file, as it will appear on the web.
			let web_url =
				self.s.cfg.root_url.join(&*applied_rules.path)
				.with_context(|| format!(
					"applying configured replacements to `{url_rel}` yielded `{}`, which is not a valid relative URL",
					applied_rules.path,
				))?;

			anyhow::ensure!(
				web_url.as_str().starts_with(self.s.cfg.root_url.as_str()),
				"applying replacements to the path `{}` resulted in the URL `{web_url}`, which does not start with the configured `root_url`, `{}`, in violation of the sitemaps protocol",
				dent_path.display(),
				self.s.cfg.root_url,
			);

			// Check if this file is excluded by the robots file. Note that the robots protocol expects a leading slash, and `Url::make_relative` makes a string *without* a leading slash, so we'll have to copy the whole URL-path into a new string with such a slash.
			if let Some(robot) = self.robot {
			if !robot.allowed(format!("/{url_rel}").as_str()) {
				continue;
			}}

			// If this file is HTML, check it for `<meta name=robots>`.
			if applied_rules.check_html_meta_robots {
				let html_meta =
					self::check_html_meta::HtmlMeta::read(&mut fd)
					.with_context(|| format!("couldn't read HTML file `{}`", dent_path.display()))?;

				if html_meta.no_index {
					continue;
				}
			}

			// We can close the file now.
			drop(fd);

			// Start constructing a `UrlEntry`.
			let mut entry = sitemap::structs::UrlEntry::builder();
			entry = entry.loc(web_url);

			// Convert the last-modified time.
			if let Some(t) = md.modified().ok().map(|t| {
				use chrono::*;

				// Convert the time stamp to `chrono::DateTime` in UTC.
				let t = DateTime::<Utc>::from(t);

				// Convert the time stamp to a Unix timestamp.
				let t = t.timestamp();

				// Convert the time stamp to a `NaiveDateTime`. This is the same as before, but rounded to a whole second.
				let t = NaiveDateTime::from_timestamp(t, 0);

				// Convert the time stamp back to `DateTime<Utc>`.
				let t = DateTime::<Utc>::from_utc(t, Utc);

				// Finally, convert it to the representation `sitemap` wants.
				DateTime::<FixedOffset>::from(t)
			}) {
				entry = entry.lastmod(t);
			}

			// Insert it.
			self.urls.push(
				entry.build()
				.context("couldn't generate sitemap entry")?
			);

			// Make sure not to exceed 50k URLs.
			anyhow::ensure!(
				self.urls.len() <= MAX_SITEMAP_URLS,
				"more than {MAX_SITEMAP_URLS} files are to be included in the sitemap, which is not allowed by the sitemaps protocol; please divide the files into multiple sitemaps and join them together in a sitemap index",
			);
		}

		Ok(())
	}
}
