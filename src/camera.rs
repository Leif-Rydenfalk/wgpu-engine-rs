use winit::event::MouseButton;

pub struct Camera {
    pub position: [f32; 2],
    pub zoom: f32,
    pub dragging: bool,
    pub last_mouse_pos: Option<[f32; 2]>,
    window_size: [f32; 2],
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: [0.0, 0.0],
            zoom: 1.0 / 1000.0,
            dragging: false,
            last_mouse_pos: None,
            window_size: [800.0, 600.0],
        }
    }

    pub fn update_window_size(&mut self, width: f32, height: f32) {
        self.window_size = [width, height];
    }

    pub fn get_view_matrix(&self) -> [[f32; 4]; 4] {
        let aspect_ratio = self.window_size[0] / self.window_size[1];
        let scale_x = self.zoom / aspect_ratio;
        let scale_y = self.zoom;
        
        [
            [scale_x, 0.0, 0.0, 0.0],
            [0.0, scale_y, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-self.position[0] * scale_x, -self.position[1] * scale_y, 0.0, 1.0],
        ]
    }

    fn screen_to_world(&self, screen_pos: [f32; 2]) -> [f32; 2] {
        let aspect_ratio = self.window_size[0] / self.window_size[1];
        
        // Convert screen coordinates to normalized device coordinates (-1 to 1)
        let ndc_x = (2.0 * screen_pos[0] / self.window_size[0]) - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pos[1] / self.window_size[1]); // Flip Y coordinate
        
        // Convert to world space
        [
            ndc_x * (1.0 / self.zoom) * aspect_ratio + self.position[0],
            ndc_y * (1.0 / self.zoom) + self.position[1],
        ]
    }

    pub fn handle_mouse_press(&mut self, button: MouseButton, position: [f32; 2]) {
        if button == MouseButton::Left {
            self.dragging = true;
            self.last_mouse_pos = Some(position);
        }
    }

    pub fn handle_mouse_release(&mut self, button: MouseButton) {
        if button == MouseButton::Left {
            self.dragging = false;
            self.last_mouse_pos = None;
        }
    }

    pub fn handle_mouse_move(&mut self, position: [f32; 2]) {
        if self.dragging {
            if let Some(last_pos) = self.last_mouse_pos {
                let aspect_ratio = self.window_size[0] / self.window_size[1];
                
                // Calculate movement in screen space
                let delta_x = (position[0] - last_pos[0]) / self.window_size[0];
                let delta_y = (position[1] - last_pos[1]) / self.window_size[1];
                
                // Convert to world space movement
                self.position[0] -= 2.0 * delta_x * (1.0 / self.zoom) * aspect_ratio;
                self.position[1] += 2.0 * delta_y * (1.0 / self.zoom); // Note the += because Y is flipped
            }
            self.last_mouse_pos = Some(position);
        }
    }

    pub fn handle_scroll(&mut self, delta: f32, cursor_pos: [f32; 2]) {
        const MIN_ZOOM: f32 = 0.001;
        const MAX_ZOOM: f32 = 1000000.0;
        const ZOOM_SPEED: f32 = 0.5;

        // Get world position of cursor before zoom
        let world_pos_before = self.screen_to_world(cursor_pos);

        // Calculate new zoom level
        let zoom_factor = 1.0 + delta * ZOOM_SPEED;
        let new_zoom = (self.zoom * zoom_factor).clamp(MIN_ZOOM, MAX_ZOOM);
        
        // Apply new zoom
        self.zoom = new_zoom;

        // Get world position of cursor after zoom
        let world_pos_after = self.screen_to_world(cursor_pos);

        // Adjust camera position to keep cursor point stationary
        self.position[0] += world_pos_before[0] - world_pos_after[0];
        self.position[1] += world_pos_before[1] - world_pos_after[1];
    }
}