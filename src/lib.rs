// src/lib.rs
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlRenderingContext, HtmlCanvasElement};

mod posteritem;
mod rowlist;
mod columnlist;
mod texture_manager;

use crate::posteritem::PosterItem;
use crate::columnlist::ColumnList;
use crate::texture_manager::TextureManager;


macro_rules! log {
    ($($t:tt)*) => (web_sys::console::log_1(&format!($($t)*).into()))
}
// 1. Define the Engine Struct (Exported to JS)
#[wasm_bindgen]
pub struct GameEngine {
    context: WebGlRenderingContext,
    root_list: ColumnList,
    texture_manager: TextureManager,
}

#[wasm_bindgen]
impl GameEngine {
    // 2. The Constructor (Called from JS: "new GameEngine()")
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<GameEngine, JsValue> {
        // A. Setup WebGL
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let canvas = document.get_element_by_id(canvas_id).expect("no canvas");
        let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>()?;
        
        let context = canvas
            .get_context("webgl")?
            .unwrap()
            .dyn_into::<WebGlRenderingContext>()?;

        // B. Setup Shaders (Helper functions are at the bottom)
        let vert_shader = compile_shader(&context, WebGlRenderingContext::VERTEX_SHADER, PosterItem::get_vertex_shader())?;
        let frag_shader = compile_shader(&context, WebGlRenderingContext::FRAGMENT_SHADER, PosterItem::get_fragment_shader())?;
        let program = link_program(&context, &vert_shader, &frag_shader)?;
        context.use_program(Some(&program));

        // C. Configure Global GL State
        let res_loc = context.get_uniform_location(&program, "u_resolution").expect("u_resolution missing");
        context.uniform2f(Some(&res_loc), canvas.width() as f32, canvas.height() as f32);
        
        context.enable(WebGlRenderingContext::BLEND);
        context.blend_func(WebGlRenderingContext::SRC_ALPHA, WebGlRenderingContext::ONE_MINUS_SRC_ALPHA);

        // Enable Attributes
        let stride = 4 * 4; 
        let pos_loc = context.get_attrib_location(&program, "position");
        context.enable_vertex_attrib_array(pos_loc as u32);
        context.vertex_attrib_pointer_with_i32(pos_loc as u32, 2, WebGlRenderingContext::FLOAT, false, stride, 0);

        let tex_loc = context.get_attrib_location(&program, "texCoord");
        context.enable_vertex_attrib_array(tex_loc as u32);
        context.vertex_attrib_pointer_with_i32(tex_loc as u32, 2, WebGlRenderingContext::FLOAT, false, stride, 8);

        // D. Create The Game State
        let mut texture_manager = TextureManager::new();
        let mut root_list = ColumnList::new();
        
        // Load Assets
        root_list.load_assets(&context, &mut texture_manager)?;

        // Return the Struct to JS
        Ok(GameEngine {
            context,
            root_list,
            texture_manager,
        })
    }

    // 3. The Bridge: Input (Called from JS) 🌉
    pub fn send_key(&mut self, key_code: u32) {
        self.root_list.handle_input(key_code);
    }

    // 4. The Loop: Render (Called from JS requestAnimationFrame) 🔄
    pub fn render(&mut self) {
        // Clear
        self.context.clear_color(0.1, 0.1, 0.1, 1.0);
        self.context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        // Update
        self.root_list.update(&self.context);

        // Draw
        self.root_list.draw(&self.context);
    }
}

// ... (Keep helper functions compile_shader and link_program exactly as they were) ...
fn compile_shader(context: &WebGlRenderingContext, shader_type: u32, source: &str) -> Result<web_sys::WebGlShader, String> {
    let shader = context.create_shader(shader_type).ok_or("Unable to create shader object")?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);
    if context.get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS).as_bool().unwrap_or(false) { Ok(shader) } else { Err(context.get_shader_info_log(&shader).unwrap_or_else(|| "Unknown error".into())) }
}

fn link_program(context: &WebGlRenderingContext, vert: &web_sys::WebGlShader, frag: &web_sys::WebGlShader) -> Result<web_sys::WebGlProgram, String> {
    let program = context.create_program().ok_or("Unable to create shader program")?;
    context.attach_shader(&program, vert);
    context.attach_shader(&program, frag);
    context.link_program(&program);
    if context.get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS).as_bool().unwrap_or(false) { Ok(program) } else { Err(context.get_program_info_log(&program).unwrap_or_else(|| "Unknown error".into())) }
}