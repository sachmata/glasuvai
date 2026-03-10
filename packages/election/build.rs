//! Build script: computes a SHA-256 digest over all ballot data files
//! and exposes it as the `GLASUVAI_DATA_SHA256` environment variable
//! (accessible via `env!()` in the crate source).

use sha2::{Digest, Sha256};
use walkdir::WalkDir;

fn main() {
    let data_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/elections");
    println!("cargo::rerun-if-changed={data_dir}");

    let mut hasher = Sha256::new();
    let mut paths: Vec<_> = WalkDir::new(data_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        // Skip non-data files (e.g. SOURCES.md)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "toml"))
        .map(|e| e.into_path())
        .collect();

    // Sort for deterministic ordering across platforms
    paths.sort();

    for path in &paths {
        let content = std::fs::read(path).expect("failed to read data file");
        hasher.update(&content);
    }

    let digest = format!("{:x}", hasher.finalize());
    println!("cargo::rustc-env=GLASUVAI_DATA_SHA256={digest}");
}
