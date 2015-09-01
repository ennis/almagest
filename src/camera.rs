use input;
use nalgebra::*;
use glfw;
use std;
use num::traits::{Zero, One};

#[derive(Copy, Clone)]
pub struct Camera
{
	pub view_matrix: Mat4<f32>,
	pub proj_matrix: Mat4<f32>
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum TrackballCameraMode
{
	Idle,
	Rotate,
	Pan
}

#[derive(Copy, Clone)]
pub struct TrackballCameraSettings
{
	initial_eye: Pnt3<f32>,
	field_of_view: f64,
	near_plane: f64,
	far_plane: f64,
	sensitivity: f64
}

impl TrackballCameraSettings
{
	pub fn default() -> TrackballCameraSettings
	{
		TrackballCameraSettings {
			initial_eye: Pnt3::new(0.0f32, 0.0f32, 2.0f32),
			field_of_view: 45.0,
			near_plane: 0.01,
			far_plane: 1000.0,
			sensitivity: 0.1
		}
	}
	
	pub fn with_eye_center(self, initial_eye: &Pnt3<f32>) -> Self
	{
		TrackballCameraSettings {
			initial_eye: *initial_eye,
			.. self
		}
	}
	
	pub fn with_field_of_view(self, fov: f64) -> Self
	{
		TrackballCameraSettings {
			field_of_view: fov,
			.. self
		}
	}

	pub fn with_near_plane(self, near_plane: f64) -> Self 
	{
		TrackballCameraSettings {
			near_plane: near_plane,
			.. self
		}
	}
	
	pub fn with_far_plane(self, far_plane: f64) -> Self
	{
		TrackballCameraSettings {
			far_plane: far_plane,
			.. self
		}
	}
	
	pub fn with_sensitivity(self, sensitivity: f64) -> Self 
	{
		TrackballCameraSettings {
			sensitivity: sensitivity,
			.. self
		}
	}

	pub fn build(&self) -> TrackballCameraController
	{
		TrackballCameraController {
			settings: *self,
			v_eye: self.initial_eye.to_vec(),
			scene_rot: (0.0, 0.0),
			last_wheel_offset: 0.0,
			mode: TrackballCameraMode::Idle,
			last_mouse_pos: (0.0, 0.0)
		}
	}
}

// Should take as input an 'event' struct
pub struct TrackballCameraController
{
	settings: TrackballCameraSettings,
	v_eye: Vec3<f32>,
	scene_rot: (f64, f64),
	last_wheel_offset: f64,
	mode: TrackballCameraMode,
	last_mouse_pos: (f64, f64)
}

impl TrackballCameraController
{
	pub fn get_camera(&mut self, window: &glfw::Window) -> Camera
	{
		let camera_up = Vec3::new(0.0f32, 1.0f32, 0.0f32);
		let camera_right = Vec3::new(1.0f32, 0.0f32, 0.0f32);
		let camera_front = Vec3::new(0.0f32, 0.0f32, 1.0f32);

		let leftmb = window.get_mouse_button(glfw::MouseButton::Button1) == glfw::Action::Press;
		let middlemb = window.get_mouse_button(glfw::MouseButton::Button3) == glfw::Action::Press;
		let (pos_x, pos_y) = window.get_cursor_pos();
		let (width, height) = window.get_size();
		let aspect_ratio = width / height;

		// Mode update
		if !leftmb && !middlemb {
			self.mode = TrackballCameraMode::Idle
		}
		else if self.mode != TrackballCameraMode::Rotate && leftmb {
			self.mode = TrackballCameraMode::Rotate;
			self.last_mouse_pos = (pos_x, pos_y);
		}
		else if self.mode != TrackballCameraMode::Pan && middlemb {
			self.mode = TrackballCameraMode::Pan;
			self.last_mouse_pos = (pos_x, pos_y);
		};

		let (last_x, last_y) = self.last_mouse_pos;
		let dx = (pos_x - last_x) * self.settings.sensitivity;
		let dy = (pos_y - last_y) * self.settings.sensitivity;

		// Update scene rotation
		if self.mode != TrackballCameraMode::Idle
		{
			if self.mode == TrackballCameraMode::Rotate
			{
				let rot_speed = 0.5f64;
				let TWOPI = 2.0 * std::f64::consts::PI;
				let (scene_rot_x, scene_rot_y) = self.scene_rot;

				self.scene_rot = 
					((scene_rot_x + rot_speed*dy) % TWOPI,
					 (scene_rot_y + rot_speed*dx) % TWOPI);
			}
		};


		let mut look_at = Iso3::<f32>::one();
		look_at.look_at_z(
			&Pnt3::new(0.0f32, 0.0f32, -2.0f32), 
			&Pnt3::new(0.0f32, 0.0f32, 0.0f32), 
			&Vec3::new(0.0f32, 1.0f32, 0.0f32));
		look_at = inv(&look_at).unwrap()
			.prepend_rotation(&(Vec3::new(1.0, 0.0, 0.0) * self.scene_rot.0 as f32))
			.prepend_rotation(&(Vec3::new(0.0, 1.0, 0.0) * self.scene_rot.1 as f32));

		if self.mode != TrackballCameraMode::Rotate
		{
			let inv_look_at = look_at.inv_transformation();
			let w_cam_right = inv_look_at.rotate(&camera_right);
			let w_cam_up = inv_look_at.rotate(&camera_up);
			let w_cam_front = inv_look_at.rotate(&camera_front);

			if self.mode == TrackballCameraMode::Pan {
				let pan_speed = 1.0;
				self.v_eye = self.v_eye + w_cam_right * (dx * pan_speed) as f32 + w_cam_up * (dy * pan_speed) as f32;
			} else {
				/*let scroll = (scrollOffsetY - lastWheelOffset) * sensitivity;
				let scroll_speed = 10.0;
				self.v_eye += (float)(scroll * scroll_speed) * wCamFront;*/
			};
		}

		self.last_mouse_pos = (pos_x, pos_y);

		Camera {
			view_matrix: look_at.prepend_translation(&self.v_eye).to_homogeneous(),
			proj_matrix: PerspMat3::<f32>::new(
				aspect_ratio as f32, self.settings.field_of_view as f32, 
				self.settings.near_plane as f32, self.settings.far_plane as f32).to_mat()
		}
	}
}