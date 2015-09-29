#[macro_use]
extern crate glium;
extern crate nalgebra;

mod teapot;

fn main() {
    // use glium display builder trait to create a window with openGL context
    // use Surface trait to get standard frame manipulation functionality
    use glium::{DisplayBuild, Surface};
    use glium::glutin::{ElementState, MouseButton};
    use glium::glutin::Event::{Closed, MouseInput, MouseMoved};
    use nalgebra::{PerspMat3, Iso3, Mat3, Pnt3, Vec3, ToHomogeneous, Eye, Rot3};

    // create window
    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(640, 480)
        .with_title("shadows".to_string())
        .with_vsync()
        .with_multisampling(16)
        .with_depth_buffer(24)
        .build_glium().unwrap();
    
    // create vertex and index buffers for shape
    let positions = glium::VertexBuffer::new(&display, &teapot::VERTICES).unwrap();
    let normals = glium::VertexBuffer::new(&display, &teapot::NORMALS).unwrap();
    let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList,
        &teapot::INDICES).unwrap();
    
    // create some shape
    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 3],
        normal: [f32; 3]
    }
    implement_vertex!(Vertex, position, normal);
    
    let shape = glium::vertex::VertexBuffer::new(&display, &[
        Vertex { position: [-200.0, -50.0,  200.0], normal: [0.0, 1.0, 0.0] },
        Vertex { position: [ 200.0, -50.0,  200.0], normal: [0.0, 1.0, 0.0] },
        Vertex { position: [-200.0, -50.0, -200.0], normal: [0.0, 1.0, 0.0] },
        Vertex { position: [ 200.0, -50.0, -200.0], normal: [0.0, 1.0, 0.0] },
        ]).unwrap();
        
    // create shadow map
    let shadow_color_texture = glium::texture::Texture2d::empty(&display, 1024, 1024).unwrap();
    let shadow_texture = glium::texture::DepthTexture2d::empty_with_format(&display,
        glium::texture::DepthFormat::I16, glium::texture::MipmapsOption::NoMipmap, 1024, 1024).unwrap();
    let mut shadow_buffer = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(&display,
        &shadow_color_texture, &shadow_texture).unwrap();
    let shadow_bias = [
        [0.5, 0.0, 0.0, 0.0],
        [0.0, 0.5, 0.0, 0.0],
        [0.0, 0.0, 0.5, 0.0],
        [0.5, 0.5, 0.5, 1.0f32]
        ];
        
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
        // backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockWise,
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
        let shadow_perspective = PerspMat3::new(1.0, std::f32::consts::PI / 2.0, 0.1, 10.0);
        let mut shadow_view: Iso3<f32> = nalgebra::one();
        shadow_view.look_at_z(&Pnt3::new(light_pos[0], -light_pos[1], light_pos[2]), &Pnt3::new(0.0, 0.0, 2.0), &Vec3::new(0.0, 0.0, -1.0));
        
        shadow_buffer.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);
        shadow_buffer.draw((&positions, &normals), &indices, &shadow_program,
            &uniform!{ perspective: shadow_perspective, view: shadow_view.to_homogeneous(), model: model },
            &params).unwrap();
        
        // start drawing on the frame
        let mut target = display.draw();
        target.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);

        // create the perspective matrix
        let (width, height) = target.get_dimensions();
        let perspective = PerspMat3::new(width as f32 / height as f32, std::f32::consts::PI / 3.0, 0.1, 10.0);
        
        // create view matrix
        let mut view: Iso3<f32> = nalgebra::one();
        view.look_at_z(&Pnt3::new(0.0, 0.0, 2.0), &Pnt3::new(0.0, 0.0, 3.0), &Vec3::new(0.0, 1.0, 0.0));

        // draw shape
        target.draw((&positions, &normals), &indices, &program,
            &uniform!{ perspective: perspective, view: view.to_homogeneous(), model: model, light_pos: light_pos,
                shadow_bias: shadow_bias, shadow_perspective: shadow_perspective, shadow_view: shadow_view.to_homogeneous(),
                shadow_map: &shadow_texture },
            &params).unwrap();
        target.draw(&shape, &glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip), &program,
            &uniform!{ perspective: perspective, view: view.to_homogeneous(), model: model, light_pos: light_pos,
                shadow_bias: shadow_bias, shadow_perspective: shadow_perspective, shadow_view: shadow_view.to_homogeneous(),
                shadow_map: &shadow_texture },
            &params).unwrap();
            
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
