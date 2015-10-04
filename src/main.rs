#[macro_use]
extern crate glium;
extern crate nalgebra;
extern crate num;

mod numpy_compat;
mod mesh;

use numpy_compat::{load_txt, load_complex};
use num::complex::Complex;

fn load_measuerement(filename: &str) -> std::io::Result<Vec<Complex<f32>>> {
    // load only the first column of reconstruction from file
    let array: Vec<_> = try!(load_complex(filename)).iter()
        .map(|row| row[0]).collect();

    // calculate norms for real and imaginary parts
    let norm_real = (-array.iter().fold(0.0f32, |acc, val| acc.min(val.re - 1.0)))
        .max(array.iter().fold(0.0f32, |acc, val| acc.max(val.re - 1.0)));
    let norm_imag = (-array.iter().fold(0.0f32, |acc, val| acc.min(val.im)))
        .max(array.iter().fold(0.0f32, |acc, val| acc.max(val.im)));

    // norm values and substract reference value from real part
    Ok(array.iter().map(|val| {
        Complex::new((val.re - 1.0) / norm_real,
            val.im / norm_imag)
    }).collect::<Vec<_>>())
}

fn main() {
    // use glium display builder trait to create a window with openGL context
    // use Surface trait to get standard frame manipulation functionality
    use glium::{DisplayBuild, Surface};
    use glium::glutin::{ElementState, MouseButton};
    use glium::glutin::Event::{Closed, MouseInput, MouseMoved};
    use nalgebra::{PerspMat3, Iso3, Vec3, ToHomogeneous, Rot3};

    // create window
    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(800, 600)
        .with_title("mpflow-plotting".to_string())
        .with_vsync()
        .with_multisampling(16)
        .with_depth_buffer(24)
        .build_glium().unwrap();

    // get path of data from command line
    let path = std::env::args().nth(1).unwrap();

    // load mesh from files
    let nodes: Vec<Vec<f32>> = load_txt(&format!("{}/mesh/nodes.txt", path))
        .ok().expect("Cannot open mesh nodes file!");
    let elements: Vec<Vec<i32>> = load_txt(&format!("{}/mesh/elements.txt", path))
        .ok().expect("Cannot open mesh elements file!");

    // load mesh and reconstruction from file
    let reconstruction: Vec<f32> = load_measuerement(&format!("{}/reconstruction.txt", path))
        .ok().expect("Cannot load reconstruction from file!")
        .iter().map(|v| v.re).collect();

    // generate mesh
    let front_faces = mesh::generate_mesh(&display, &nodes, &elements, &reconstruction, true);
    let back_faces = mesh::generate_mesh(&display, &nodes, &elements, &reconstruction, false);

    // create shadow map
    let shadow_color_texture = glium::texture::Texture2d::empty(&display, 2048, 2048).unwrap();
    let shadow_texture = glium::texture::DepthTexture2d::empty_with_format(&display,
        glium::texture::DepthFormat::F32, glium::texture::MipmapsOption::NoMipmap, 2048, 2048).unwrap();
    let mut shadow_buffer = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(&display,
        &shadow_color_texture, &shadow_texture).unwrap();

    // create glium program
    let program = glium::Program::from_source(&display, include_str!("shaders/vertex.glsl"),
        include_str!("shaders/fragment.glsl"), None).unwrap();
    let shadow_program = glium::Program::from_source(&display, include_str!("shaders/vertex_shadow.glsl"),
        include_str!("shaders/fragment_shadow.glsl"), None).unwrap();

    // create draw parameter struct and enable depth testing
    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockWise,
        .. Default::default()
    };

    // position of light source
    let light_pos = [0.0, 2.0, 3.0f32];

    // create transformation matrices to render shadow map from lights point of view
    let shadow_perspective = PerspMat3::new(1.0, std::f32::consts::PI / 3.0, 0.1, 10.0);
    let shadow_view = (Rot3::new(Vec3::new(-std::f32::consts::PI / 4.0, 0.0, 0.0))).to_homogeneous() *
        Iso3::new(-Vec3::new(light_pos[0], light_pos[1], light_pos[2]), Vec3::new(0.0, 0.0, 0.0)).to_homogeneous();

    // start event loop, exit loop when window closes
    let mut mouse_pressed = false;
    let mut mouse_pos = (0.0f32, 0.0f32);
    let mut old_mouse_pos = (0.0f32, 0.0f32);
    let mut rot_angle = (0.0f32, 0.0f32);
    loop {
        // get current dimensions of main framebuffer
        let (width, height) = display.get_framebuffer_dimensions();

        // create view and perspective matrix
        let perspective = PerspMat3::new(width as f32 / height as f32, std::f32::consts::PI / 6.0, 0.1, 10.0);
        let view = Iso3::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0)).to_homogeneous();

        // rotate model according to mouse rotation
        let model = Iso3::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, 0.0)).to_homogeneous() *
            (Rot3::new(Vec3::new(2.0 * std::f32::consts::PI * rot_angle.1, 0.0, 0.0))).to_homogeneous() *
            (Rot3::new(Vec3::new(0.0, 0.0, -2.0 * std::f32::consts::PI * rot_angle.0))).to_homogeneous();

        // create uniforms
        let uniforms = uniform!{ light_pos: light_pos,
            perspective: perspective, view: view, model: model,
            shadow_perspective: shadow_perspective, shadow_view: shadow_view,
            shadow_map: shadow_texture.sampled()
                .minify_filter(glium::uniforms::MinifySamplerFilter::Linear)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
                .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp),
            };
        let shadow_uniforms = uniform!{ perspective: shadow_perspective, view: shadow_view, model: model }; 

        // render shadow map
        shadow_buffer.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);

        shadow_buffer.draw(&front_faces, &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
            &shadow_program, &shadow_uniforms, &params).unwrap();
        shadow_buffer.draw(&back_faces, &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
            &shadow_program, &shadow_uniforms, &params).unwrap();

        // start drawing on the frame
        let mut target = display.draw();
        target.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);

        // draw mesh
        target.draw(&front_faces, &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList), &program,
            &uniforms, &params).unwrap();
        target.draw(&back_faces, &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList), &program,
            &uniforms, &params).unwrap();

        // drawing is finished, so swap buffers
        target.finish().unwrap();

        // listen to the events produced by the window
        for ev in display.poll_events() {
            // process event
            match ev {
                Closed => return,
                MouseInput(ElementState::Pressed, MouseButton::Left) => {
                    mouse_pressed = true;
                    old_mouse_pos = mouse_pos;
                },
                MouseInput(ElementState::Released, MouseButton::Left) => mouse_pressed = false,
                MouseMoved((x, y)) => {
                    mouse_pos = (
                        2.0f32 * (x - width as i32 / 2) as f32 / height as f32,
                        2.0f32 * (height as i32 / 2 - y) as f32 / height as f32);

                    if mouse_pressed {
                        rot_angle = (rot_angle.0 + mouse_pos.0 - old_mouse_pos.0,
                            rot_angle.1 + mouse_pos.1 - old_mouse_pos.1);
                        old_mouse_pos = mouse_pos;
                    }
                },
                _ => ()
            }
        }
    }
}
