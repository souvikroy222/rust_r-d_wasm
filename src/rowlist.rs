use crate::posteritem::PosterItem;
use crate::texture_manager::TextureManager;
use wasm_bindgen::JsValue;
use web_sys::WebGlRenderingContext;

pub struct RowList {
    pub items: Vec<PosterItem>,
    pub selected_index: usize,
    pub is_active: bool,

    // SCROLL STATE 📜
    pub scroll_x: f32,        // Current visual position (Lerped)
    pub target_scroll_x: f32, // Where we want to go

    // NEW: Vertical Scroll (Received from Parent)
    pub offset_y: f32,
}

impl RowList {
    pub fn new(y_start: f32) -> Self {
        let mut items = Vec::new();

        for i in 0..10 {
            let x_axis = 50.0 + (i as f32 * 320.00);
            let y_axis = y_start;
            let img_width = 300.0;
            let img_height = 200.0;

            let img_src = if i % 2 == 0 {
                "https://m.media-amazon.com/images/M/MV5BNGI0MDI4NjEtOWU3ZS00ODQyLWFhYTgtNGYxM2ZkM2Q2YjE3XkEyXkFqcGc@._V1_.jpg"
            } else {
                "https://m.media-amazon.com/images/M/MV5BNDE0MGFkYzktYTMyNS00Mjk1LWI3YzEtYWYxMzAxNTI2YmUyXkEyXkFqcGc@._V1_QL75_UX480_.jpg"
            };
            //create each item here
            let item = PosterItem::new(x_axis, y_axis, img_width, img_height, img_src, true);
            items.push(item);
        }

        Self {
            items,
            selected_index: 0,
            is_active: false,

            // Start at 0
            scroll_x: 0.0,
            target_scroll_x: 0.0,
            offset_y: 0.0, // Default 0
        }
    }

    // 1. INPUT HANDLER
    pub fn handle_input(&mut self, key_code: u32) {
        if !self.is_active {
            return;
        }

        match key_code {
            37 => {
                // LEFT
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            39 => {
                // RIGHT
                if self.selected_index < self.items.len() - 1 {
                    self.selected_index += 1;
                }
            }
            _ => {}
        }

        // --- SCROLL CALCULATION ---
        // Rule: If we pass item 4, start shifting left!
        // Item Width + Gap = 220.0

        if self.selected_index > 4 {
            // If we are at index 5, we want to shift 1 item to the left.
            // If we are at index 6, shift 2 items...
            let shift_count = (self.selected_index - 4) as f32;
            self.target_scroll_x = -(shift_count * 320.0);
        } else {
            // If we are at index 0-4, reset to start
            self.target_scroll_x = 0.0;
        }
    }

    // 2. LOAD ASSETS
    pub fn load_assets(
        &mut self,
        context: &WebGlRenderingContext,
        manager: &mut TextureManager,
    ) -> Result<(), JsValue> {
        for item in &mut self.items {
            item.init_buffer(context).unwrap_or_else(|e| {
                web_sys::console::error_1(&format!("Buffer error: {}", e).into())
            });
            let shared_assets = manager.get_texture(context, &item.src)?;
            item.set_texture(shared_assets.texture, shared_assets.image);
        }
        Ok(())
    }

    // 3. UPDATE LOOP
    pub fn update(&mut self, context: &WebGlRenderingContext) {
        // --- SCROLL ANIMATION (LERP) ---
        let diff = self.target_scroll_x - self.scroll_x;

        // Use a nice smooth speed (0.1)
        if diff.abs() > 0.5 {
            self.scroll_x += diff * 0.1;
        } else {
            self.scroll_x = self.target_scroll_x; // Snap when close
        }

        for (i, item) in self.items.iter_mut().enumerate() {
            // Update Selection
            let should_be_selected = self.is_active && (i == self.selected_index);
            if item.is_selected != should_be_selected {
                item.is_selected = should_be_selected;
            }

            // PUSH SCROLL TO ITEM
            // We give the scroll offset to the item so it knows where to draw
            item.offset_x = self.scroll_x;
            item.offset_y = self.offset_y; // Vertical (From Parent)

            // Call Item Update
            item.update(context);
        }
    }

    // 4. DRAW LOOP
    pub fn draw(&self, context: &WebGlRenderingContext) {
        for item in &self.items {
            if let (Some(texture), Some(buffer)) = (&item.texture, &item.buffer) {
                context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(buffer));

                context.vertex_attrib_pointer_with_i32(
                    0,
                    2,
                    WebGlRenderingContext::FLOAT,
                    false,
                    16,
                    0,
                );
                context.vertex_attrib_pointer_with_i32(
                    1,
                    2,
                    WebGlRenderingContext::FLOAT,
                    false,
                    16,
                    8,
                );

                context.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(texture));
                context.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 6);
            }
        }
    }
}
