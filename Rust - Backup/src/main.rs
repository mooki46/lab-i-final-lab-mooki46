#[macro_use]
extern crate glium;
extern crate winit;
use std::env;

mod cloth;

use glium::{ Surface, VertexBuffer };
use std::{ fs, time::Instant };
use std::io::Read;
use cloth::Cloth;

extern crate num_cpus;
use once_cell::sync::Lazy;

pub static CORE_COUNT: Lazy<usize> = Lazy::new(|| num_cpus::get_physical());

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
    env::set_var("RUST_BACKTRACE", "1");
    println!("Core Count: {}", *CORE_COUNT);

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

    let n = 30;
    let m = 30;

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
            point.y += 30.0; // extra offset
        }
    }

    let vertex_shader_src = read_shader_src("src/shaders/vertex.glsl");
    let fragment_shader_src = read_shader_src("src/shaders/fragment.glsl");

    let program = glium::Program
        ::from_source(&display, &vertex_shader_src, &fragment_shader_src, None)
        .unwrap();

    let mut mouse_pos = (0.0, 0.0);
    let mut closest_point = None;
    let mut window_size = (0, 0);
    let mut matrix = [[0.0f32; 4]; 4];
    let mut aspect_ratio: f32 = 0.0;

    let mut fps_values = Vec::new();
    let mut simulation_times = Vec::new();
    let mut frame_draw_times = Vec::new();
    let mut last_frame_time = Instant::now();

    let mut affected_point: Option<(usize, usize)> = None;

    // render loop
    let _ = event_loop.run(move |event, window_target| {
        match event {
            winit::event::Event::WindowEvent { event, .. } =>
                match event {
                    winit::event::WindowEvent::CloseRequested => {
                        // calculate and print average fps
                        if !fps_values.is_empty() {
                            fps_values.remove(0); // remove the first frame
                            let total_frames = fps_values.len();
                            let avg_fps = fps_values.iter().sum::<f32>() / (total_frames as f32);
                            println!("Average FPS: {}", avg_fps);
                        }

                        if !simulation_times.is_empty() {
                            let total_sims = simulation_times.len();
                            let avg_time: f32 =
                                simulation_times.iter().sum::<f32>() / (total_sims as f32);
                            println!("Average Simulation Time: {} ms", avg_time);
                        }

                        if !frame_draw_times.is_empty() {
                            let total_draws = frame_draw_times.len();
                            let avg_time: f32 =
                                frame_draw_times.iter().sum::<f32>() / (total_draws as f32);
                            println!("Average Draw Time: {} ms", avg_time);
                        }

                        window_target.exit()
                    }
                    winit::event::WindowEvent::Resized(new_size) => {
                        window_size = new_size.into();
                        display.resize(window_size);
                    }
                    winit::event::WindowEvent::CursorMoved { position, .. } => {
                        mouse_pos = position.into();
                    }
                    winit::event::WindowEvent::MouseInput { state, button, .. } => {
                        if button == winit::event::MouseButton::Left {
                            if state == winit::event::ElementState::Pressed {
                                // Iterate over cloth points to find the closest one to the mouse
                                println!("Mouse Positions ({}, {})", mouse_pos.0, mouse_pos.1);

                                let mouse_x = (mouse_pos.0 / (window_size.0 as f32)) * 2.0 - 1.0;
                                let mouse_y = -2.0 * (mouse_pos.1 / (window_size.1 as f32) - 0.5);

                                println!("Normalized Mouse Positions ({}, {})", mouse_x, mouse_y);

                                println!("Aspect Ratio: {}", aspect_ratio);

                                let closest = cloth.points
                                    .iter()
                                    .enumerate()
                                    .flat_map(|(i, row)| {
                                        row.iter()
                                            .enumerate()
                                            .map(move |(j, _)| (i, j))
                                    })
                                    .min_by(|&(i1, j1), &(i2, j2)| {
                                        let point1 = &cloth.points[i1][j1];
                                        let t_point_x1 = point1.x * 0.03 * aspect_ratio;
                                        let t_point_y1 = point1.y * 0.03;
                                        let dx1 = t_point_x1 - mouse_x;
                                        let dy1 = t_point_y1 - mouse_y;
                                        let distance1 = dx1 * dx1 + dy1 * dy1;

                                        let point2 = &cloth.points[i2][j2];
                                        let t_point_x2 = point2.x * 0.03 * aspect_ratio;
                                        let t_point_y2 = point2.y * 0.03;
                                        let dx2 = t_point_x2 - mouse_x;
                                        let dy2 = t_point_y2 - mouse_y;
                                        let distance2 = dx2 * dx2 + dy2 * dy2;

                                        distance1.partial_cmp(&distance2).unwrap()
                                    });

                                println!("Closest: {:?}", closest);

                                if let Some((i, j)) = closest {
                                    cloth.points[i][j].ext_m += 10.0;
                                    closest_point = Some((i, j));
                                }
                            } else if state == winit::event::ElementState::Released {
                                if let Some((i, j)) = closest_point {
                                    cloth.points[i][j].ext_m = 0.0;
                                }
                                closest_point = None;
                            }
                        }
                    }
                    winit::event::WindowEvent::KeyboardInput { event, .. } => {
                        if
                            let winit::event::KeyEvent {
                                state,
                                logical_key: winit::keyboard::Key::Character(c),
                                ..
                            } = event
                        {
                            if
                                c.to_lowercase() == "g" &&
                                state == winit::event::ElementState::Pressed
                            {
                                cloth.g_on = !cloth.g_on;
                            }

                            if c.to_lowercase() == "f" {
                                if
                                    state == winit::event::ElementState::Pressed &&
                                    affected_point.is_none()
                                {
                                    let i = rand::random::<usize>() % cloth.points.len();
                                    let j = rand::random::<usize>() % cloth.points[0].len();

                                    cloth.points[i][j].ext_m += 10.0;
                                    affected_point = Some((i, j));
                                } else if state == winit::event::ElementState::Released {
                                    if let Some((i, j)) = affected_point {
                                        cloth.points[i][j].ext_m = 0.0;
                                    }
                                    affected_point = None;
                                }
                            }
                        }
                    }
                    winit::event::WindowEvent::RedrawRequested => {
                        let frame_start_time = Instant::now();
                        let frame_time = frame_start_time.duration_since(last_frame_time);
                        let fps = 1.0 / frame_time.as_secs_f32();
                        fps_values.push(fps);

                        let mut target = display.draw();
                        target.clear_color(1.0, 1.0, 1.0, 1.0);

                        let width = window_size.0;
                        let height = window_size.1;

                        aspect_ratio = (height as f32) / (width as f32);

                        matrix = [
                            [0.03 * aspect_ratio, 0.0, 0.0, 0.0],
                            [0.0, 0.03, 0.0, 0.0],
                            [0.0, 0.0, 1.0, 0.0],
                            [0.0, 0.0, 0.0, 1.0f32],
                        ];

                        let uniforms =
                            uniform! {
                            matrix: matrix,
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
                        for _ in 0..10 {
                            let sim_start = Instant::now();
                            // cloth.simulate_multithreaded(0.01);
                            cloth.simulate(0.01);
                            let sim_end = Instant::now();
                            let sim_time = sim_end.duration_since(sim_start).as_micros();
                            simulation_times.push((sim_time as f32) / 1000.0); // convert to millis
                        }

                        let draw_start = Instant::now();
                        target
                            .draw(
                                &vertex_buffer,
                                &index_buffer,
                                &program,
                                &uniforms,
                                &Default::default()
                            )
                            .unwrap();
                        let draw_end = Instant::now();
                        let draw_time = draw_end.duration_since(draw_start).as_micros();
                        frame_draw_times.push((draw_time as f32) / 1000.0); // convert to millis

                        target.finish().unwrap();

                        last_frame_time = frame_start_time;
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
