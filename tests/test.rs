use snapbox::path::PathFixture;
use std::path::{Path, PathBuf};

#[test]
fn test() {
	let test_data: PathFixture =
		PathFixture::mutable_temp()
		.unwrap()
		.with_template(Path::new(env!("CARGO_MANIFEST_DIR")).join("test-data").as_path())
		.unwrap();
	let test_data_path: &Path = test_data.path().unwrap();

	// Set file times to a consistent value.
	for (f, t) in [
		(&["site", "foo", "bar.html"][..], 101),
		(&["site", "index.html"][..], 42),
		(&["site", "index.txt"][..], 54),
		(&["site", "secret", "secrets.html"][..], 0xdeadbeef),
		(&["site", "super-secret", "more-secrets.html"][..], 0xdeadbeef),
	] {
		let f = PathBuf::from_iter(
			[test_data_path].into_iter()
			.chain(
				f.iter()
				.map(|pf: &&str| Path::new(*pf))
			)
		);
		let t = filetime::FileTime::from_unix_time(t, 500_000_000);

		filetime::set_file_mtime(f.as_path(), t)
		.unwrap_or_else(|error| panic!("couldn't set file time on `{}`: {error}", f.display()));
	}

	snapbox::cmd::Command::new(snapbox::cmd::cargo_bin!("sitemap-from-files"))
	.args(["-o", "-"])
	.arg(test_data_path.join("config.toml"))
	.assert()
	.success()
	.stderr_eq("")
	.stdout_eq_path(PathBuf::from_iter([env!("CARGO_MANIFEST_DIR"), "test-data", "expected-sitemap.xml"]));

	test_data.close().unwrap();
}
