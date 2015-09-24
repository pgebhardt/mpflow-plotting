#[macro_use]
extern crate glium;
extern crate nalgebra as na;
use na::{PerspMat3, Iso3, Mat3, Pnt3, Vec3, ToHomogeneous, Eye};

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

fn main() {
    // use glium display builder trait to create a window with openGL context
    // use Surface trait to get standard frame manipulation functionality
    use glium::{DisplayBuild, Surface};
    
    // create window
    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(640, 480)
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
    
    // start event loop, exit loop when window closes
    let mut t = 0.0f32;
    loop {
        t += 1e-2;
        if t > 2.0 * std::f32::consts::PI {
            t -= 2.0 * std::f32::consts::PI;
        }

        // create view matrix
        let mut view: Iso3<f32> = na::one();
        view.look_at_z(&Pnt3::new(0.0, 0.0, 0.0), &Pnt3::new(0.0, 0.0, 1.0), &Vec3::new(0.0, 1.0, 0.0));
        let view = view.to_homogeneous();
        
        // create model transformation matrix
        let model = Iso3::new(Vec3::new(0.0, 0.0, 2.0), Vec3::new(0.0, t, 0.0)).to_homogeneous() *
            (Mat3::new_identity(3) * 0.01).to_homogeneous();
        
        // start drawing on the frame
        let mut target = display.draw();
        target.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);

        // create the perspective matrix
        let (width, height) = target.get_dimensions();
        let perspective = PerspMat3::new(width as f32 / height as f32, std::f32::consts::PI / 3.0, 0.1, 1024.0);
        
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
