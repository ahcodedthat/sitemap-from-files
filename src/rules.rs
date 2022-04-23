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

	pub fn apply<'p, 's>(&'s self, path: &'p str) -> AppliedRules<'s, 'p> {
		let matching_rules = self.regex_set.matches(path);

		if !matching_rules.matched_any() {
			return AppliedRules::Exclude;
		}

		let mut include = false;
		let mut replacing_rule: Option<(&'c Rule, &'c str)> = None;

		for matching_rule in matching_rules {
			// `matching_rule` is currently an index into the `Config::rules` array. Look it up.
			let matching_rule = &self.config.rules[matching_rule];

			include = matching_rule.include.unwrap_or(include);

			if let Some(replace) = &matching_rule.replace {
				replacing_rule = Some((matching_rule, replace.as_str()));
			}
		}

		if !include {
			return AppliedRules::Exclude;
		}

		let path: Cow<'p, str> = match replacing_rule {
			None => Cow::Borrowed(path),
			Some((replacing_rule, replace)) =>
				replacing_rule.r#match
				.replacen(path, replacing_rule.replace_limit, replace),
		};

		AppliedRules::Include {
			replacing_rule: replacing_rule.map(|(r, _)| r),
			path,
		}
	}
}

pub enum AppliedRules<'c, 'p> {
	/// This file is *not* to be included in the sitemap.
	Exclude,

	/// This file is to be included in the sitemap.
	Include {
		/// The rule whose [`Rule::replace`] has been applied. `None` if none of the matching rules have a `replace`.
		replacing_rule: Option<&'c Rule>,

		/// The new URL-path for the sitemap entry. Will be [`Cow::Owned`] if the path has been subjected to replacement, or [`Cow::Borrowed`] if not.
		path: Cow<'p, str>,
	},
}
