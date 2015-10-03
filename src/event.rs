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
	KeyUp(glfw::Key),
	// dt since last update
	Update(f64),
	// dt since last render
	Render(f64)
}

pub enum KeyCode
{
	W=0,
	S,
	A,
	D,
	Up,
	Down,
	Left,
	Right,
	Q,
	E
}

const NUM_KEY_CODES: usize = 10;
const NUM_MOUSE_BUTTONS: usize = 5;

/// Holds the state of input devices
pub struct Input
{
	key_state: [bool; NUM_KEY_CODES],
	mouse_state: [bool; NUM_MOUSE_BUTTONS]
}


impl Input
{
	pub fn new() -> Input
	{
		Input {
			key_state: [false; NUM_KEY_CODES],
			mouse_state: [false; NUM_MOUSE_BUTTONS]
		}
	}

	pub fn event(&mut self, e: &Event)
	{
		match e
		{
			&Event::KeyDown(k) => {
				match k {
					glfw::Key::W => self.press_key(KeyCode::W),
					glfw::Key::S => self.press_key(KeyCode::S),
					glfw::Key::A => self.press_key(KeyCode::A),
					glfw::Key::D => self.press_key(KeyCode::D),
					glfw::Key::Q => self.press_key(KeyCode::Q),
					glfw::Key::E => self.press_key(KeyCode::E),
					glfw::Key::Up => self.press_key(KeyCode::Up),
					glfw::Key::Left => self.press_key(KeyCode::Left),
					glfw::Key::Down => self.press_key(KeyCode::Down),
					glfw::Key::Right => self.press_key(KeyCode::Right),
					_ => {}
				}
			},
			&Event::KeyUp(k) => {
				match k {
					glfw::Key::W => self.release_key(KeyCode::W),
					glfw::Key::S => self.release_key(KeyCode::S),
					glfw::Key::A => self.release_key(KeyCode::A),
					glfw::Key::D => self.release_key(KeyCode::D),
					glfw::Key::Q => self.release_key(KeyCode::Q),
					glfw::Key::E => self.release_key(KeyCode::E),
					glfw::Key::Up => self.release_key(KeyCode::Up),
					glfw::Key::Left => self.release_key(KeyCode::Left),
					glfw::Key::Down => self.release_key(KeyCode::Down),
					glfw::Key::Right => self.release_key(KeyCode::Right),
					_ => {}
				}
			},
			&Event::MouseButton(b, a) => {
				match a {
					glfw::Action::Press => {
						match b {
							glfw::MouseButton::Button1 => self.press_mouse_button(0),
							glfw::MouseButton::Button2 => self.press_mouse_button(1),
							glfw::MouseButton::Button3 => self.press_mouse_button(2),
							glfw::MouseButton::Button4 => self.press_mouse_button(3),
							_ => {}
						}
					},
					glfw::Action::Release => {
						match b {
							glfw::MouseButton::Button1 => self.release_mouse_button(0),
							glfw::MouseButton::Button2 => self.release_mouse_button(1),
							glfw::MouseButton::Button3 => self.release_mouse_button(2),
							glfw::MouseButton::Button4 => self.release_mouse_button(3),
							_ => {}
						}
					},
					_ => {}
				}
			},
			_ => {}
		}
	}

	fn press_key(&mut self, k: KeyCode)
	{
		self.key_state[k as usize] = true;
	}

	fn release_key(&mut self, k: KeyCode)
	{
		self.key_state[k as usize] = false;
	}

	fn press_mouse_button(&mut self, b: usize)
	{
		self.mouse_state[b] = true;
	}

	fn release_mouse_button(&mut self, b: usize)
	{
		self.mouse_state[b] = false;
	}

	pub fn get_key(&self, k: KeyCode) -> bool
	{
		// TODO: some window frameworks may provide an API to get the current key states
		// use it when available?
		self.key_state[k as usize]
	}
}
