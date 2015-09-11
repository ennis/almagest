use glfw;
use glfw::{Key};

// A game loop event
#[derive(Copy,Clone)]
pub enum Event
{
	// Window resize event
	WindowResize(u32, u32),
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
