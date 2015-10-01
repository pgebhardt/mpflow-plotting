#[macro_use]
extern crate glium;
extern crate nalgebra;
extern crate num;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use num::traits::Float;

// vertex format
#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    value: f32,
}

fn load_file<T: FromStr>(filename: &str) -> Vec<Vec<T>> {
    // open file
    let f = BufReader::new(File::open(filename).unwrap());

    // read something
    let arr: Vec<Vec<T>> = f.lines()
        .map(|l| l.unwrap().split(char::is_whitespace)
            .map(|number| number.parse().ok().unwrap())
            .collect())
        .collect();

    arr
}

fn load_measurement(filename: &str) -> Vec<Vec<f32>> {
    // open file
    let f = BufReader::new(File::open(filename).unwrap());

    // read something
    let mut arr: Vec<Vec<f32>> = f.lines()
        .map(|l| {
            let line = l.unwrap();

            if &line[0..1] == "(" {
                return (&line[1..line.len()-1]).to_string().split(',').map(|number| number.parse().ok().unwrap()).collect(); 
            }
            else {
                return vec![line.parse().ok().unwrap()];
            }
        })
        .collect();

    // correct real part
    let norm_real = (-arr.iter().fold(0.0f32, |acc, ref item| acc.min(item[0] - 1.0)))
        .max(arr.iter().fold(0.0f32, |acc, ref item| acc.max(item[0] - 1.0)));
    for v in arr.iter_mut() {
        v[0] = (v[0] - 1.0) / norm_real;
    }

    // correct imaginary part
    if arr[0].len() > 1 {
        let norm_imag = (-arr.iter().fold(0.0f32, |acc, ref item| acc.min(item[1])))
            .max(arr.iter().fold(0.0f32, |acc, ref item| acc.max(item[1])));
        for v in arr.iter_mut() {
            v[1] = v[1] / norm_imag;
        }
    }

    arr
}

fn calculate_z_values(nodes: &Vec<Vec<f32>>, elements: &Vec<Vec<i32>>, values: &Vec<f32>) -> Vec<f32> {
    // calculate node and element area
    let mut element_area = vec![0.0f32; elements.len()];
    let mut node_area = vec![0.0f32; nodes.len()];

    for (i, elem) in elements.iter().enumerate() {
        element_area[i] = 0.5 * (
            (nodes[elem[1] as usize][0] - nodes[elem[0] as usize][0]) *
            (nodes[elem[2] as usize][1] - nodes[elem[0] as usize][1]) -
            (nodes[elem[2] as usize][0] - nodes[elem[0] as usize][0]) *
            (nodes[elem[1] as usize][1] - nodes[elem[0] as usize][1])).abs();

        for n in elem.iter() {
            node_area[*n as usize] += element_area[i];
        }
    }

    // calculate z values
    let mut z_values = vec![0.0f32; nodes.len()];
    for (i, elem) in elements.iter().enumerate() {
        for n in elem.iter() {
            z_values[*n as usize] +=
                values[i] * element_area[i] / node_area[*n as usize];
        }
    }

    z_values
}

fn calculate_normal(p1: &[f32; 3], p2: &[f32; 3], p3: &[f32; 3]) -> [f32; 3] {
    let u = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];
    let v = [p3[0] - p1[0], p3[1] - p1[1], p3[2] - p1[2]];

    [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ]
}

