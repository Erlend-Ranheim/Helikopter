// Uncomment these following global attributes to silence most warnings of "low" interest:
/*
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]
*/
extern crate nalgebra_glm as glm;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::{mem, os::raw::c_void, ptr};
use std::ptr::null;
use glm::vec3;

mod mesh;
mod shader;
mod util;

mod scene_graph;
use scene_graph::SceneNode;

mod toolbox;

use glutin::event::{
    DeviceEvent,
    ElementState::{Pressed, Released},
    Event, KeyboardInput,
    VirtualKeyCode::{self, *},
    WindowEvent,
};
use glutin::event_loop::ControlFlow;
use crate::mesh::Helicopter;
use crate::toolbox::simple_heading_animation;

// initial window size
const INITIAL_SCREEN_W: u32 = 800;
const INITIAL_SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //

// Get the size of an arbitrary array of numbers measured in bytes
// Example usage:  byte_size_of_array(my_array)
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
// Example usage:  pointer_to_array(my_array)
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
// Example usage:  size_of::<u64>()
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T, represented as a relative pointer
// Example usage:  offset::<u64>(4)
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

unsafe fn draw_scene(node: &scene_graph::SceneNode,
                     view_projection_matrix: &glm::Mat4,
                     transformation_so_far: &glm::Mat4) {

    // Compute this node's local transformation
    let translate_to_ref = glm::translation(&node.reference_point);

    let rotation_x = glm::rotation(node.rotation.x, &glm::vec3(1.0, 0.0, 0.0));
    let rotation_y = glm::rotation(node.rotation.y, &glm::vec3(0.0, 1.0, 0.0));
    let rotation_z = glm::rotation(node.rotation.z, &glm::vec3(0.0, 0.0, 1.0));
    let rotation = rotation_z * rotation_y * rotation_x;

    let translate_from_ref = glm::translation(&-node.reference_point);

    let translate_pos = glm::translation(&node.position);

    let local_transform = translate_pos * translate_to_ref * rotation * translate_from_ref;

    let current_transform = transformation_so_far * local_transform;


    // Draw this node if it has geometry
    if node.index_count > 0 {
        //Calculates and passes the matrix to shader
        let mvp = view_projection_matrix * current_transform;
        gl::UniformMatrix4fv(0, 1, gl::FALSE, mvp.as_ptr());

        gl::UniformMatrix4fv(1, 1, gl::FALSE, current_transform.as_ptr());

        gl::BindVertexArray(node.vao_id);
        gl::DrawElements(
            gl::TRIANGLES,
            node.index_count,
            gl::UNSIGNED_INT,
            ptr::null(),
        );
    }

    // Recurse to children with the accumulated transformation
    for &child in &node.children {
        if !child.is_null() {
            draw_scene(&*child, view_projection_matrix, &current_transform);
        }
    }
}

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()

// == // Generate your VAO here
unsafe fn create_vao(
    vertices: &Vec<f32>,
    indices: &Vec<u32>,
    color: &Vec<f32>,
    normals: &Vec<f32>,
) -> u32 {
    // Implement me!

    // Also, feel free to delete comments :)

    // This should:
    // * Generate a VAO and bind it
    // VAO = Vertex Array Object
    // VAO stores the configuration for how vertex data should be interpreted and rendered.
    // This code creates a new VAO and makes it the active one.
    let mut vao = 0;
    gl::GenVertexArrays(1, &mut vao);
    gl::BindVertexArray(vao);

    // * Generate a VBO and bind it
    // VBO = Vertex Buffer Object
    let mut vbo_positions = 0;
    gl::GenBuffers(1, &mut vbo_positions);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo_positions);
    // * Fill it with data
    gl::BufferData(
        //function to fill the bound buffer with data
        gl::ARRAY_BUFFER,             //Targets the currently bound VBO
        byte_size_of_array(vertices), //Allocates size to memory
        pointer_to_array(vertices),   //pointer to array of vertices
        gl::STATIC_DRAW,              //tells GL that data wont change often
    );
    // * Configure a VAP for the data and enable it
    gl::VertexAttribPointer(
        0,
        3,
        gl::FLOAT,
        gl::FALSE,
        3 * size_of::<f32>(),
        offset::<f32>(0),
    );
    gl::EnableVertexAttribArray(0);


    // * Generate a VBO for colors
    let mut vbo_colors = 0;
    gl::GenBuffers(1, &mut vbo_colors);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo_colors);

    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(color),
        pointer_to_array(color),
        gl::STATIC_DRAW,
    );

    gl::VertexAttribPointer(
        1,
        4,
        gl::FLOAT,
        gl::FALSE,
        4 * size_of::<f32>(),
        offset::<f32>(0),
    );
    gl::EnableVertexAttribArray(1);
    // * Generate a IBO and bind it

    let mut ibo = 0;
    gl::GenBuffers(1, &mut ibo);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
    // * Fill it with data
    gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,
        byte_size_of_array(indices),
        pointer_to_array(indices),
        gl::STATIC_DRAW,
    );

    // * Generate a VBO for normals
    let mut vbo_normals = 0;
    gl::GenBuffers(1, &mut vbo_normals);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo_normals);

    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(normals),
        pointer_to_array(normals),
        gl::STATIC_DRAW,
    );

    gl::VertexAttribPointer(
        2,
        3,
        gl::FLOAT,
        gl::FALSE,
        3*size_of::<f32>(),
        offset::<f32>(0),
    );
    gl::EnableVertexAttribArray(2);


    // * Return the ID of the VAO

    vao
}

fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize::new(
            INITIAL_SCREEN_W,
            INITIAL_SCREEN_H,
        ));
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Set up shared tuple for tracking changes to the window size
    let arc_window_size = Arc::new(Mutex::new((INITIAL_SCREEN_W, INITIAL_SCREEN_H, false)));
    // Make a reference of this tuple to send to the render thread
    let window_size = Arc::clone(&arc_window_size);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers.
        // This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        let mut window_aspect_ratio = INITIAL_SCREEN_W as f32 / INITIAL_SCREEN_H as f32;

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!(
                "{}: {}",
                util::get_gl_string(gl::VENDOR),
                util::get_gl_string(gl::RENDERER)
            );
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!(
                "GLSL\t: {}",
                util::get_gl_string(gl::SHADING_LANGUAGE_VERSION)
            );
        }

        // == // Set up your VAO around here

        let vertices: Vec<f32> = vec![
            -0.8, -0.4, -0.7, -0.5, -1.0, -0.7, 0.0, -0.2, -0.7, 0.1, 0.1, 0.1, -0.4, -0.4, 0.1,
            0.2, -0.6, 0.1, -0.5, 0.4, 0.9, -0.25, -1.0, 0.9, -0.1, 0.5, 0.9,
        ];
        let indices: Vec<u32> = vec![6, 7, 8, 3, 4, 5, 0, 1, 2];

        let colors: Vec<f32> = vec![
            0.1, 1.0, 0.0, 0.3, 0.1, 1.0, 0.0, 0.3, 0.1, 1.0, 0.0, 0.3, 0.1, 0.2, 1.0, 0.3, 0.1,
            0.2, 1.0, 0.3, 0.1, 0.2, 1.0, 0.3, 1.0, 0.3, 0.2, 0.3, 1.0, 0.3, 0.2, 0.3, 1.0, 0.3,
            0.2, 0.3,
        ];

        //let my_vao = unsafe { create_vao(&vertices, &indices, &colors) };

        let terrain_mesh = mesh::Terrain::load(
            "C:/Users/ranhe/RustroverProjects/gloom-rs/resources/lunarsurface.obj",
        );

        let helicopter = Helicopter::load("C:/Users/ranhe/RustroverProjects/gloom-rs/resources/helicopter.obj");

        let body = helicopter.body;
        let door = helicopter.door;
        let main_rotor = helicopter.main_rotor;
        let tail_rotor = helicopter.tail_rotor;

        let body_vao = unsafe {
            create_vao(
                &body.vertices,
                &body.indices,
                &body.colors,
                &body.normals,
            )
        };

        let door_vao = unsafe {
            create_vao(
                &door.vertices,
                &door.indices,
                &door.colors,
                &door.normals,
            )
        };

        let main_rotor_vao = unsafe {
            create_vao(
                &main_rotor.vertices,
                &main_rotor.indices,
                &main_rotor.colors,
                &main_rotor.normals,
            )
        };

        let tail_rotor_vao = unsafe {
            create_vao(
                &tail_rotor.vertices,
                &tail_rotor.indices,
                &tail_rotor.colors,
                &tail_rotor.normals,
            )
        };

        let terrain_vao = unsafe {
            create_vao(
                &terrain_mesh.vertices,
                &terrain_mesh.indices,
                &terrain_mesh.colors,
                &terrain_mesh.normals
            )
        };

        // == // Set up your shaders here

        // Basic usage of shader helper:
        // The example code below creates a 'shader' object.
        // It which contains the field `.program_id` and the method `.activate()`.
        // The `.` in the path is relative to `Cargo.toml`.
        // This snippet is not enough to do the exercise, and will need to be modified (outside
        // of just using the correct path), but it only needs to be called once

        let simple_shader = unsafe {
            shader::ShaderBuilder::new()
                .attach_file("C:/Users/ranhe/RustroverProjects/gloom-rs/shaders/simple.frag")
                .attach_file("C:/Users/ranhe/RustroverProjects/gloom-rs/shaders/simple.vert") //./path/to/simple/shader.file
                .link()
        };
        unsafe {
            simple_shader.activate();
        }



        let mut camera_pos_axis = glm::vec3(0.0f32, 0.0f32, -10.0f32);
        let mut camera_pos_rotate = glm::vec2(0.0f32, 0.0f32);

        // The main rendering loop
        let first_frame_time = std::time::Instant::now();
        let mut previous_frame_time = first_frame_time;


        let mut helicopters: Vec<scene_graph::Node
        > = Vec::new();
        let mut main_rotors: Vec<scene_graph::Node
        > = Vec::new();
        let mut tail_rotors: Vec<scene_graph::Node
        > = Vec::new();


        // Creates one terrain node as root node
        let mut terrain_root_node = SceneNode::new();
        let mut terrain_node = SceneNode::from_vao(terrain_vao, terrain_mesh.index_count);
        terrain_root_node.add_child(&terrain_node);

        // Creates 5 helicopters, that all has the same terrain root node
        for i in 0..5{
            let mut helicopter_root_node = SceneNode::new();

            let mut body_node = SceneNode::from_vao(body_vao, body.index_count);
            let mut door_node = SceneNode::from_vao(door_vao, door.index_count);
            let mut main_rotor_node = SceneNode::from_vao(main_rotor_vao, main_rotor.index_count);
            let mut tail_rotor_node = SceneNode::from_vao(tail_rotor_vao, tail_rotor.index_count);

            tail_rotor_node.reference_point = vec3(0.35, 2.3, 10.4);
            main_rotor_node.reference_point = vec3(0.0, 0.0, 0.0);

            terrain_node.add_child(&helicopter_root_node);
            helicopter_root_node.add_child(&body_node);
            body_node.add_child(&door_node);
            body_node.add_child(&main_rotor_node);
            body_node.add_child(&tail_rotor_node);

            helicopters.push(helicopter_root_node);
            main_rotors.push(main_rotor_node);
            tail_rotors.push(tail_rotor_node);
        }




        loop {
            // Compute time passed since the previous frame and since the start of the program
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(previous_frame_time).as_secs_f32();
            previous_frame_time = now;

            // Handle resize events
            if let Ok(mut new_size) = window_size.lock() {
                if new_size.2 {
                    context.resize(glutin::dpi::PhysicalSize::new(new_size.0, new_size.1));
                    window_aspect_ratio = new_size.0 as f32 / new_size.1 as f32;
                    (*new_size).2 = false;
                    println!("Window was resized to {}x{}", new_size.0, new_size.1);
                    unsafe {
                        gl::Viewport(0, 0, new_size.0 as i32, new_size.1 as i32);
                    }
                }
            }

            let speed = 12.0;

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        // The `VirtualKeyCode` enum is defined here:
                        //    https://docs.rs/winit/0.25.0/winit/event/enum.VirtualKeyCode.html
                        VirtualKeyCode::D => {
                            camera_pos_axis.x += speed*delta_time;
                        }
                        VirtualKeyCode::A => {
                            camera_pos_axis.x -= speed*delta_time;
                        }
                        VirtualKeyCode::W => {
                            camera_pos_axis.y += speed*delta_time;
                        }
                        VirtualKeyCode::S => {
                            camera_pos_axis.y -= speed*delta_time;
                        }
                        VirtualKeyCode::LShift => {
                            camera_pos_axis.z += speed*delta_time;
                        }
                        VirtualKeyCode::Space => {
                            camera_pos_axis.z -= speed*delta_time;
                        }

                        VirtualKeyCode::Up => {
                            camera_pos_rotate.x += delta_time;
                        }
                        VirtualKeyCode::Down => {
                            camera_pos_rotate.x -= delta_time;
                        }
                        VirtualKeyCode::Left => {
                            camera_pos_rotate.y += delta_time;
                        }
                        VirtualKeyCode::Right => {
                            camera_pos_rotate.y -= delta_time;
                        }

                        // default handler:
                        _ => {}
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {
                // == // Optionally access the accumulated mouse movement between
                // == // frames here with `delta.0` and `delta.1`

                *delta = (0.0, 0.0); // reset when done
            }

            // == // Please compute camera transforms here (exercise 2 & 3)
            unsafe {
                // Clear the color and depth buffers
                gl::ClearColor(0.035, 0.046, 0.078, 1.0); // night sky
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                let oscillation = elapsed.sin();
                //affine transformations
                let rotate_x = glm::rotate_x(&glm::Mat4::identity(), camera_pos_rotate.x);
                let rotate_y = glm::rotate_y(&glm::Mat4::identity(), camera_pos_rotate.y);
                let rotate = rotate_x * rotate_y;
                let translate = glm::translation(&camera_pos_axis);

                let projection: glm::Mat4 = glm::perspective(window_aspect_ratio, 0.5, 1.0, 1000.0);

                let final_matrix = projection * rotate * translate;

                //Passing matrix to gpu
                gl::UniformMatrix4fv(0, 1, gl::FALSE, final_matrix.as_ptr());

                // == // Issue the necessary gl:: commands to draw your scene here
                /***
                gl::BindVertexArray(terrain_vao);
                gl::DrawElements(
                    gl::TRIANGLES,            // primitive type
                    terrain_mesh.index_count, // number of indices
                    gl::UNSIGNED_INT,         // index type
                    ptr::null(),              // offset
                );
                gl::BindVertexArray(body_vao);
                gl::DrawElements(
                    gl::TRIANGLES,            // primitive type
                    body.index_count, // number of indices
                    gl::UNSIGNED_INT,         // index type
                    ptr::null(),              // offset
                );
                gl::BindVertexArray(door_vao);
                gl::DrawElements(
                    gl::TRIANGLES,            // primitive type
                    door.index_count, // number of indices
                    gl::UNSIGNED_INT,         // index type
                    ptr::null(),              // offset
                );

                gl::BindVertexArray(main_rotor_vao);
                gl::DrawElements(
                    gl::TRIANGLES,            // primitive type
                    main_rotor.index_count, // number of indices
                    gl::UNSIGNED_INT,         // index type
                    ptr::null(),              // offset
                );

                gl::BindVertexArray(tail_rotor_vao);
                gl::DrawElements(
                    gl::TRIANGLES,            // primitive type
                    tail_rotor.index_count, // number of indices
                    gl::UNSIGNED_INT,         // index type
                    ptr::null(),              // offset
                );


**/
                for (i, heli) in helicopters.iter_mut().enumerate() {
                    let offsets = 1.2 * i as f32 ;
                    let heading = simple_heading_animation(elapsed+offsets);

                    heli.position.x = heading.x;
                    heli.position.z = heading.z;
                    heli.rotation.x = heading.roll;
                    heli.rotation.y = heading.yaw;
                    heli.rotation.z = heading.pitch;

                    let main_rotor_node = &mut *main_rotors[i];
                    main_rotor_node.rotation.y = elapsed * 10.0;

                    let tail_rotor_node = &mut *tail_rotors[i];
                    tail_rotor_node.rotation.x = elapsed * 20.0;


                }
                draw_scene(&terrain_root_node,&final_matrix, &glm::Mat4::identity());







            }


            // Display the new color buffer on the display
            context.swap_buffers().unwrap(); // we use "double buffering" to avoid artifacts
        }
    });

    // == //
    // == // From here on down there are only internals.
    // == //

    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events are initially handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(physical_size),
                ..
            } => {
                println!(
                    "New window size received: {}x{}",
                    physical_size.width, physical_size.height
                );
                if let Ok(mut new_size) = arc_window_size.lock() {
                    *new_size = (physical_size.width, physical_size.height, true);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: key_state,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        }
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle Escape and Q keys separately
                match keycode {
                    Escape => {
                        *control_flow = ControlFlow::Exit;
                    }
                    Q => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            _ => {}
        }
    });
}
