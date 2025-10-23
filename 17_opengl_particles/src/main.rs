use glfw::{Action, Context, Key};
use std::env;
use std::f32::consts::PI;

use gl::types::*;
use nalgebra::{Matrix4, Point3, Vector3};

#[derive(Debug, serde::Deserialize)]
struct Point {
    x: f32,
    y: f32,
    z: f32,
}

fn read_csv(path: &str) -> Vec<Point> {
    let mut rdr = csv::Reader::from_path(path).expect("Failed to open CSV");
    rdr.deserialize()
        .map(|r| r.expect("Bad CSV row"))
        .collect()
}

fn draw_bounding_cube(x_min: f32, x_max: f32, y_min: f32, y_max: f32, z_min: f32, z_max: f32) {
    unsafe {
        gl::Color3f(1.0, 1.0, 1.0);
        gl::Begin(gl::LINES);

        let verts = [
            (x_min, y_min, z_min, x_max, y_min, z_min),
            (x_max, y_min, z_min, x_max, y_min, z_max),
            (x_max, y_min, z_max, x_min, y_min, z_max),
            (x_min, y_min, z_max, x_min, y_min, z_min),
            (x_min, y_max, z_min, x_max, y_max, z_min),
            (x_max, y_max, z_min, x_max, y_max, z_max),
            (x_max, y_max, z_max, x_min, y_max, z_max),
            (x_min, y_max, z_max, x_min, y_max, z_min),
            (x_min, y_min, z_min, x_min, y_max, z_min),
            (x_max, y_min, z_min, x_max, y_max, z_min),
            (x_max, y_min, z_max, x_max, y_max, z_max),
            (x_min, y_min, z_max, x_min, y_max, z_max),
        ];

        for &(x1, y1, z1, x2, y2, z2) in &verts {
            gl::Vertex3f(x1, y1, z1);
            gl::Vertex3f(x2, y2, z2);
        }

        gl::End();
    }
}

fn draw_sphere(x: f32, y: f32, z: f32, radius: f32) {
    unsafe {
        gl::PushMatrix();
        gl::Translatef(x, y, z);

        // simple sphere approximation using glut-like parametric bands
        let stacks = 16;
        let slices = 16;
        for i in 0..stacks {
            let lat0 = PI * (-0.5 + (i as f32) / (stacks as f32));
            let z0 = radius * lat0.sin();
            let zr0 = radius * lat0.cos();

            let lat1 = PI * (-0.5 + ((i + 1) as f32) / (stacks as f32));
            let z1 = radius * lat1.sin();
            let zr1 = radius * lat1.cos();

            gl::Begin(gl::QUAD_STRIP);
            for j in 0..=slices {
                let lng = 2.0 * PI * (j as f32) / (slices as f32);
                let x = lng.cos();
                let y = lng.sin();

                gl::Normal3f(x * zr0, y * zr0, z0);
                gl::Vertex3f(x * zr0, y * zr0, z0);
                gl::Normal3f(x * zr1, y * zr1, z1);
                gl::Vertex3f(x * zr1, y * zr1, z1);
            }
            gl::End();
        }

        gl::PopMatrix();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <csv_file>", args[0]);
        std::process::exit(1);
    }
    let data = read_csv(&args[1]);

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    let (mut window, events) = glfw
        .create_window(800, 600, "Rotating 3D Spheres", glfw::WindowMode::Windowed)
        .expect("Failed to create window");
    window.make_current();
    window.set_key_polling(true);

    gl::load_with(|s| window.get_proc_address(s) as *const _);

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
    }

    let (x_min, x_max) = data.iter().map(|p| p.x).fold((f32::MAX, f32::MIN), |(a, b), v| (a.min(v), b.max(v)));
    let (y_min, y_max) = data.iter().map(|p| p.y).fold((f32::MAX, f32::MIN), |(a, b), v| (a.min(v), b.max(v)));
    let (z_min, z_max) = data.iter().map(|p| p.z).fold((f32::MAX, f32::MIN), |(a, b), v| (a.min(v), b.max(v)));
    let (xc, yc, zc) = ((x_min + x_max)/2.0, (y_min + y_max)/2.0, (z_min + z_max)/2.0);

    let mut angle = 0.0_f32;

    while !window.should_close() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::MatrixMode(gl::MODELVIEW);
            gl::LoadIdentity();

            // Simple camera
            gl::Translatef(0.0, 0.0, -200.0);
            gl::Rotatef(angle, 0.0, 1.0, 0.0);

            draw_bounding_cube(x_min, x_max, y_min, y_max, z_min, z_max);

            gl::Color3f(0.0, 0.6, 1.0);
            for p in &data {
                draw_sphere(p.x - xc, p.y - yc, p.z - zc, 1.0);
            }
        }

        angle += 0.5;
        if angle > 360.0 {
            angle -= 360.0;
        }

        window.swap_buffers();
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            if let glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) = event {
                window.set_should_close(true);
            }
        }
    }
}
