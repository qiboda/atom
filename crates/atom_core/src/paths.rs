use std::{fs, path::PathBuf};

/// 项目路径工具。以 `.atom.project` 标记文件定位项目根目录，并缓存常用目录路径。
#[derive(Debug, Default)]
pub struct ProjectPaths;

impl ProjectPaths {
    /// 项目根目录：从当前工作目录向上查找 `.atom.project` 标记文件定位，结果缓存。
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

    /// 保存数据目录：`<root>/saved`。
    pub fn saved_path() -> &'static PathBuf {
        static CELL: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();
        CELL.get_or_init(|| ProjectPaths::root_path().join("saved"))
    }

    /// 资源目录：`<root>/assets`。
    pub fn assets_path() -> &'static PathBuf {
        static CELL: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();
        CELL.get_or_init(|| ProjectPaths::root_path().join("assets"))
    }

    /// 配置目录：`<root>/config`。
    pub fn config_root_path() -> &'static PathBuf {
        static CELL: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();
        CELL.get_or_init(|| ProjectPaths::root_path().join("config"))
    }

    /// 处理后的资源目录：`<root>/processed_assets`。
    pub fn processed_assets_path() -> &'static PathBuf {
        static CELL: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();
        CELL.get_or_init(|| ProjectPaths::root_path().join("processed_assets"))
    }
}
