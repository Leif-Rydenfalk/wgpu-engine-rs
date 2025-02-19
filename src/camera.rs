// In camera.rs, replace the entire file:
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
            zoom: 1.0,
            dragging: false,
            last_mouse_pos: None,
            window_size: [800.0, 600.0], // Default size
        }
    }

    pub fn update_window_size(&mut self, width: f32, height: f32) {
        self.window_size = [width, height];
    }

    pub fn get_view_matrix(&self) -> [[f32; 4]; 4] {
        let scale = self.zoom;
        [
            [scale, 0.0, 0.0, 0.0],
            [0.0, scale, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [
                -self.position[0] * scale,
                -self.position[1] * scale,
                0.0,
                1.0,
            ],
        ]
    }

    // Convert screen coordinates to world coordinates
    fn screen_to_world(&self, screen_pos: [f32; 2]) -> [f32; 2] {
        [
            (screen_pos[0] - self.window_size[0] / 2.0) / self.zoom + self.position[0],
            (screen_pos[1] - self.window_size[1] / 2.0) / self.zoom + self.position[1],
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
                let delta_x = (position[0] - last_pos[0]) / self.zoom;
                let delta_y = (position[1] - last_pos[1]) / self.zoom;
                self.position[0] -= delta_x;
                self.position[1] -= delta_y;
            }
            self.last_mouse_pos = Some(position);
        }
    }

    pub fn handle_scroll(&mut self, delta: f32, cursor_pos: [f32; 2]) {
        const MIN_ZOOM: f32 = 0.1;
        const MAX_ZOOM: f32 = 10.0;
        const ZOOM_SPEED: f32 = 0.1;

        // Convert cursor position to world space before zoom
        let world_pos_before = self.screen_to_world(cursor_pos);

        // Calculate new zoom
        let new_zoom = (self.zoom * (1.0 + delta * ZOOM_SPEED)).clamp(MIN_ZOOM, MAX_ZOOM);

        // Apply new zoom
        self.zoom = new_zoom;

        // Convert cursor position to world space after zoom
        let world_pos_after = self.screen_to_world(cursor_pos);

        // Adjust position to maintain cursor world position
        self.position[0] += world_pos_after[0] - world_pos_before[0];
        self.position[1] += world_pos_after[1] - world_pos_before[1];
    }
}
