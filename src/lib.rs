use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGlRenderingContext};

mod item;
use crate::item::Item;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // 1. SETUP
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let canvas = document.get_element_by_id("my-canvas").expect("no canvas");
    let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>()?;
    let context = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()?;

    // 2. SHADERS
    let vert_shader = compile_shader(
        &context,
        WebGlRenderingContext::VERTEX_SHADER,
        Item::get_vertex_shader(),
    )?;
    let frag_shader = compile_shader(
        &context,
        WebGlRenderingContext::FRAGMENT_SHADER,
        Item::get_fragment_shader(),
    )?;
    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));

    // 3. SCREEN SIZE
    let res_loc = context
        .get_uniform_location(&program, "u_resolution")
        .expect("u_resolution missing");
    context.uniform2f(
        Some(&res_loc),
        canvas.width() as f32,
        canvas.height() as f32,
    );

    let mut item1 = Item::new(
        50.0,
        50.0,
        200.0,
        200.0,
        "https://media-cache.cinematerial.com/p/500x/uq34tcxi/swades-indian-movie-poster.jpg",
        false,
    );

    // This one will auto-resize! Height is 0.0 initially.
    let mut item2 = Item::new(
        260.0,
        50.0,
        300.0,
        100.0,
        "https://m.media-amazon.com/images/M/MV5BNGI0MDI4NjEtOWU3ZS00ODQyLWFhYTgtNGYxM2ZkM2Q2YjE3XkEyXkFqcGc@._V1_.jpg",
        true,
    );

    //load their textures immediately
    item1.load_texture(&context)?;
    item2.load_texture(&context)?;

    // Store in RefCell
    let items = Rc::new(RefCell::new(vec![item1, item2]));

    // Enable Attributes (Do this once)
    let buffer = context.create_buffer().ok_or("Failed buffer")?;
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));

    let stride = 4 * 4;
    let pos_loc = context.get_attrib_location(&program, "position");
    context.enable_vertex_attrib_array(pos_loc as u32);
    context.vertex_attrib_pointer_with_i32(
        pos_loc as u32,
        2,
        WebGlRenderingContext::FLOAT,
        false,
        stride,
        0,
    );

    let tex_loc = context.get_attrib_location(&program, "texCoord");
    context.enable_vertex_attrib_array(tex_loc as u32);
    context.vertex_attrib_pointer_with_i32(
        tex_loc as u32,
        2,
        WebGlRenderingContext::FLOAT,
        false,
        stride,
        8,
    );

    // --- 6. THE GAME LOOP 🎮 ---
    // We wrap everything in a Closure that calls itself
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let context = Rc::new(context); // Share context with loop

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // --- DRAW FRAME ---
        context.clear_color(0.1, 0.1, 0.1, 1.0);
        context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        let mut items_borrow = items.borrow_mut();

        for item in items_borrow.iter_mut() {
            //the item checks if it needs to resize itself
            item.update();

            //draw logic
            if let Some(texture) = &item.texture {
                // Update Buffer with shape
                let vertices = item.create_rect();
                unsafe {
                    let vert_view = js_sys::Float32Array::view(&vertices);
                    context.buffer_data_with_array_buffer_view(
                        WebGlRenderingContext::ARRAY_BUFFER,
                        &vert_view,
                        WebGlRenderingContext::STATIC_DRAW, // Ideally DYNAMIC_DRAW
                    );
                }
                // Bind Texture and Draw
            context.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(texture));
            context.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 6);
            }

            
        }

        // --- REQUEST NEXT FRAME ---
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

// Helper for the loop
fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .expect("no global `window` exists")
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

// ... Keep compile_shader and link_program below ...
fn compile_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<web_sys::WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

fn link_program(
    context: &WebGlRenderingContext,
    vert_shader: &web_sys::WebGlShader,
    frag_shader: &web_sys::WebGlShader,
) -> Result<web_sys::WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader program"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
