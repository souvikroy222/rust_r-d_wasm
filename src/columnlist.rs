use crate::rowlist::RowList;
use crate::texture_manager::TextureManager;
use wasm_bindgen::JsValue;
use web_sys::WebGlRenderingContext;

pub struct ColumnList {
    pub rows: Vec<RowList>,
    pub selected_row_index: usize, // Which row is currently active?

    // VERTICAL SCROLL STATE 📜
    pub scroll_y: f32,
    pub target_scroll_y: f32,
}

impl ColumnList {
    pub fn new() -> Self {
        let mut rows = Vec::new();

        // ⚠️ STRESS TEST SETTING:
        // You can change "0..20" to "0..1000" now!
        // Because we use TextureManager, 1000 rows (10,000 items) will only use ~20MB RAM.
        for i in 0..20 {
            // Calculate Y position: Start at 50, go down 220px per row
            let y_start = 50.0 + (i as f32 * 480.0);

            // Create a row at that specific Y height
            let row = RowList::new(y_start);

            rows.push(row);
        }

        let mut list = Self {
            rows,
            selected_row_index: 0,

            // Start at 0
            scroll_y: 0.0,
            target_scroll_y: 0.0,
        };

        // Activate the first row by default
        if let Some(first_row) = list.rows.get_mut(0) {
            first_row.is_active = true;
        }

        list
    }

    // 1. LOAD ASSETS (Passes Manager down the chain)
    pub fn load_assets(
        &mut self,
        context: &WebGlRenderingContext,
        manager: &mut TextureManager,
    ) -> Result<(), JsValue> {
        for row in &mut self.rows {
            // We pass the manager so rows can request SHARED textures
            row.load_assets(context, manager)?;
        }
        Ok(())
    }

    // 2. INPUT HANDLER (Up/Down Logic)
    pub fn handle_input(&mut self, key_code: u32) {
        match key_code {
            38 => {
                // UP ARROW
                if self.selected_row_index > 0 {
                    // A. Deactivate old row (Visuals: selected item shrinks)
                    self.rows[self.selected_row_index].is_active = false;

                    // B. Move Selection
                    self.selected_row_index -= 1;

                    // C. Activate new row (Visuals: saved item grows)
                    self.rows[self.selected_row_index].is_active = true;
                }
            }
            40 => {
                // DOWN ARROW
                if self.selected_row_index < self.rows.len() - 1 {
                    // A. Deactivate old row
                    self.rows[self.selected_row_index].is_active = false;

                    // B. Move Selection
                    self.selected_row_index += 1;

                    // C. Activate new row
                    self.rows[self.selected_row_index].is_active = true;
                }
            }
            // LEFT (37) or RIGHT (39) -> Delegate to the Active Row
            37 | 39 => {
                if let Some(row) = self.rows.get_mut(self.selected_row_index) {
                    row.handle_input(key_code);
                }
            }
            _ => {}
        }

        // --- VERTICAL SCROLL CALCULATION ---
        // Rule: If we go past index 1 (the 2nd row), start scrolling up.
        // Row Height = 220.0

        if self.selected_row_index > 1 {
            // If index is 2, shift up by 1 row height (220)
            // If index is 3, shift up by 2 row heights (440)
            let shift_count = (self.selected_row_index - 1) as f32;
            self.target_scroll_y = -(shift_count * 480.0);
        } else {
            // If index is 0 or 1, stay at top
            self.target_scroll_y = 0.0;
        }
    }

    // 3. UPDATE LOOP
    pub fn update(&mut self, context: &WebGlRenderingContext) {
        // 1. Vertical Lerp Logic
        let diff = self.target_scroll_y - self.scroll_y;
        if diff.abs() > 0.5 {
            self.scroll_y += diff * 0.1; // Smooth scroll
        } else {
            self.scroll_y = self.target_scroll_y;
        }

        for row in &mut self.rows {
            // Give every row the global vertical offset
            row.offset_y = self.scroll_y;
            row.update(context);
        }
    }

    // 4. DRAW LOOP
    pub fn draw(&self, context: &WebGlRenderingContext) {
        // Optimization: In a real engine, you'd only draw rows visible on screen!
        // For now, we draw everything.
        for row in &self.rows {
            row.draw(context);
        }
    }

    // 5. HELPER (For changing images dynamically)
    // pub fn get_item_mut(&mut self, row_index: usize, item_index: usize) -> Option<&mut crate::item::Item> {
    //     if let Some(row) = self.rows.get_mut(row_index) {
    //         return row.items.get_mut(item_index);
    //     }
    //     None
    // }
}
