use glfw;
use glfw::{Key};

// A game loop event
pub enum Event 
{
	// Input event 
	MouseButton(glfw::MouseButton),
	// TODO: relative mouse position 
	MouseMove(f64, f64),
	// wheel delta
	MouseWheel(f64),
	KeyDown(glfw::Key),
	// dt since last update
	Update(f64),
	// dt since last render
	Render(f64)
}

pub enum Action 
{
	// TODO named events?
	MoveViewpoint(f32, f32),
	MoveA(f32, f32),
	MoveB(f32, f32),
	Zoom(f32),
	Jump,
	Action1,
	Action2
}

// Translates input events to actions
/*pub struct InputMapper;

impl InputMapper
{
	pub fn map(&self, ie: Event) -> Option<Action>
	{
		match ie {
			InputEvent::MouseMoveEvent(x, y) => Some(Action::MoveA(x, y)),
			InputEvent::MouseWheelEvent(x) => Some(Action::Zoom(x)),
			InputEvent::MouseButtonEvent(b) => match b {
				glfw::MouseButton::Button1 => Some(Action::Action1),
				glfw::MouseButton::Button2 => Some(Action::Action2),
				_ => None
			},
			InputEvent::KeyEvent(k) => match k {
				Key::Space => Some(Action::Jump),
				_ => None
			}
		}
	}
}*/
