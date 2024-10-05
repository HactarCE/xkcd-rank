use std::path::PathBuf;

/// Returns the directory of the current EXE.
pub fn main_dir() -> PathBuf {
    std::env::current_dir()
        .expect("unable to get path to current executable")
        .to_path_buf()
}

pub fn cache_dir() -> PathBuf {
    main_dir().join("cache")
}

pub fn img_dir() -> PathBuf {
    cache_dir().join("img")
}

pub fn img_path(n: usize, ext: &str) -> PathBuf {
    img_dir().join(format!("{n}.{ext}"))
}

pub fn comics_json_path() -> PathBuf {
    cache_dir().join("comics.json")
}
