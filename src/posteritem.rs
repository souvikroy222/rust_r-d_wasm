use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlRenderingContext, WebGlTexture, HtmlImageElement, WebGlBuffer};

pub struct PosterItem {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub src: String,
    
    // Flags
    pub resize_contain: bool,
    pub is_selected: bool,

    // ANIMATION & SCROLL STATE 🎬
    pub anim_scale: f32,
    pub offset_x: f32,      // Horizontal Scroll (From RowList)
    pub offset_y: f32,      // NEW: Vertical Scroll (From ColumnList)
    
    // Optimization State
    prev_is_selected: bool, 
    prev_offset_x: f32,
    prev_offset_y: f32,     // NEW: Track changes

    // Assets
    pub texture: Option<Rc<WebGlTexture>>, 
    pub image_element: Option<Rc<HtmlImageElement>>,
    pub buffer: Option<WebGlBuffer>, 
}

impl PosterItem {
    pub fn new(x: f32, y: f32, w: f32, h: f32, src: &str, resize_contain: bool) -> Self {
        Self {
            x, y, w, h,
            src: src.to_string(),
            resize_contain,
            is_selected: false,
            anim_scale: 1.0,
            
            offset_x: 0.0,
            offset_y: 0.0, // Start at 0
            
            prev_offset_x: 0.0,
            prev_offset_y: 0.0,
            prev_is_selected: false,

            texture: None,
            image_element: None,
            buffer: None,
        }
    }

    // 1. Init Buffer (Standard)
    pub fn init_buffer(&mut self, context: &WebGlRenderingContext) -> Result<(), String> {
        let buffer = context.create_buffer().ok_or("Failed to create buffer")?;
        context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
        let vertices = self.create_rect();
        unsafe {
            let vert_array = js_sys::Float32Array::from(vertices.as_slice());
            context.buffer_data_with_array_buffer_view(WebGlRenderingContext::ARRAY_BUFFER, &vert_array, WebGlRenderingContext::STATIC_DRAW);
        }
        self.buffer = Some(buffer);
        Ok(())
    }

    // 2. Set Texture (Standard)
    pub fn set_texture(&mut self, texture: Rc<WebGlTexture>, image: Rc<HtmlImageElement>) {
        self.texture = Some(texture);
        self.image_element = Some(image);
    }

    // 3. UPDATE LOOP 🔄
    pub fn update(&mut self, context: &WebGlRenderingContext) {
        let mut needs_upload = false;

        // A. Resize Logic
        if self.resize_contain {
            if let Some(img) = &self.image_element {
                let img_w = img.natural_width() as f32;
                let img_h = img.natural_height() as f32;
                if img_w > 0.0 && img_h > 0.0 {
                    let ratio = img_h / img_w;
                    if (self.h - (self.w * ratio)).abs() > 0.01 {
                         self.h = self.w * ratio;
                         self.resize_contain = false;
                         needs_upload = true;
                    }
                }
            }
        }

        // B. SCROLL CHECK (X and Y) 📜
        if (self.offset_x - self.prev_offset_x).abs() > 0.1 {
            needs_upload = true;
            self.prev_offset_x = self.offset_x;
        }
        // NEW: Check Vertical Scroll
        if (self.offset_y - self.prev_offset_y).abs() > 0.1 {
            needs_upload = true;
            self.prev_offset_y = self.offset_y;
        }

        // C. ANIMATION LOGIC (LERP)
        let target_scale = if self.is_selected { 1.2 } else { 1.0 };
        let diff = target_scale - self.anim_scale;

        if diff.abs() > 0.001 {
            self.anim_scale += diff * 0.15;
            needs_upload = true; 
        } else {
            if self.anim_scale != target_scale {
                self.anim_scale = target_scale;
                needs_upload = true;
            }
        }

        // D. UPLOAD
        if needs_upload {
             if let Some(buffer) = &self.buffer {
                 context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(buffer));
                 let vertices = self.create_rect();
                 unsafe {
                     let vert_array = js_sys::Float32Array::from(vertices.as_slice());
                     context.buffer_data_with_array_buffer_view(WebGlRenderingContext::ARRAY_BUFFER, &vert_array, WebGlRenderingContext::DYNAMIC_DRAW);
                 }
             }
        }
    }

    // 4. Geometry Generator (Uses offset_x AND offset_y!)
    pub fn create_rect(&self) -> Vec<f32> {
        let scale = self.anim_scale;
        let center_x = self.x + (self.w / 2.0);
        let center_y = self.y + (self.h / 2.0);
        let new_w = self.w * scale;
        let new_h = self.h * scale;

        // APPLY SCROLL OFFSETS HERE! 
        let final_center_x = center_x + self.offset_x;
        let final_center_y = center_y + self.offset_y; // NEW!

        let x = final_center_x - (new_w / 2.0);
        let y = final_center_y - (new_h / 2.0);
        let x2 = x + new_w;
        let y2 = y + new_h;
        
        vec![
            x,  y,   0.0, 0.0,
            x,  y2,  0.0, 1.0,
            x2, y,   1.0, 0.0,
            x2, y,   1.0, 0.0,
            x,  y2,  0.0, 1.0,
            x2, y2,  1.0, 1.0,
        ]
    }

    // ... (rest of file: change_image, shaders - same as before) ...
    pub fn change_image(&mut self, new_src: &str) {
        self.src = new_src.to_string();
        self.resize_contain = true;
        self.texture = None;
        self.image_element = None;
    }
    pub fn get_vertex_shader() -> &'static str {
        r#"
            attribute vec2 position;
            attribute vec2 texCoord;
            uniform vec2 u_resolution;
            varying vec2 v_texCoord;
            void main() {
                vec2 zeroToOne = position / u_resolution;
                vec2 zeroToTwo = zeroToOne * 2.0;
                vec2 clipSpace = zeroToTwo - 1.0;
                gl_Position = vec4(clipSpace.x, clipSpace.y * -1.0, 0, 1);
                v_texCoord = texCoord;
            }
        "#
    }
    pub fn get_fragment_shader() -> &'static str {
        r#"
            precision mediump float;
            varying vec2 v_texCoord;
            uniform sampler2D u_texture;
            void main() {
                gl_FragColor = texture2D(u_texture, v_texCoord);
            }
        "#
    }
}