fn main() {
    // use glium display builder trait to create a window with openGL context
    // use Surface trait to get standard frame manipulation functionality
    use glium::{DisplayBuild, Surface};
    use glium::glutin::{ElementState, MouseButton};
    use glium::glutin::Event::{Closed, MouseInput, MouseMoved};
    use nalgebra::{PerspMat3, Iso3, Vec3, ToHomogeneous, Rot3};

    // make vertex format available to glium
    implement_vertex!(Vertex, position, normal, value);

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

    // generate mesh
    let mesh = {
        // load mesh and reconstruction from file
        let nodes: Vec<Vec<f32>> = load_file(&format!("{}/mesh/nodes.txt", path));
        let elements: Vec<Vec<i32>> = load_file(&format!("{}/mesh/elements.txt", path));
        let reconstruction: Vec<f32> = load_measurement(&format!("{}/reconstruction.txt", path))
            .iter().map(|v| v[0]).collect();

        // create interpolated z values
        let z_values = calculate_z_values(&nodes, &elements, &reconstruction);

        // calculate radius of mesh to scale each vertex to unit size
        let radius = nodes.iter().fold(0.0f32, |acc, item| (item[0] * item[0] + item[1] * item[1]).sqrt().max(acc));

        // fill vertex buffer
        let mut vertex_data = Vec::new();
        for (i, shape) in elements.iter().enumerate() {
            let triangle = [
                [nodes[shape[0] as usize][0] / radius, nodes[shape[0] as usize][1] / radius, -z_values[shape[0] as usize]],
                [nodes[shape[1] as usize][0] / radius, nodes[shape[1] as usize][1] / radius, -z_values[shape[1] as usize]],
                [nodes[shape[2] as usize][0] / radius, nodes[shape[2] as usize][1] / radius, -z_values[shape[2] as usize]],
                ];

            // set front faces
            let normal = calculate_normal(&triangle[0], &triangle[1], &triangle[2]);
            for j in 0..3 {
                if normal[2] <= 0.0 {
                    vertex_data.push(Vertex {
                        position: triangle[j],
                        normal: normal,
                        value: reconstruction[i],
                    });
                }
                else {
                    vertex_data.push(Vertex {
                        position: triangle[3 - j - 1],
                        normal: [-normal[0], -normal[1], -normal[2]],
                        value: reconstruction[i],
                    });
                }
            }

            // set back faces
            for j in 0..3 {
                if normal[2] <= 0.0 {
                    vertex_data.push(Vertex {
                        position: triangle[3 - j - 1],
                        normal: [-normal[0], -normal[1], -normal[2]],
                        value: reconstruction[i],
                    });
                }
                else {
                    vertex_data.push(Vertex {
                        position: triangle[j],
                        normal: normal,
                        value: reconstruction[i],
                    });
                }
            }
        }

        // load data to gpu
        glium::vertex::VertexBuffer::new(&display, &vertex_data).unwrap()
    };

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
        // listen to the events produced by the window
        for ev in display.wait_events() {
            // get current dimensions of main framebuffer
            let (width, height) = display.get_framebuffer_dimensions();

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

            // rotate model according to mouse rotation
            let model = Iso3::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, 0.0)).to_homogeneous() *
                (Rot3::new(Vec3::new(2.0 * std::f32::consts::PI * rot_angle.1, 0.0, 0.0))).to_homogeneous() *
                (Rot3::new(Vec3::new(0.0, 0.0, -2.0 * std::f32::consts::PI * rot_angle.0))).to_homogeneous();

            // rander shadow map
            shadow_buffer.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);

            let shadow_uniforms = uniform!{ perspective: shadow_perspective, view: shadow_view, model: model }; 
            shadow_buffer.draw(&mesh, &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                &shadow_program, &shadow_uniforms, &params).unwrap();

            // start drawing on the frame
            let mut target = display.draw();
            target.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);

            // create view and perspective matrix
            let perspective = PerspMat3::new(width as f32 / height as f32, std::f32::consts::PI / 6.0, 0.1, 10.0);
            let view = Iso3::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0)).to_homogeneous();

            // draw shape
            let uniforms = uniform!{ light_pos: light_pos,
                perspective: perspective, view: view, model: model,
                shadow_perspective: shadow_perspective, shadow_view: shadow_view,
                shadow_map: shadow_texture.sampled()
                    .minify_filter(glium::uniforms::MinifySamplerFilter::Linear)
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
                    .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp),
                };

            target.draw(&mesh, &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList), &program,
                &uniforms, &params).unwrap();

            // drawing is finished, so swap buffers
            target.finish().unwrap();
        }
    }
}
