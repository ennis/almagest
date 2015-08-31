use scene::{Scene};
use rendering::pass::{FeatureRenderer, ForwardPass};

#[cfg(test)]
use std::thread;

struct Terrain;

struct TerrainRenderer
{
	test: i32
}

impl FeatureRenderer for TerrainRenderer
{
}

#[test]
fn it_works()
{
}
