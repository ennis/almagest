
use std::rc::Rc;
use std::path::{Path, PathBuf};
use std::fs::File;

pub trait AssetStore
{
    fn open_asset(&self, asset_id: &str) -> File {
        File::open(&self.asset_path(asset_id)).unwrap()
    }

    fn asset_path(&self, asset_id: &str) -> PathBuf;
}

pub trait AssetLoader<T>
{
    fn load<'a>(&'a self, asset_id: &str) -> Rc<T>;
}
