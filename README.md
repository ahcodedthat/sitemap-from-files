# `sitemap-from-files`, an XML sitemap generator for static websites

This command-line program generates an [XML sitemap](https://sitemaps.org/) from a collection of files.

Several similar programs exist, including [one written in Python 2 by Google](https://code.google.com/archive/p/sitemap-generators/) and [another one written in JavaScript by @zerodevx](https://github.com/zerodevx/static-sitemap-cli). This one is written in Rust for greater portability: it should work on any [platform supported by Rust](https://doc.rust-lang.org/rustc/platform-support.html) (except platforms that lack `std`).


## Building

To build this program:

1. [Install Rust and Cargo.](https://www.rust-lang.org/tools/install)
2. Get a copy of this source code tree.
3. Run `cargo build --release`. Cargo will download and compile the libraries that `sitemap-from-files` needs, then build `sitemap-from-files` itself. The `--release` option turns on optimizations.

Once you've done this, you'll find a built executable named `sitemap-from-files` in the folder `target/release`. Feel free to move it wherever is convenient (such as `/usr/local/bin`).


## Usage

To use this program, first you need to write a configuration file. It looks like this:

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

See [`Configuration.md`](Configuration.md) for an explanation of what goes into a configuration file.

Once the configuration file is written, simply run `sitemap-from-files path/to/config.toml` to generate a sitemap.
