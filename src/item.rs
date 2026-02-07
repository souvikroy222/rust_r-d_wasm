use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlRenderingContext, WebGlTexture, HtmlImageElement};

pub struct Item {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub src: String,
    pub resize_contain: bool,
    
    // NEW: The Item holds its own assets!
    // They are "Option" because they start as None.
    pub texture: Option<WebGlTexture>, 
    pub image_element: Option<HtmlImageElement>,
}

impl Item {
    // 1. Constructor (Optional, but makes creating items cleaner)
    pub fn new(x: f32, y: f32, w: f32, h: f32, src: &str, resize_contain: bool) -> Self {
        Self {
            x, y, w, h,
            src: src.to_string(),
            resize_contain,
            texture: None,
            image_element: None,
        }
    }

    // 2. The Logic Loop (Call this every frame!)
    pub fn update(&mut self) {
        // If we need to resize AND we have an image loaded...
        if self.resize_contain {
            if let Some(img) = &self.image_element {
                let img_w = img.natural_width() as f32;
                let img_h = img.natural_height() as f32;

                // Check if image is ready (size > 0)
                if img_w > 0.0 && img_h > 0.0 {
                    let ratio = img_h / img_w;
                    self.h = self.w * ratio;
                    
                    // Turn off flag so we don't recalculate forever
                    self.resize_contain = false; 
                    
                    // (Optional) Debug log
                    // web_sys::console::log_1(&"Resized!".into()); 
                }
            }
        }
    }

    // 3. Texture Loader (Now stores results inside the struct)
    pub fn load_texture(&mut self, context: &WebGlRenderingContext) -> Result<(), JsValue> {
        let texture = context.create_texture().ok_or("failed to create texture")?;
        context.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&texture));

        // Blue Placeholder
        let blue_pixel: [u8; 4] = [0, 0, 255, 255]; 
        context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGlRenderingContext::TEXTURE_2D, 0, WebGlRenderingContext::RGBA as i32, 
            1, 1, 0, WebGlRenderingContext::RGBA, WebGlRenderingContext::UNSIGNED_BYTE, Some(&blue_pixel),
        )?;

        let img = HtmlImageElement::new().unwrap();
        img.set_cross_origin(Some("anonymous"));

        let texture_clone = texture.clone();
        let context_clone = context.clone();
        let img_clone = img.clone();

        let closure = Closure::wrap(Box::new(move || {
            context_clone.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&texture_clone));
            let _ = context_clone.tex_image_2d_with_u32_and_u32_and_image(
                WebGlRenderingContext::TEXTURE_2D, 0, WebGlRenderingContext::RGBA as i32,
                WebGlRenderingContext::RGBA, WebGlRenderingContext::UNSIGNED_BYTE, &img_clone,
            );
            context_clone.tex_parameteri(WebGlRenderingContext::TEXTURE_2D, WebGlRenderingContext::TEXTURE_WRAP_S, WebGlRenderingContext::CLAMP_TO_EDGE as i32);
            context_clone.tex_parameteri(WebGlRenderingContext::TEXTURE_2D, WebGlRenderingContext::TEXTURE_WRAP_T, WebGlRenderingContext::CLAMP_TO_EDGE as i32);
            context_clone.tex_parameteri(WebGlRenderingContext::TEXTURE_2D, WebGlRenderingContext::TEXTURE_MIN_FILTER, WebGlRenderingContext::LINEAR as i32);
            context_clone.tex_parameteri(WebGlRenderingContext::TEXTURE_2D, WebGlRenderingContext::TEXTURE_MAG_FILTER, WebGlRenderingContext::LINEAR as i32);
        }) as Box<dyn FnMut()>);

        img.set_onload(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
        img.set_src(&self.src);

        // STORE THEM INSIDE THE STRUCT!
        self.texture = Some(texture);
        self.image_element = Some(img);

        Ok(())
    }

    // 4. Geometry (Same as before)
    pub fn create_rect(&self) -> Vec<f32> {
        let x2 = self.x + self.w;
        let y2 = self.y + self.h;
        
        vec![
            self.x, self.y,  0.0, 0.0,
            self.x, y2,      0.0, 1.0,
            x2,     self.y,  1.0, 0.0,
            x2,     self.y,  1.0, 0.0,
            self.x, y2,      0.0, 1.0,
            x2,     y2,      1.0, 1.0,
        ]
    }

    // 5. Shaders (Same as before)
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