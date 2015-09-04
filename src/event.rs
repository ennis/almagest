use glfw;
use glfw::{Key};

// A game loop event
pub enum Event 
{
	// Input event 
	MouseButton(glfw::MouseButton, glfw::Action),
	MouseMove(f64, f64),
	// wheel delta
	MouseWheel(f64),
	KeyDown(glfw::Key),
	// dt since last update
	Update(f64),
	// dt since last render
	Render(f64)
}
