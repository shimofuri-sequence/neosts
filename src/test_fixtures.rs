use std::path::PathBuf;

pub(crate) fn fixture_path(name: &str) -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    path.exists().then_some(path)
}
