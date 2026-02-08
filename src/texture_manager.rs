use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlImageElement, WebGlRenderingContext, WebGlTexture};

#[derive(Clone)]
pub struct SharedTexture {
    pub texture: Rc<WebGlTexture>,
    pub image: Rc<HtmlImageElement>,
}

pub struct TextureManager {
    cache: HashMap<String, SharedTexture>,
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get_texture(
        &mut self,
        context: &WebGlRenderingContext,
        src: &str,
    ) -> Result<SharedTexture, JsValue> {
        // 1. CHECK CACHE: If we already loaded this URL, return the saved one!
        if let Some(shared) = self.cache.get(src) {
            return Ok(shared.clone());
        }

        //if not
        let texture = context.create_texture().ok_or("failed to create texture")?;
        let texture_rc = Rc::new(texture); // Wrap in Shared Pointer

        // B. Bind & Set Blue Placeholder
        context.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&texture_rc));
        let blue_pixel: [u8; 4] = [0, 0, 255, 255];
        context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGlRenderingContext::TEXTURE_2D,
            0,
            WebGlRenderingContext::RGBA as i32,
            1,
            1,
            0,
            WebGlRenderingContext::RGBA,
            WebGlRenderingContext::UNSIGNED_BYTE,
            Some(&blue_pixel),
        )?;

        // C. Create Image Element
        let img = HtmlImageElement::new().unwrap();
        img.set_cross_origin(Some("anonymous"));
        let img_rc = Rc::new(img); // Wrap in Shared Pointer

        // D. Setup Async Loading (Closure)
        let texture_clone = texture_rc.clone();
        let img_clone = img_rc.clone();
        let context_clone = context.clone();

        let closure = Closure::wrap(Box::new(move || {
            context_clone.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&texture_clone));
            let _ = context_clone.tex_image_2d_with_u32_and_u32_and_image(
                WebGlRenderingContext::TEXTURE_2D,
                0,
                WebGlRenderingContext::RGBA as i32,
                WebGlRenderingContext::RGBA,
                WebGlRenderingContext::UNSIGNED_BYTE,
                &img_clone,
            );

            // Safe parameters for any size
            context_clone.tex_parameteri(
                WebGlRenderingContext::TEXTURE_2D,
                WebGlRenderingContext::TEXTURE_WRAP_S,
                WebGlRenderingContext::CLAMP_TO_EDGE as i32,
            );
            context_clone.tex_parameteri(
                WebGlRenderingContext::TEXTURE_2D,
                WebGlRenderingContext::TEXTURE_WRAP_T,
                WebGlRenderingContext::CLAMP_TO_EDGE as i32,
            );
            context_clone.tex_parameteri(
                WebGlRenderingContext::TEXTURE_2D,
                WebGlRenderingContext::TEXTURE_MIN_FILTER,
                WebGlRenderingContext::LINEAR as i32,
            );
            context_clone.tex_parameteri(
                WebGlRenderingContext::TEXTURE_2D,
                WebGlRenderingContext::TEXTURE_MAG_FILTER,
                WebGlRenderingContext::LINEAR as i32,
            );
        }) as Box<dyn FnMut()>);

        img_rc.set_onload(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
        img_rc.set_src(src);

        // 3. STORE IN CACHE
        let shared = SharedTexture {
            texture: texture_rc,
            image: img_rc,
        };

        self.cache.insert(src.to_string(), shared.clone());

        Ok(shared)
    }
}
