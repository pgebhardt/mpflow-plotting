#[macro_use]
extern crate glium;
extern crate nalgebra;
extern crate obj;
extern crate genmesh;

// vertex format
#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 3]
}
    
fn load_wavefront(display: &glium::Display, data: &[u8]) -> glium::vertex::VertexBufferAny {
    // load wavefrom from data
    let mut data = std::io::BufReader::new(data);
    let data = obj::Obj::load(&mut data);
    
    // extract all vertices from model
    let mut vertex_data = Vec::new();
    for shape in data.object_iter().next().unwrap().group_iter().flat_map(|g| g.indices.iter()) {
        match shape {
            &genmesh::Polygon::PolyTri(genmesh::Triangle { x: v1, y: v2, z: v3 }) => {
                for v in [v1, v2, v3].iter() {
                    let position = data.position()[v.0];
                    let normal = v.2.map(|index| data.normal()[index]);
                    
                    vertex_data.push(Vertex {
                        position: position,
                        normal: normal.unwrap_or([0.0, 0.0, 0.0]),
                        color: [0.6, 0.0, 0.0]
                    })
                }
            },
            _ => unimplemented!()
        }
    }
    
    glium::vertex::VertexBuffer::new(display, &vertex_data).unwrap().into_vertex_buffer_any()
}

fn main() {
    // use glium display builder trait to create a window with openGL context
    // use Surface trait to get standard frame manipulation functionality
    use glium::{DisplayBuild, Surface};
    use glium::glutin::{ElementState, MouseButton};
    use glium::glutin::Event::{Closed, MouseInput, MouseMoved};
    use nalgebra::{PerspMat3, Iso3, Mat3, Vec3, ToHomogeneous, Eye, Rot3};

    // make vertex format available to glium
    implement_vertex!(Vertex, position, normal, color);
    
    // create window
    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(800, 600)
        .with_title("shadows".to_string())
        .with_vsync()
        .with_multisampling(16)
        .with_depth_buffer(24)
        .build_glium().unwrap();
    
    // generate some shapes
    let teapot = load_wavefront(&display, include_bytes!("obj/teapot.obj"));
    let table = glium::vertex::VertexBuffer::new(&display, &[
        Vertex { position: [-200.0, -50.0,  200.0], normal: [0.0, 1.0, 0.0], color: [0.6, 0.6, 0.6] },
        Vertex { position: [-200.0, -50.0, -200.0], normal: [0.0, 1.0, 0.0], color: [0.6, 0.6, 0.6] },
        Vertex { position: [ 200.0, -50.0,  200.0], normal: [0.0, 1.0, 0.0], color: [0.6, 0.6, 0.6] },
        Vertex { position: [ 200.0, -50.0, -200.0], normal: [0.0, 1.0, 0.0], color: [0.6, 0.6, 0.6] },
        ]).unwrap();
        
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
    let mut params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        // backface_culling: glium::draw_parameters::BackfaceCullingMode::CullCounterClockWise,
        .. Default::default()
    };

    // position of light source
    let light_pos = [0.0, 2.0, 2.0f32];
    
    // start event loop, exit loop when window closes
    let mut mouse_pressed = false;
    let mut mouse_pos = (0.0f32, 0.0f32);
    let mut old_mouse_pos = (0.0f32, 0.0f32);
    let mut rot_angle = (0.0f32, 0.0f32);
    loop {
        // create model transformation matrix
        let model = Iso3::new(Vec3::new(0.0, 0.0, 2.0), Vec3::new(0.0, 0.0, 0.0)).to_homogeneous() *
            (Mat3::new_identity(3) * 0.01).to_homogeneous() *
            (Rot3::new(Vec3::new(2.0 * std::f32::consts::PI * rot_angle.1, 0.0, 0.0))).to_homogeneous() *
            (Rot3::new(Vec3::new(0.0, 2.0 * std::f32::consts::PI * rot_angle.0, 0.0))).to_homogeneous();
        
        // rander shadow map
        shadow_buffer.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);

        let shadow_perspective = PerspMat3::new(1.0, std::f32::consts::PI / 1.5, 0.1, 10.0);
        let shadow_view = (Rot3::new(Vec3::new(-std::f32::consts::PI / 2.0, 0.0, 0.0))).to_homogeneous() *
            Iso3::new(-Vec3::new(light_pos[0], light_pos[1], light_pos[2]), Vec3::new(0.0, 0.0, 0.0)).to_homogeneous();
        let shadow_uniforms = uniform!{ perspective: shadow_perspective, view: shadow_view, model: model }; 

        params.backface_culling = glium::draw_parameters::BackfaceCullingMode::CullClockWise;
        shadow_buffer.draw(&teapot, &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
            &shadow_program, &shadow_uniforms, &params).unwrap();
        shadow_buffer.draw(&table, &glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip),
            &shadow_program, &shadow_uniforms, &params).unwrap();
        
        // start drawing on the frame
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

        // create the perspective matrix
        let (width, height) = target.get_dimensions();
        let perspective = PerspMat3::new(width as f32 / height as f32, std::f32::consts::PI / 3.0, 0.1, 10.0);
        
        // create view matrix
        let view = Iso3::new(Vec3::new(0.0, 0.0, 2.0), Vec3::new(0.0, 0.0, 0.0)).to_homogeneous();

        // draw shape
        let uniforms = uniform!{ light_pos: light_pos,
            perspective: perspective, view: view, model: model,
            shadow_perspective: shadow_perspective, shadow_view: shadow_view,
            shadow_map: shadow_texture.sampled()
                .minify_filter(glium::uniforms::MinifySamplerFilter::Linear)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
                .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp),
            };
            
        params.backface_culling = glium::draw_parameters::BackfaceCullingMode::CullingDisabled;
        target.draw(&teapot, &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList), &program,
            &uniforms, &params).unwrap();
        target.draw(&table, &glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip), &program,
            &uniforms, &params).unwrap();
            
        // drawing is finished, so swap buffers
        target.finish().unwrap();
        
        // listen to the events produced by the window
        for ev in display.poll_events() {
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
