use std::{fs, path::PathBuf};

#[derive(Debug, Default)]
pub struct ProjectPaths;

impl ProjectPaths {
    pub fn root_path() -> &'static PathBuf {
        static CELL: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();
        CELL.get_or_init(|| {
            // 一路向上查找.atom.project文件所在路径。
            let mut dir = std::env::current_dir().expect("env current dir must get");
            loop {
                if dir.join(".atom.project").exists() {
                    return fs::canonicalize(dir).expect("canonicalize dir must succeed");
                }
                if !dir.pop() {
                    panic!("Cannot find .atom.project in any parent directory");
                }
            }
        })
    }

    pub fn saved_path() -> &'static PathBuf {
        static CELL: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();
        CELL.get_or_init(|| ProjectPaths::root_path().join("saved"))
    }

    pub fn assets_path() -> &'static PathBuf {
        static CELL: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();
        CELL.get_or_init(|| ProjectPaths::root_path().join("assets"))
    }

    pub fn config_root_path() -> &'static PathBuf {
        static CELL: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();
        CELL.get_or_init(|| ProjectPaths::root_path().join("config"))
    }

    pub fn processed_assets_path() -> &'static PathBuf {
        static CELL: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();
        CELL.get_or_init(|| ProjectPaths::root_path().join("processed_assets"))
    }
}
