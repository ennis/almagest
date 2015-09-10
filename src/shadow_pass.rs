
use nalgebra::*;

#[derive(Copy, Clone)]
pub struct LightData
{
    pub light_dir: Vec3<f32>,
    pub light_matrix: Mat4<f32>
}
