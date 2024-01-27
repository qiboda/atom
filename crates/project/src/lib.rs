use std::path::PathBuf;

pub fn project_saved_root_path() -> &'static PathBuf {
    static CELL: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();
    CELL.get_or_init(|| {
        let saved_path: String = std::env::var("ATOM_SAVED_ROOT").unwrap();
        PathBuf::from(&saved_path)
    })
}

pub fn project_asset_root_path() -> &'static PathBuf {
    static CELL: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();
    CELL.get_or_init(|| {
        let asset_path = std::env::var("ATOM_ASSET_ROOT").unwrap();
        PathBuf::from(&asset_path)
    })
}
