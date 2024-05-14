#[macro_use]
extern crate glium;
extern crate winit;
mod cloth;
use glium::{ Surface, VertexBuffer };
use std::fs;
use std::io::Read;

use cloth::Cloth;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

fn read_shader_src(path: &str) -> &'static str {
    let mut src = String::new();
    let mut file = fs::File::open(path).expect("Failed to open shader file");
    file.read_to_string(&mut src).expect("Failed to read shader file");

    Box::leak(src.into_boxed_str())
}

fn main() {
    // create event loop
    let event_loop = winit::event_loop::EventLoopBuilder
        ::new()
        .build()
        .expect("event loop building");
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder
        ::new()
        .with_title("Cloth Simulation")
        .with_inner_size(1280, 1000)
        .build(&event_loop);

    let n = 10;
    let m = 20;
    // create cloth
    let mut cloth = Cloth::new(n, m);

    // Adjust coordinates to center the grid
    let max_y = cloth.points
        .iter()
        .flat_map(|row| row.iter().map(|point| point.y))
        .fold(f32::MIN, f32::max);

    for row in &mut cloth.points {
        for point in row {
            point.x -= (m as f32) / 2.0;
            point.y -= max_y;
            point.y += 14.0; // extra offset
        }
    }

    let vertex_shader_src = read_shader_src("src/shaders/vertex.glsl");
    let fragment_shader_src = read_shader_src("src/shaders/fragment.glsl");

    let program = glium::Program
        ::from_source(&display, &vertex_shader_src, &fragment_shader_src, None)
        .unwrap();

    // render loop
    let _ = event_loop.run(move |event, window_target| {
        match event {
            winit::event::Event::WindowEvent { event, .. } =>
                match event {
                    winit::event::WindowEvent::CloseRequested => window_target.exit(),
                    winit::event::WindowEvent::MouseInput { .. } => {}
                    winit::event::WindowEvent::Resized(window_size) => {
                        display.resize(window_size.into());
                    }
                    winit::event::WindowEvent::RedrawRequested => {
                        let mut target = display.draw();
                        target.clear_color(1.0, 1.0, 1.0, 1.0);

                        let matrix = [
                            [0.07, 0.0, 0.0, 0.0],
                            [0.0, 0.07, 0.0, 0.0],
                            [0.0, 0.0, 0.07, 0.0],
                            [0.0, 0.0, 2.0, 1.0f32],
                        ];

                        let perspective = {
                            let (width, height) = target.get_dimensions();
                            let aspect_ratio = (height as f32) / (width as f32);

                            let fov: f32 = 3.141592 / 3.0;
                            let zfar = 1024.0;
                            let znear = 0.1;

                            let f = 1.0 / (fov / 2.0).tan();

                            [
                                [f * aspect_ratio, 0.0, 0.0, 0.0],
                                [0.0, f, 0.0, 0.0],
                                [0.0, 0.0, (zfar + znear) / (zfar - znear), 1.0],
                                [0.0, 0.0, -(2.0 * zfar * znear) / (zfar - znear), 0.0],
                            ]
                        };

                        let uniforms =
                            uniform! {
                            matrix: matrix,
                            perspective: perspective,
                        };

                        // convert to vertices and indices
                        let vertices: Vec<Vertex> = cloth.points
                            .iter()
                            .flat_map(|row| {
                                row.iter().map(|point| Vertex { position: [point.x, point.y] })
                            })
                            .collect();

                        let indices: Vec<u16> = cloth.springs
                            .iter()
                            .flat_map(|spring| {
                                vec![
                                    (spring.p1.0 as u16) * (m as u16) + (spring.p1.1 as u16),
                                    (spring.p2.0 as u16) * (m as u16) + (spring.p2.1 as u16)
                                ]
                            })
                            .collect();

                        // create vertex and index buffer
                        let vertex_buffer = VertexBuffer::new(&display, &vertices).unwrap();
                        let index_buffer = glium::IndexBuffer
                            ::new(&display, glium::index::PrimitiveType::LinesList, &indices)
                            .unwrap();

                        // update simulation
                        cloth.simulate(0.01);

                        target
                            .draw(
                                &vertex_buffer,
                                &index_buffer,
                                &program,
                                &uniforms,
                                &Default::default()
                            )
                            .unwrap();
                        target.finish().unwrap();
                    }
                    _ => (),
                }
            winit::event::Event::AboutToWait => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}
