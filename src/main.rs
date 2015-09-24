#[macro_use]
extern crate glium;

mod teapot;

use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::fs::File;

fn load_file(path: &Path) -> io::Result<String> {
    // open file
    let mut file = try!(File::open(&path));
    
    // read the entire file
    let mut contents = String::new();
    try!(file.read_to_string(&mut contents));
    
    Ok(contents)
}

fn create_perspective_matrix(dimensions: (u32, u32)) -> [[f32; 4]; 4] {
    let (width, height) = dimensions;
    let aspect_ratio = height as f32 / width as f32;
    
    let fov = std::f32::consts::PI / 3.0;
    let z_far = 1024.0f32;
    let z_near = 0.1f32;
    
    let f = 1.0 / (fov / 2.0).tan();
    
    [
        [f * aspect_ratio   ,   0.0 ,                   0.0                     ,   0.0],
        [       0.0         ,   f   ,                   0.0                     ,   0.0],
        [       0.0         ,   0.0 ,   (z_far + z_near) / (z_far - z_near)     ,   1.0],
        [       0.0         ,   0.0 ,-(2.0 * z_far * z_near) / (z_far - z_near) ,   0.0],
    ]       
}

fn cross(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1],
    a[2] * b[0] - a[0] * b[2],
    a[0] * b[1] - a[1] * b[0]]
}

fn dot(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize(x: &[f32; 3]) -> [f32; 3] {
    let len = (x[0] * x[0] + x[1] * x[1] + x[2] * x[2]).sqrt();
    
    [x[0] / len, x[1] / len, x[2] / len]
}
 
fn create_view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
    let f = normalize(direction);
    let s = cross(up, &f);
    let s_norm = normalize(&s);
    
    let u = cross(&f, &s_norm);
    let p = [-dot(&position, &s_norm), -dot(&position, &u), -dot(&position, &f)];
    
    [
        [s[0], u[0], f[0], 0.0],
        [s[1], u[1], f[1], 0.0],
        [s[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}           

fn main() {
    // use glium display builder trait to create a window with openGL context
    // use Surface trait to get standard frame manipulation functionality
    use glium::{DisplayBuild, Surface};
    
    // create window
    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(640, 640)
        .with_title("13-blinn_phong".to_string())
        .with_vsync()
        .with_multisampling(16)
        .with_depth_buffer(24)
        .build_glium().unwrap();
    
    // create vertex and index buffers for shape
    let positions = glium::VertexBuffer::new(&display, &teapot::VERTICES).unwrap();
    let normals = glium::VertexBuffer::new(&display, &teapot::NORMALS).unwrap();
    let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList,
        &teapot::INDICES).unwrap();
    
    // load shaders from file
    let vertex_shader_src = load_file(&Path::new("shaders/vertex.glsl")).unwrap();
    let fragment_shader_src = load_file(&Path::new("shaders/fragment.glsl")).unwrap();

    // create glium program
    let program = glium::Program::from_source(&display, &vertex_shader_src, &fragment_shader_src, None).unwrap();
    
    // create draw parameter struct and enable depth testing
    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        // backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockWise,
        .. Default::default()
    };

    // direction of light source
    let light = [1.4, 0.4, -0.7f32];
    
    // create model transformation matrix
    let model = [
        [0.01, 0.0, 0.0, 0.0],
        [0.0, 0.01, 0.0, 0.0],
        [0.0, 0.0, 0.01, 0.0],
        [0.0, 0.0, 2.0, 1.0f32],
    ];
    
    // start event loop, exit loop when window closes
    let mut t = 0.0f32;
    loop {
        t += 1e-2;
        if t > 2.0 * std::f32::consts::PI {
            t -= 2.0 * std::f32::consts::PI;
        }

        // create view matrix
        let view = create_view_matrix(&[2.0 * t.sin(), 0.0, -2.0 * t.cos() + 2.0], &[-2.0 * t.sin(), 0.0, 2.0 * t.cos()], &[0.0, 1.0, 0.0]);
        
        // create transformation matrix
        // start drawing on the frame
        let mut target = display.draw();
        target.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);

        // create the perspective matrix
        let perspective = create_perspective_matrix(target.get_dimensions());

        // draw shape
        target.draw((&positions, &normals), &indices, &program,
            &uniform!{ perspective: perspective, view: view, model: model, u_light: light},
            &params).unwrap();
        
        // drawing is finished, so swap buffers
        target.finish().unwrap();
        
        // listen to the events produced by the window
        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => return,
                _ => ()
            }
        }
    }
}
