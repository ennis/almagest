
use std::rc::Rc;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::cell::{RefCell};
use std::collections::{HashMap};

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

// TODO: rename this to 'object cache' or something
pub struct AssetCache<T>
{
	map: RefCell<HashMap<String, Rc<T>>>
}

impl<T> AssetCache<T>
{
	pub fn new() -> AssetCache<T>
	{
		AssetCache {
			map: RefCell::new(HashMap::new())
		}
	}

	pub fn load_with<'a, F: Fn(&'a str) -> T>(&self, id: &'a str, f: F) -> Rc<T>
	{
		// cannot use this because the closure may in turn access the asset cache
		//self.map.entry(id).or_insert_with(Rc::new(f(id))).clone();
		let key_found = self.map.borrow().contains_key(id);
		if !key_found {
			let val = Rc::new(f(id));
			self.map.borrow_mut().insert(id.to_string(), val.clone());
			val
		} else {
			trace!("Reusing asset {}", id);
			self.map.borrow().get(id).unwrap().clone()
		}
	}
}
