use glfw;
use glfw::{Key, WindowEvent, Context};
use std::sync::mpsc::{Receiver};
use gl;
use event::Event;

pub struct Window
{
	win: glfw::Window,
	events: Receiver<(f64, WindowEvent)>,
	absolute_wheel_pos: f64
}

pub struct WindowSettings
{
	title: String,
	dimensions: (u32, u32)
}

impl WindowSettings
{
	pub fn new(title: &str, dimensions: (u32, u32)) -> WindowSettings
	{
		WindowSettings {
			title: title.to_string(),
			dimensions: (800, 600)
		}
	}
	
	pub fn build(&self, glfw: &glfw::Glfw) -> Option<Window>
	{
		let result = glfw.create_window(self.dimensions.0, self.dimensions.1, &self.title, glfw::WindowMode::Windowed);
		
		result.map(|(mut win, events)| {
			win.set_key_polling(true);
			win.make_current();
			
			// Load GL function pointers
			gl::load_with(|s| win.get_proc_address(s));
			
			Window {
				win: win,
				events: events,
				absolute_wheel_pos: 0.0
			}
		})
	}
}


impl Window
{
	pub fn cursor_pos(&self) -> (f64, f64) { self.win.get_cursor_pos() }
	pub fn mouse_wheel_pos(&self) -> f64 { self.absolute_wheel_pos }
	
	pub fn event_loop<F: FnMut(Event, &glfw::Window) -> bool>(
		&mut self,
		glfw: &mut glfw::Glfw, 
		mut event_handler: F)
	{
		while !self.win.should_close() {
			// Translate input events
			glfw.poll_events();
			for (_, event) in glfw::flush_messages(&self.events) {
				match event {
					glfw::WindowEvent::Key(Key::Escape, _, glfw::Action::Press, _) => {
						self.win.set_should_close(true);
					},
					glfw::WindowEvent::Key(k, _, glfw::Action::Press, _) => {
						event_handler(Event::KeyDown(k), &self.win);
					},
					glfw::WindowEvent::Key(k, _, glfw::Action::Repeat, _) => {
						event_handler(Event::KeyDown(k), &self.win);
					},
					_ => {}
				}
			}
			// send render event
			event_handler(Event::Update(0.0), &self.win);
			event_handler(Event::Render(0.0), &self.win);
			self.win.swap_buffers();
		}
	}

}