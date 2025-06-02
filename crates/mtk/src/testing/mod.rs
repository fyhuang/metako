use std::path::PathBuf;

/// Return the path to one of the testdata folders
pub fn testdata_path(name: &str) -> PathBuf {
    // Path to "base" crate
    let base_crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let testdata_path = base_crate_path // metako/crates/mtk
        .parent().unwrap() // metako/crates
        .parent().unwrap() // metako
        .join("testdata").join(name);
    assert!(testdata_path.is_dir());
    testdata_path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_testdata_path() {
        let path = testdata_path("mixed");
        println!("{:?}", path);
        assert!(path.is_dir());
    }
}
