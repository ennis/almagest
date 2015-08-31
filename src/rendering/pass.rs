use scene::{Scene, Entity};
use nalgebra::{Vec3, Quat};

pub enum Light
{
	// direction encoded in transform component
	Directional,
	// position encoded in transform component
	Point,
	Spot /* TODO */
}

pub struct LightPass
{
	light: Light,
	position: Vec3<f32>,
	rotation: Quat<f32>
}

pub struct ForwardPass
{
	light: LightPass
}

pub struct TranslucentPass;

pub struct ShadowPass
{
	light: LightPass
}

pub struct DeferredPass;

/*pub struct MeshBlock 
{
	mesh: &Mesh
}*/

pub trait FeatureRenderer
{

}
