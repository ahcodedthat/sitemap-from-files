use crate::config::{Config, Rule};
use std::borrow::Cow;

pub struct Rules<'c> {
	config: &'c Config,
	regex_set: regex::RegexSet,
}

impl<'c> Rules<'c> {
	pub fn new(config: &'c Config) -> Self {
		let regex_set =
			regex::RegexSet::new(
				config.rules.iter()
				.map(|rule| rule.r#match.as_str())
			)
			.expect("one or more `match`es in the configuration file are invalid, but this is impossible because they have already been validated");

		Self { config, regex_set }
	}

	/// Determines what behavior should be used for the file at the given `path`.
	///
	/// The return value is `None` if the rules say to exclude the file from the sitemap, or `Some` if they say to include it.
	pub fn apply<'p, 's>(&'s self, path: &'p str) -> Option<AppliedRules<'s, 'p>> {
		let matching_rules = self.regex_set.matches(path);

		if !matching_rules.matched_any() {
			return None;
		}

		let mut include = false;
		let mut applied = AppliedRules {
			replacing_rule: None,
			path: Cow::Borrowed(path),
			check_html_meta_robots: false,
		};
		let mut replace: Option<(&'c Rule, &'c str)> = None;

		for matching_rule in matching_rules {
			// `matching_rule` is currently an index into the `Config::rules` array. Look it up.
			let matching_rule = &self.config.rules[matching_rule];

			include = matching_rule.include.unwrap_or(include);

			if let Some(matching_replace) = &matching_rule.replace {
				replace = Some((matching_rule, matching_replace.as_str()));
			}

			if let Some(flag) = matching_rule.check_html_meta_robots {
				applied.check_html_meta_robots = flag;
			}
		}

		if !include {
			return None;
		}

		if let Some((replacing_rule, replace)) = replace {
			applied.replacing_rule = Some(replacing_rule);
			applied.path =
				replacing_rule.r#match
				.replacen(path, replacing_rule.replace_limit, replace);
		}

		Some(applied)
	}
}

pub struct AppliedRules<'c, 'p> {
	/// The rule whose [`Rule::replace`] has been applied. `None` if none of the matching rules have a `replace`.
	pub replacing_rule: Option<&'c Rule>,

	/// The new URL-path for the sitemap entry. Will be [`Cow::Owned`] if the path has been subjected to replacement, or [`Cow::Borrowed`] if not.
	pub path: Cow<'p, str>,

	/// Whether to try to parse the file as HTML and look for `<meta name=robots>`.
	pub check_html_meta_robots: bool,
}
