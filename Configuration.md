# `sitemap-from-files` — configuration file reference

`sitemap-from-files` requires a configuration file in the [TOML](https://toml.io/) language. This document describes the content of the configuration file.


## Example

Here's an example configuration file:

```toml
root_dir = "path/to/site"
root_url = "https://www.example.com/"
sitemap_path = "path/to/site/sitemap.xml"

[[rule]]
match = '\.html$'
include = true

[[rule]]
match = '(^|/)index\.html$'
replace = '$1'

[[rule]]
match = '^secret/'
include = false
```


## Top level

The following settings appear at the top level of the configuration file.

### `root_dir`

```toml
# Type: string
# Required
root_dir = "path/to/site"
```

This is the path to the folder containing the web pages you want to include in your sitemap.

If this path is relative, it will be resolved from the location of the configuration file itself. So, if your files are at `/home/me/mysite/public` and the configuration file is at `/home/me/mysite/sitemap-from-files.toml`, then you can just write:

```toml
root_dir = "public"
```

### `root_url`

```toml
# Type: string
# Required
root_url = "https://www.example.com/"
```

This is the base URL of your website. The URL must be absolute.

### `sitemap_path`

```toml
# Type: string
# Optional
sitemap_path = "path/to/site/sitemap.xml"
```

This is the path to the sitemap file that `sitemap-from-files` will generate.

If this field is not present, the sitemap will be written to standard output instead of a file. This can be overridden by the command-line option `-o`.

Just like `root_dir`, this path can be relative to the configuration file. So, if your files are at `/home/me/mysite/public` and the configuration file is at `/home/me/mysite/sitemap-from-files.toml`, then you can just write:

```toml
sitemap_path = "public/sitemap.xml"
```


## Rules

Rules begin with a `[[rule]]` heading. They tell `sitemap-from-files` which files to list in the sitemap and control a few other aspects of its behavior.

Each rule must have a `match` field and at least one other field. Additionally, there must be at least one rule with `include = true`.

It is possible for more than one rule to match the same file. If they do, their effects are combined. If more than one matching rule has the same effect (such as `replace`), later rules take precedence over earlier rules. For example, if a file is matched by three rules and the first two matching rules have a `replace` field, then only the second matching rule's `replace`ment is performed; the first matching rule's `replace` has no effect for this file (but may still affect other files).

Available fields for rules are as follows.

### `match`

```toml
[[rule]]
# Type: string (regular expression)
# Required
match = '\.html$'
```

A regex (regular expression) that is matched against the URL-path of each file. The rule is applied only to files whose path matches this regex.

The “URL-path” is formed by taking the path to the file in question, turning it into a `file:///` URL (with percent-encoding), then stripping off the part up to the `root_dir`. The URL-path does *not* start with a slash. For example, if your `root_dir` is `/var/www` and the file in question is `/var/www/foo/bar baz.html`, then the URL-path will be `foo/bar%20baz.html`.

Regex matching is provided by the Rust [`regex`](https://github.com/rust-lang/regex) library. See its documentation for [supported regex syntax](https://docs.rs/regex/1.5.5/regex/index.html#syntax). Note that look-around and back-references are unfortunately not supported.

The regex may match a substring of the URL-path and need not match the entire URL-path. Thus, `match = '\.html$'` will match all files ending in `.html`. An empty string (that is, `match = ''`) will match *all* files. To match against the entire URL-path, your regex should begin with `^` and end with `$`.

Because most regexes have a lot of backslashes, the `match` string should usually be written as a TOML *literal string* (that is, in single quotes `'…'`) instead of a TOML *basic string* (that is, in double quotes `"…"`). That way, you don't have to type each backslash character twice. See [the TOML website](https://toml.io/) to learn more about TOML syntax.

Only regular files and symbolic links to regular files are matched. `sitemap-from-files` will never list anything else (such as a folder or named pipe) in a sitemap, regardless of rules.

### `include`

```toml
[[rule]]
match = '…'
# Type: boolean
# Optional
include = true
```

This can be either `true` or `false`. If `include = true`, the file matched by this rule will be included in the sitemap.

A file will only be included in the sitemap if at least one rule that matches it has `include = true`. That's why there must be at least one rule with `include = true`: otherwise the sitemap would be completely empty.

`include = false` can be used to override an earlier rule's `include = true`. This can be used to include all HTML pages with some exceptions. For example, these rules will include all files whose name ends in `.html` *except* files inside the folder named `secret`:

```toml
[[rule]]
match = '\.html$'
include = true

[[rule]]
match = '^secret/'
include = false
```

Note that `include = true` can similarly override an earlier `include = false`.

If a `robots.txt` file is present in the `root_dir`, files excluded by `robots.txt` will never be listed in the sitemap, regardless of rules.

### `replace`

```toml
[[rule]]
match = '…'
# Type: string
# Optional
replace = 'foo'
```

If this field is present, then the portion of the URL-path matched by the `match` field will be replaced with the contents of this field.

`$` symbols in the replacement text have a special meaning: they refer to capture groups in the `match`. If you want to include a literal `$` character in the replacement, write `$$` here. See [the regex engine's documentation](https://docs.rs/regex/1.5.5/regex/struct.Regex.html#replacement-string-syntax) for details.

The replacement can be an empty string, in which case the matched portion of the URL-path is simply removed.

The most common use for `replace` is to remove `index.html` from URLs. Here's how to do that:

```toml
[[rule]]
match = '(^|/)index\.html$'
replace = '$1'
```

Note the use of a capture group to preserve the slash before `index.html`.

### `replace_limit`

```toml
[[rule]]
match = '…'
replace = '…'
# Type: non-negative integer
# Optional
# No effect without `replace`
replace_limit = 1
```

Limits how many times `replace` is applied.

Normally, if the regex in `match` matches more than once in the URL-path, then *all* matches are replaced. This field can be used to only replace *some* of the matches.

For example, consider this rule:

```toml
# Replaces the first two slashes in the URL-path with underscores.
[[rule]]
match = '/'
replace = '_'
replace_limit = 2
```

If this rule is applied to the URL-path `one/two/three/four.html`, the result will be `one_two_three/four.html`.

`replace_limit` has no effect unless the rule it appears in also has a `replace` field.

### `check_html_meta_robots`

```toml
[[rule]]
match = '…'
# Type: boolean
# Optional
check_html_meta_robots = true
```

If true, files matched by this rule will be parsed as UTF-8 HTML. If such a file contains a `<meta name=robots>` element whose `content` attribute contains `noindex`, then it will be excluded from the sitemap.

Don't enable this for anything other than HTML files that are encoded in ASCII or UTF-8. (It may also work with other ASCII-compatible encodings such as ISO 8859, but that is not guaranteed.) This limitation exists because [the `html5ever` HTML parser library does not currently implement](https://github.com/servo/html5ever/issues/18) the [HTML5 encoding sniffing algorithm](https://html.spec.whatwg.org/multipage/parsing.html#encoding-sniffing-algorithm) and there is no other reliable way to detect the encoding of an HTML file.
