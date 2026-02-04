use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGlShader;
use web_sys::{WebGlRenderingContext, WebGlProgram};

// Basic State to track animation
struct AppState {
    current_x: f32, // Where the box is drawing now
    target_x: f32,  // Where the box wants to go
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    // 1. Setup Canvas & Context (WebGL 1.0)
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("tv-canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>()?;

    // --- FIX FOR BLURRY EDGES START ---
    // Get the ratio between physical pixels and CSS pixels (usually 1.0, 1.5, or 2.0 on TVs)
    let dpr = window.device_pixel_ratio(); 
    
    // Get the CSS size (how big the element is on screen)
    let css_width = canvas.client_width() as f64;
    let css_height = canvas.client_height() as f64;

    // Set the internal buffer size to match physical pixels
    let physical_width = (css_width * dpr) as u32;
    let physical_height = (css_height * dpr) as u32;

    canvas.set_width(physical_width);
    canvas.set_height(physical_height);
    // --- FIX END ---
    
    // Explicitly ask for webgl1 for old Chromium compatibility
    let gl = canvas.get_context("webgl")?.unwrap().dyn_into::<WebGlRenderingContext>()?;

    // 2. Compile Shaders
    let vert_code = r#"
        attribute vec2 position;
        uniform float u_offset_x;
        void main() {
            // Simple 2D translation. 
            // In clip space, screen is -1.0 to 1.0
            gl_Position = vec4(position.x + u_offset_x, position.y, 0.0, 1.0);
        }
    "#;
    let frag_code = "void main() { gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0); }"; // Red color

    let program = link_program(&gl, vert_code, frag_code)?;
    gl.use_program(Some(&program));

    // 3. Define Geometry (A simple square, 2 triangles)
    // Coords: -0.2 to 0.2 (Size relative to screen)
    let vertices: [f32; 12] = [
        -0.2, -0.2,   0.2, -0.2,   -0.2,  0.2, 
        -0.2,  0.2,   0.2, -0.2,    0.2,  0.2,
    ];

    let buffer = gl.create_buffer().ok_or("failed to create buffer")?;
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
    
    // "view" into the WASM memory buffer to pass to JS
    // Note: Creating a Float32Array view is cheap (no copy)
    unsafe {
        let vert_array = js_sys::Float32Array::view(&vertices);
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }

    // Link "position" attribute
    let position_attrib = gl.get_attrib_location(&program, "position");
    gl.vertex_attrib_pointer_with_i32(position_attrib as u32, 2, WebGlRenderingContext::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(position_attrib as u32);

    // Get Uniform Location
    let u_offset_loc = gl.get_uniform_location(&program, "u_offset_x").expect("u_offset_x not found");

    // 4. State Management
    let state = Rc::new(RefCell::new(AppState {
        current_x: 0.0,
        target_x: 0.0,
    }));

    // 5. Input Handler
    let state_input = state.clone();
    let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        let mut s = state_input.borrow_mut();
        // Move by 0.5 units in Clip Space (-1 to 1)
        match event.key_code() {
            39 => s.target_x += 0.5, // Right
            37 => s.target_x -= 0.5, // Left
            _ => {}
        }
    }) as Box<dyn FnMut(_)>);
    window.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
    closure.forget();

    // 6. Render Loop (The Heart of Performance)
    // We use a recursive requestAnimationFrame loop
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    let state_render = state.clone();
    let gl_render = gl.clone();
    
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let mut s = state_render.borrow_mut();

        // LERP: Smooth animation logic
        // Move 10% of the distance per frame. 
        // This creates a nice "slide" effect that slows down as it arrives.
        let diff = s.target_x - s.current_x;
        
        // Only draw if we are moving (Energy Efficiency)
        if diff.abs() > 0.001 {
            s.current_x += diff * 0.1;

            gl_render.clear_color(0.0, 0.0, 0.0, 1.0); // Black background
            gl_render.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

            gl_render.uniform1f(Some(&u_offset_loc), s.current_x);
            
            // Draw 6 vertices (2 triangles)
            gl_render.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 6);
        }

        // Request next frame
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn link_program(
    gl: &WebGlRenderingContext,
    vert_source: &str,
    frag_source: &str,
) -> Result<WebGlProgram, String> {
    let program = gl.create_program().ok_or("Unable to create shader object")?;
    let vert_shader = compile_shader(&gl, WebGlRenderingContext::VERTEX_SHADER, vert_source)?;
    let frag_shader = compile_shader(&gl, WebGlRenderingContext::FRAGMENT_SHADER, frag_source)?;

    gl.attach_shader(&program, &vert_shader);
    gl.attach_shader(&program, &frag_shader);
    gl.link_program(&program);

    if gl.get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS).as_bool().unwrap_or(false) {
        Ok(program)
    } else {
        Err(gl.get_program_info_log(&program).unwrap_or_else(|| "Unknown link error".into()))
    }
}

fn compile_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl.create_shader(shader_type).ok_or("Unable to create shader object")?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl.get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS).as_bool().unwrap_or(false) {
        Ok(shader)
    } else {
        Err(gl.get_shader_info_log(&shader).unwrap_or_else(|| "Unknown shader compile error".into()))
    }
}