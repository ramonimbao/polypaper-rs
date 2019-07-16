use cgmath::prelude::*;
use cgmath::Vector3;
use chrono::Local;
use quicksilver::geom::{Triangle, Vector};
use quicksilver::graphics::{Background::Col, Color, PixelFormat};
use quicksilver::input::{ButtonState, Key};
use quicksilver::lifecycle::{run, Event, Settings, State, Window};
use quicksilver::Result;
use rand::prelude::*;

const WIDTH: f32 = 1920.0;
const HEIGHT: f32 = 1080.0;
const NUM_LIGHTS: usize = 2;
const Z_OFFSET: f32 = 100.0;


struct Tri {
    vertices: (Vector3<f32>, Vector3<f32>, Vector3<f32>),
    centroid: Vector3<f32>,
    normal: Vector3<f32>,
    color: Vector3<f32>,
}

struct Light {
    position: Vector3<f32>,
    ray: Vector3<f32>,
    ambient: Vector3<f32>,
    diffuse: Vector3<f32>,
}

struct Polypaper {
    triangles: Vec<Tri>,
}

fn generate_triangle(v0: Vector3<f32>, v1: Vector3<f32>, v2: Vector3<f32>) -> Tri {
    // Get centroid
    let centroid = Vector3::new(
        (v0.x + v1.x + v2.x) / 3.0,
        (v0.y + v1.y + v2.z) / 3.0,
        (v0.z + v1.z + v2.z) / 3.0,
    );

    // Get normal
    let u = v1 - v0;
    let v = v2 - v0;
    let normal = (u.cross(v)).normalize();

    let color = Vector3::new(0.0, 0.0, 0.0);

    Tri {
        vertices: (v0, v1, v2),
        centroid,
        normal,
        color,
    }
}

fn vec3_component_multiply(a: &Vector3<f32>, b: &Vector3<f32>) -> Vector3<f32> {
    Vector3::new(a.x * b.x, a.y * b.y, a.z * b.z)
}

fn clamp(v: &mut f32, min: f32, max: f32) {
    if *v > max {
        *v = max;
    }
    if *v < min {
        *v = min;
    }
}

fn generate_mesh() -> Vec<Tri> {
    let mut rng = thread_rng();

    let variation = (WIDTH / 32.0).min(HEIGHT / 16.0);

    // Generate vertices
    println!("Generating vertices...");
    let mut vertices = Vec::new();
    for y in 0..=10 {
        for x in 0..=18 {
            vertices.push(Vector3::new(
                (WIDTH / 16.0) * x as f32 + rng.gen_range(-variation, variation) - WIDTH / 16.0,
                (HEIGHT / 8.0) * y as f32 + rng.gen_range(-variation, variation) - HEIGHT / 8.0,
                rng.gen_range(-50.0, 50.0),
            ));
        }
    }

    // Generate triangles
    println!("Generating traingles...");
    let mut triangles = Vec::new();
    for j in (0..=171).step_by(19) {
        for i in 0..18 {
            triangles.push(generate_triangle(
                vertices[j + i],
                vertices[j + i + 1],
                vertices[j + i + 19],
            ));
            triangles.push(generate_triangle(
                vertices[j + i + 1],
                vertices[j + i + 19],
                vertices[j + i + 20],
            ));
        }
    }
    // Position the lights
    println!("Positioning the lights...");
    let mut lights = Vec::new();
    for _ in 0..NUM_LIGHTS {
        lights.push(Light {
            position: Vector3::new(
                rng.gen_range(-50.0, WIDTH + 50.0),
                rng.gen_range(-50.0, HEIGHT + 50.0),
                rng.gen_range(Z_OFFSET / 2.0, Z_OFFSET * 2.0),
            ),
            ray: Vector3::new(0.0, 0.0, 0.0),
            ambient: Vector3::new(rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>()),
            diffuse: Vector3::new(rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>()),
        });
    }

    // Compute the illuminance of the triangles
    let mesh_ambient = Vector3::new(rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>());
    //let light_ambient = Vector3::new(rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>());
    let mesh_diffuse = Vector3::new(rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>());
    //let light_diffuse = Vector3::new(1.0, 1.0, 1.0);

    for triangle in triangles.iter_mut() {
        triangle.color = Vector3::new(0.0, 0.0, 0.0);
        for light in lights.iter_mut() {
            light.ray = (light.position - triangle.centroid).normalize();
            let illuminance = triangle.normal.dot(light.ray);

            // Calculate ambient
            // In glm, vec3 * vec3 is a component-wise multiplication
            // I don't know if such exists in cgmath, but it's easy enough to do manually
            let ambient = vec3_component_multiply(&mesh_ambient, &light.ambient);
            triangle.color += ambient / NUM_LIGHTS as f32;
            // Calculate diffuse
            let diffuse = vec3_component_multiply(&mesh_diffuse, &light.diffuse);
            triangle.color += (diffuse * illuminance) / NUM_LIGHTS as f32;
        }

        // Clamp the color value.
        // cgmath's clamp() is a nightly feature for some reason, so let's just do it manually.
        clamp(&mut triangle.color.x, 0.0, 1.0);
        clamp(&mut triangle.color.y, 0.0, 1.0);
        clamp(&mut triangle.color.z, 0.0, 1.0);
    }
    triangles
}

impl State for Polypaper {
    fn new() -> Result<Polypaper> {
        Ok(Polypaper {
            triangles: generate_mesh(),
        })
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
        if let Event::Key(key, ButtonState::Pressed) = event {
            match key {
                Key::Space => {
                    self.triangles = generate_mesh();
                }
                Key::Return => {
                    let now = Local::now();
                    println!("Saving...");
                    window
                        .screenshot(PixelFormat::RGBA)
                        .save(format!("output_{}.png", now.format("%F_%H-%M-%S")))?;
                    println!("...done!");
                    self.triangles = generate_mesh(); // The screen change would let you know it's done
                }
                Key::Escape => {
                    window.close();
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;

        for triangle in &self.triangles {
            window.draw(
                &Triangle::new(
                    (triangle.vertices.0.x, triangle.vertices.0.y),
                    (triangle.vertices.1.x, triangle.vertices.1.y),
                    (triangle.vertices.2.x, triangle.vertices.2.y),
                ),
                Col(Color {
                    r: triangle.color.x,
                    g: triangle.color.y,
                    b: triangle.color.z,
                    a: 1.0,
                }),
            );
        }

        Ok(())
    }
}

fn main() {
    run::<Polypaper>(
        "Polypaper",
        Vector::new(WIDTH, HEIGHT),
        Settings {
            fullscreen: true,
            vsync: true,
            multisampling: Some(16),
            ..Default::default()
        },
    );
}
