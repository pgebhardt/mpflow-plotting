#[macro_use]
extern crate glium;

fn main() {
    // use glium display builder trait to create a window with openGL context
    // use Surface trait to get standard frame manipulation functionality
    use glium::{DisplayBuild, Surface};
    
    // create window
    let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();
    
    // start event loop, exit loop when window closes
    loop {
        // start drawing on the frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
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
