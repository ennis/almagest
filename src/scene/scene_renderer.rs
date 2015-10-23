use scene::*;
use event::*;
use camera::{Camera, TrackballCameraController};

//
// A simple scene renderer
//

#[derive(Copy, Clone, Debug)]
enum DisplayMode 
{
	Shade,
	Shadow,
	Normals,
	Depth
}

/// Contains state for rendering 
struct SceneRenderer 
{
	shader_cache: ShaderCache,
	depth_only_pso: PipelineState,
	normals_only_pso: PipelineState,
	mode: DisplayMode,
	mode_index: usize,
	camera_controller: TrackballCameraController
}

impl SceneRenderer 
{
	/// Creates a new SceneRenderer
	pub fn new(asset_root: &Path) -> SceneRenderer
	{
		SceneRenderer {
			shader_cache: ShaderCache::new(),
			depth_only_pso: load_pipeline_state(&asset_root.join("shaders/render_depth.glsl"), Keywords::empty()),
			normals_only_pso: load_pipeline_state(&asset_root.join("shaders/render_normals.glsl"), Keywords::empty()),
			mode: DisplayMode::Shade,
			mode_index: 0
		}
	}

	fn cycle_display_mode(&mut self)
	{
		const DISPLAY_MODE_CYCLE : [DisplayMode; 4] = [
			DisplayMode::Shade, 
			DisplayMode::Normals, 
			DisplayMode::Shadow, 
			DisplayMode::Depth];

		self.mode_index += 1;
		self.mode_index %= DISPLAY_MODE_CYCLE.len();
		self.mode = DISPLAY_MODE_CYCLE[self.mode_index];
		println!("Display mode: {:?}", self.mode);
	}

	/// Handle application events
	pub fn event(&mut self, event: &Event)
	{
		// pass events to camera controller
		camera_controller.event(event);

		match event 
		{
			&Event::KeyDown(glfw::Key::F) => { self.cycle_display_mode(); },
			_ => {}
		}
	}

	/// Render the scene into the given window
	pub fn render(&mut self, graphics: &Graphics, window: &Window)
	{
		let (width, height) = window.dimensions();
		// TODO
	}

}