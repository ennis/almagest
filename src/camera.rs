use event::*;
use nalgebra::*;
use glfw;
use std;
use num::traits::{Zero, One};
use window::Window;

#[derive(Copy, Clone)]
pub struct Camera
{
	pub view_matrix: Mat4<f32>,
	pub proj_matrix: Mat4<f32>,
	pub w_eye: Pnt3<f32>
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
		let mut tmp = TrackballCameraController {
			settings: *self,
			v_eye: self.initial_eye.to_vec(),
			scene_rot: (0.0, 0.0),
			panning: false,
			rotating: false,
			w_cam_up: Vec3::zero(),
			w_cam_right: Vec3::zero(),
			w_cam_front: Vec3::zero()
		};
		tmp.update_pan_vectors();
		tmp
	}
}

// Should take as input an 'event' struct
pub struct TrackballCameraController
{
	settings: TrackballCameraSettings,
	v_eye: Vec3<f32>,
	scene_rot: (f64, f64),
	panning: bool,
	rotating: bool,
	w_cam_up: Vec3<f32>,
	w_cam_right: Vec3<f32>,
	w_cam_front: Vec3<f32>
}

impl TrackballCameraController
{
	fn get_look_at(&self) -> Iso3<f32>
	{
		let mut look_at = Iso3::one();
		look_at.look_at_z(
			&Pnt3::new(0.0f32, 0.0f32, -2.0f32),
			&Pnt3::new(0.0f32, 0.0f32, 0.0f32),
			&Vec3::new(0.0f32, 1.0f32, 0.0f32));
		look_at = inv(&look_at).unwrap()
			.prepend_rotation(&(Vec3::new(1.0, 0.0, 0.0) * self.scene_rot.0 as f32))
			.prepend_rotation(&(Vec3::new(0.0, 1.0, 0.0) * self.scene_rot.1 as f32));
		look_at
	}

	fn update_pan_vectors(&mut self)
	{
		let camera_up = Vec3::new(0.0f32, 1.0f32, 0.0f32);
		let camera_right = Vec3::new(1.0f32, 0.0f32, 0.0f32);
		let camera_front = Vec3::new(0.0f32, 0.0f32, 1.0f32);

		// update view vectors
		let inv_look_at = self.get_look_at().inv_transformation();
		self.w_cam_right = inv_look_at.rotate(&camera_right);
		self.w_cam_up = inv_look_at.rotate(&camera_up);
		self.w_cam_front = inv_look_at.rotate(&camera_front);
	}

	pub fn event(&mut self, event: &Event)
	{
		// TODO constants
		let camera_up = Vec3::new(0.0f32, 1.0f32, 0.0f32);
		let camera_right = Vec3::new(1.0f32, 0.0f32, 0.0f32);
		let camera_front = Vec3::new(0.0f32, 0.0f32, 1.0f32);

		match event {
			&Event::MouseButton(button, action) => {
				match button {
					glfw::MouseButton::Button1 => {
						match action {
							glfw::Action::Press => self.rotating = true,
							glfw::Action::Release => self.rotating = false,
							_ => {}
						}
					},
					glfw::MouseButton::Button3 => {
						match action {
							glfw::Action::Press => self.panning = true,
							glfw::Action::Release => self.panning = false,
							_ => {}
						}
					},
					_ => {}
				}
			},
			// reset camera
			&Event::KeyDown(glfw::Key::R) => {
				*self = self.settings.build();
			},
			// move viewpoint
			&Event::MouseMove(raw_dx, raw_dy) => {
				let dx = self.settings.sensitivity * raw_dx;
				let dy = self.settings.sensitivity * raw_dy;
				if self.rotating {
					let rot_speed = 0.1f64;
					let twopi = 2.0 * std::f64::consts::PI;
					let (scene_rot_x, scene_rot_y) = self.scene_rot;
					self.scene_rot =
						((scene_rot_x + rot_speed*dy) % twopi,
						(scene_rot_y + rot_speed*dx) % twopi);
					// update the panning directions after changing the rotation values
					self.update_pan_vectors();
				}

				if self.panning && !self.rotating {
					let pan_speed = 1.0;
					self.v_eye = self.v_eye + self.w_cam_right * (dx * pan_speed) as f32 + self.w_cam_up * (dy * pan_speed) as f32;
				}
			},
			// zoom (move along camera front direction)
			&Event::MouseWheel(delta) => {
				let scroll = delta * self.settings.sensitivity;
				let scroll_speed = 10.0;
				self.v_eye =  self.v_eye + self.w_cam_front * (scroll * scroll_speed) as f32;
			},
			_ => {}
		}
	}

	pub fn get_camera(&self, window: &Window) -> Camera
	{
		let (width, height) = window.dimensions();
		let aspect_ratio = if height != 0 { width as f64 / height as f64 } else { 1.0 };

		Camera {
			view_matrix: self.get_look_at().prepend_translation(&self.v_eye).to_homogeneous(),
			proj_matrix: PerspMat3::new(
				aspect_ratio as f32, self.settings.field_of_view as f32,
				self.settings.near_plane as f32, self.settings.far_plane as f32).to_mat(),
			w_eye: Pnt3::new(0.0f32, 0.0f32, 0.0f32) + self.v_eye	// XXX this is wrong
		}
	}
}
