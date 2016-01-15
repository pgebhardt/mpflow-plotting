extern crate glium;
extern crate nalgebra;

use nalgebra::{Vec3, Norm};

// vertex format
#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    value: f32,
}
implement_vertex!(Vertex, position, normal, value);

#[derive(Copy, Clone)]
pub struct PortVertex {
    position: [f32; 3],
    color: [f32; 3],
}
implement_vertex!(PortVertex, position, color);

pub fn generate_mesh<F: glium::backend::Facade>(facade: &F,
    nodes: &Vec<Vec<f32>>, elements: &Vec<Vec<i32>>, reconstruction: &Vec<f32>, face_up: bool)
    -> Result<glium::VertexBuffer<Vertex>, glium::vertex::BufferCreationError> {

    // create interpolated z values
    let z_values = calculate_z_values(nodes, elements, reconstruction);

    // calculate radius of mesh to scale each vertex to unit size
    let radius = nodes.iter().fold(0.0f32, |acc, item| (item[0] * item[0] + item[1] * item[1]).sqrt().max(acc));

    // create vertex array
    let vertex_data: Vec<_> = elements.iter().zip(reconstruction).flat_map(|(indices, &value)| {
        // extract coordinates of triangle
        let triangle: Vec<_> = indices.iter().map(|&index| {
            Vec3::new(nodes[index as usize][0] / radius, nodes[index as usize][1] / radius, z_values[index as usize])
        }).collect();

        // calculate normal of the triangle
        let normal = calculate_normal(&triangle[0], &triangle[1], &triangle[2]).normalize();

        // set vertices
        if normal.z <= 0.0 && face_up || normal.z > 0.0 && !face_up {
            triangle.iter().map(|&node|
                Vertex {
                    position: *node.as_ref(),
                    normal: *normal.as_ref(),
                    value: value,
                }
            ).collect::<Vec<_>>()
        }
        else {
            triangle.iter().rev().map(|&node|
                Vertex {
                    position: *node.as_ref(),
                    normal: *(-normal).as_ref(),
                    value: value,
                }
            ).collect::<Vec<_>>()
        }
    }).collect();

    // load data to gpu
    glium::vertex::VertexBuffer::new(facade, &vertex_data)
}

pub fn generate_ports<F: glium::backend::Facade>(facade: &F,
    nodes: &Vec<Vec<f32>>, edges: &Vec<Vec<i32>>, ports: &Vec<Vec<i32>>)
    -> Result<glium::VertexBuffer<PortVertex>, glium::vertex::BufferCreationError> {
    // calculate radius of mesh to scale each vertex to unit size
    let radius = nodes.iter().fold(0.0f32, |acc, item| (item[0] * item[0] + item[1] * item[1]).sqrt().max(acc));

    // create vertex array
    let vertex_data: Vec<_> = ports.iter().enumerate().flat_map(|(i, e)| {
        let mut port_edges = Vec::new();
        for &edge in e.iter() {
            // don't use invalid node indices
            if edge != -1 {
                // create one vertex for each node of the port edges
                // the first port is colored red
                for &index in edges[edge as usize].iter() {
                    let index = index as usize;

                    port_edges.push(PortVertex {
                        position: *(Vec3::new(nodes[index][0], nodes[index][1], 0.0) * 1.005 / radius).as_ref(),
                        color: [if i == 0 { 1.0 } else { 0.0 }, 0.0, 0.0],
                    });
                }
            }
        }

        port_edges
    }).collect();

    // load data to gpu
    glium::vertex::VertexBuffer::new(facade, &vertex_data)
}

fn calculate_z_values(nodes: &Vec<Vec<f32>>, elements: &Vec<Vec<i32>>, values: &Vec<f32>) -> Vec<f32> {
    // interpolate z values of nodes by using the area and value of each element
    // and norm it to the area of all elements the node is part of
    let element_area: Vec<_> = elements.iter().map(|e| {
        ((nodes[e[1] as usize][0] - nodes[e[0] as usize][0]) *
        (nodes[e[2] as usize][1] - nodes[e[0] as usize][1]) -
        (nodes[e[2] as usize][0] - nodes[e[0] as usize][0]) *
        (nodes[e[1] as usize][1] - nodes[e[0] as usize][1])).abs() * 0.5
    }).collect();

    let mut node_area = vec![0.0f32; nodes.len()];
    for (i, elem) in elements.iter().enumerate() {
        for n in elem.iter() {
            node_area[*n as usize] += element_area[i];
        }
    }

    let mut z_values = vec![0.0f32; nodes.len()];
    for (i, elem) in elements.iter().enumerate() {
        for n in elem.iter() {
            z_values[*n as usize] +=
                values[i] * element_area[i] / node_area[*n as usize];
        }
    }

    z_values
}

fn calculate_normal(p1: &Vec3<f32>, p2: &Vec3<f32>, p3: &Vec3<f32>) -> Vec3<f32> {
    let u = *p2 - *p1;
    let v = *p3 - *p1;

    Vec3::new(
        u.y * v.z - u.z * v.y,
        u.z * v.x - u.x * v.z,
        u.x * v.y - u.y * v.x,
    )
}
