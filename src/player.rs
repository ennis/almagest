use terrain::*;
use nalgebra::*;
use event::*;
use glfw;
use camera::*;
use std;
use num::traits::{One};


// TODO deduplicate, this is pretty much the same as TrackballCameraSettings
pub struct PlayerCameraSettings
{
    pub field_of_view: f64,
    pub near_plane: f64,
    pub far_plane: f64,
    pub sensitivity: f64
}

pub struct PlayerCamera
{
    settings: PlayerCameraSettings,
    pos: Vec2<f32>,
    eye_height: f32,
    // theta, phi (inclination, azimuth)
	view_theta: f64,
    view_phi: f64
}

fn get_eye_dir(view_theta: f64, view_phi: f64) -> Vec3<f32>
{
    let st = view_theta.sin();
    let ct = view_theta.cos();
    let sp = view_phi.sin();
    let cp = view_phi.cos();
    Vec3::new(
        (st * cp) as f32,
        ct as f32,
        (st * sp) as f32)
}

impl PlayerCamera
{
    pub fn new(settings: PlayerCameraSettings) -> PlayerCamera
    {
        PlayerCamera {
            settings: settings,
            pos: Vec2::new(0.0, 0.0),
            eye_height: 0.2,
            view_theta: std::f64::consts::PI/2.0,
            view_phi: 0.0
        }
    }

    // TODO view bobbing, frame event
    pub fn event(&mut self, event: &Event)
    {
        // handle mouse move and WASD keys
        // TODO should not move faster on sloped terrain
        let w_eye_dir = get_eye_dir(self.view_theta, self.view_phi);
        // dummy look at without the height
        let look_at = Rot3::look_at_z(&w_eye_dir, &Vec3::new(0.0f32, 1.0f32, 0.0f32));
        // TODO should not move faster on sloped terrain
        let w_strafe_right = look_at.rotate(&Vec3::new(1.0f32, 0.0f32, 0.0f32));
        let front = Vec2::new(w_eye_dir.x, w_eye_dir.z).normalize();
        let right = Vec2::new(w_strafe_right.x, w_strafe_right.z).normalize();
        let speed = 0.1f32;

        match event {
            &Event::KeyDown(k) => {
                match k {
                    glfw::Key::W => self.pos = self.pos + front * speed,
                    glfw::Key::S => self.pos = self.pos - front * speed,
                    glfw::Key::A => self.pos = self.pos + right * speed,
                    glfw::Key::D => self.pos = self.pos - right * speed,
                    _ => {}
                }
            },
            &Event::MouseMove(raw_dx, raw_dy) => {
				let dx = self.settings.sensitivity * raw_dx;
				let dy = self.settings.sensitivity * raw_dy;
                let pi = std::f64::consts::PI;
				let twopi = 2.0 * pi;
                self.view_phi = (self.view_phi + dx) % twopi;
                self.view_theta = clamp(self.view_theta + dy, 0.1, pi-0.1);
			},
            _ => {}
        }
    }

    pub fn get_camera(&self, terrain: &Terrain, window: &glfw::Window) -> Camera
	{
		let (width, height) = window.get_size();
		let aspect_ratio = if height != 0 { width as f64 / height as f64 } else { 1.0 };

        // TODO add terrain height
        let w_eye_dir = get_eye_dir(self.view_theta, self.view_phi);
        let eye_pos = Pnt3::new(self.pos.x, terrain.sample_height(self.pos.x as f64, self.pos.y as f64) as f32 + self.eye_height, self.pos.y);
        let mut look_at = Iso3::one();
        look_at.look_at_z(
			&eye_pos,
			&(eye_pos + w_eye_dir),
			&Vec3::new(0.0f32, 1.0f32, 0.0f32));
		look_at = inv(&look_at).unwrap();

		Camera {
			view_matrix: look_at.to_homogeneous(),
			proj_matrix: PerspMat3::new(
				aspect_ratio as f32, self.settings.field_of_view as f32,
				self.settings.near_plane as f32, self.settings.far_plane as f32).to_mat()
		}
	}
}
