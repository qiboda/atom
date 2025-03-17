use std::{fs, path::PathBuf};

pub fn log_all_path() {
    let root_path = project_root_path();
    println!(
        "project_root_path:{:?} and absolute form: {:?}",
        root_path,
        fs::canonicalize(root_path)
    );

    let saved_path = project_saved_root_path();
    println!(
        "project_saved_root_path:{:?} and absolute form: {:?}",
        saved_path,
        fs::canonicalize(saved_path)
    );

    let asset_path = project_asset_root_path();
    println!(
        "project_asset_root_path:{:?} and absolute form: {:?}",
        asset_path,
        fs::canonicalize(asset_path)
    );
}

pub fn project_root_path() -> &'static PathBuf {
    static CELL: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();
    CELL.get_or_init(|| {
        let root_path = std::env::var("ATOM_ROOT").unwrap();
        fs::canonicalize(PathBuf::from(&root_path)).unwrap()
    })
}

pub fn project_saved_root_path() -> &'static PathBuf {
    static CELL: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();
    CELL.get_or_init(|| {
        let saved_path: String = std::env::var("ATOM_SAVED_ROOT").unwrap();
        println!("ATOM_SAVED_ROOT: {}", saved_path);
        fs::canonicalize(PathBuf::from(&saved_path)).unwrap()
    })
}

pub fn project_asset_root_path() -> &'static PathBuf {
    static CELL: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();
    CELL.get_or_init(|| {
        let asset_path = std::env::var("ATOM_ASSET_ROOT").unwrap();
        fs::canonicalize(PathBuf::from(&asset_path)).unwrap()
    })
}

pub fn project_config_root_path() -> &'static PathBuf {
    static CELL: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();
    CELL.get_or_init(|| {
        let config_path = std::env::var("ATOM_CONFIG_ROOT").unwrap();
        println!("config_path: {:?}", config_path);
        fs::canonicalize(PathBuf::from(&config_path)).unwrap()
    })
}

pub fn project_processed_asset_root_path() -> &'static PathBuf {
    static CELL: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();
    CELL.get_or_init(|| {
        let asset_path = std::env::var("ATOM_PROCESSED_ASSET_ROOT").unwrap();
        fs::canonicalize(PathBuf::from(&asset_path)).unwrap()
    })
}
