use std::path::{Path, PathBuf};

pub(crate) fn repo_root() -> PathBuf {
    if let Ok(current_dir) = std::env::current_dir() {
        if let Some(root) = find_repo_root(&current_dir) {
            return root;
        }
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if let Some(root) = find_repo_root(&manifest_dir) {
        return root;
    }

    panic!(
        "failed to locate repository root from current_dir or CARGO_MANIFEST_DIR ({})",
        manifest_dir.display()
    );
}

fn find_repo_root(start: &Path) -> Option<PathBuf> {
    for candidate in start.ancestors() {
        if candidate.join("Cargo.toml").is_file()
            && candidate.join("third_party/marked/test/specs").is_dir()
        {
            return Some(candidate.to_path_buf());
        }
    }
    None
}
