//! # Rotating 3D Icosahedron
//!
//! A simple 3D wireframe icosahedron that rotates on screen.
//! Uses 2D projection of 3D coordinates.

use egui::{Color32, Painter, Pos2, FontId, FontFamily};
use std::f32::consts::PI;

/// Rotating 3D icosahedron renderer
pub struct RotatingCube {
    /// Rotation angle around Y axis (in radians)
    rotation_y: f32,
    /// Rotation angle around X axis (in radians)
    rotation_x: f32,
    /// Size of the icosahedron
    pub size: f32,
}

impl RotatingCube {
    /// Create a new rotating icosahedron
    pub fn new() -> Self {
        Self {
            rotation_y: 0.0,
            rotation_x: 0.0,
            size: 150.0, // Bigger default size
        }
    }

    /// Update the cube rotation (call every frame)
    pub fn update(&mut self, delta_time: f32) {
        // Rotate around Y axis (vertical)
        self.rotation_y += delta_time * 1.0; // 1 radian per second
        // Rotate around X axis (horizontal) - slower
        self.rotation_x += delta_time * 0.5; // 0.5 radian per second
        
        // Keep angles in reasonable range to avoid overflow
        if self.rotation_y > 2.0 * PI {
            self.rotation_y -= 2.0 * PI;
        }
        if self.rotation_x > 2.0 * PI {
            self.rotation_x -= 2.0 * PI;
        }
    }

    /// Render the cube
    pub fn render(&self, painter: &Painter, center: Pos2, size: f32) {
        self.render_with_size(painter, center, size);
    }

    /// Render the icosahedron with specific size
    fn render_with_size(&self, painter: &Painter, center: Pos2, size: f32) {
        // Golden ratio constant
        let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;
        let radius = size * 0.5;
        
        // Define icosahedron vertices using golden ratio
        // Icosahedron has 12 vertices arranged in 3 orthogonal golden rectangles
        // Using standard icosahedron coordinates
        let t = radius / phi.sqrt();
        let vertices_3d = [
            // Top and bottom vertices
            [0.0, radius, 0.0],           // 0: top
            [0.0, -radius, 0.0],          // 1: bottom
            
            // First golden rectangle (in XY plane, rotated)
            [t, t / phi, 0.0],            // 2
            [-t, t / phi, 0.0],           // 3
            [t, -t / phi, 0.0],           // 4
            [-t, -t / phi, 0.0],          // 5
            
            // Second golden rectangle (in XZ plane)
            [t / phi, 0.0, t],            // 6
            [-t / phi, 0.0, t],           // 7
            [t / phi, 0.0, -t],           // 8
            [-t / phi, 0.0, -t],          // 9
            
            // Third golden rectangle (in YZ plane)
            [0.0, t / phi, t],            // 10
            [0.0, -t / phi, t],           // 11
        ];

        // Rotate vertices
        let rotated_vertices: Vec<[f32; 3]> = vertices_3d
            .iter()
            .map(|&v| self.rotate_vertex(v))
            .collect();

        // Project to 2D
        let vertices_2d: Vec<Pos2> = rotated_vertices
            .iter()
            .map(|&v| self.project_to_2d(v, center))
            .collect();

        // Define edges - icosahedron has 30 edges
        // Each vertex connects to 5 others
        // Using proper icosahedron connectivity
        // Vertices: 0=top, 1=bottom, 2-5=first rectangle, 6-9=second rectangle, 10-11=third rectangle
        let edges = [
            // Top vertex (0) connects to 5 vertices: 2, 3, 6, 7, 10
            (0, 2), (0, 3), (0, 6), (0, 7), (0, 10),
            // Bottom vertex (1) connects to 5 vertices: 4, 5, 8, 9, 11
            (1, 4), (1, 5), (1, 8), (1, 9), (1, 11),
            // First rectangle (XY plane) edges: 2-3, 4-5, 2-4, 3-5
            (2, 3), (4, 5), (2, 4), (3, 5),
            // Connect first rectangle to second rectangle: 2-6, 3-7, 4-8, 5-9
            (2, 6), (3, 7), (4, 8), (5, 9),
            // Connect first rectangle to third rectangle: 2-10, 3-10, 4-11, 5-11
            (2, 10), (3, 10), (4, 11), (5, 11),
            // Second rectangle (XZ plane) edges: 6-7, 8-9
            (6, 7), (8, 9),
            // Connect second rectangle to third rectangle: 6-10, 7-10, 8-11, 9-11
            (6, 10), (7, 10), (8, 11), (9, 11),
            // Connect second rectangle vertices to each other: 6-8, 7-9
            (6, 8), (7, 9),
            // Additional connections to complete icosahedron: 6-2, 7-3, 8-4, 9-5
            // These are already covered above, but ensuring all 30 edges are present
        ];

        // Draw edges with thicker lines in bright white
        let color = Color32::from_rgb(255, 255, 255); // Bright white
        let stroke_width = 4.0; // Thicker lines
        
        for &(i, j) in &edges {
            if i < vertices_2d.len() && j < vertices_2d.len() {
                painter.line_segment([vertices_2d[i], vertices_2d[j]], (stroke_width, color));
            }
        }

        // Draw "XF" text in RED capital letters at the center, rotating opposite to the shape
        let xf_font = FontId::new(size * 0.4, FontFamily::Proportional); // Scale font size with icosahedron
        let red_color = Color32::from_rgb(204, 0, 0); // Xterminal red
        
        // Rotate text in opposite direction to the icosahedron
        // If icosahedron rotates +Y and +X, text rotates -Y and -X
        let text_rotation = -(self.rotation_y + self.rotation_x);
        
        // Calculate rotated positions for "X" and "F" to create rotation effect
        let cos_rot = text_rotation.cos();
        let sin_rot = text_rotation.sin();
        let char_spacing = size * 0.15; // Spacing between X and F
        
        // Calculate positions for X and F with rotation
        // X position (left of center)
        let x_offset = -char_spacing * 0.5;
        let x_rotated_x = center.x + x_offset * cos_rot - 0.0 * sin_rot;
        let x_rotated_y = center.y + x_offset * sin_rot + 0.0 * cos_rot;
        let x_pos = Pos2::new(x_rotated_x, x_rotated_y);
        
        // F position (right of center)
        let f_offset = char_spacing * 0.5;
        let f_rotated_x = center.x + f_offset * cos_rot - 0.0 * sin_rot;
        let f_rotated_y = center.y + f_offset * sin_rot + 0.0 * cos_rot;
        let f_pos = Pos2::new(f_rotated_x, f_rotated_y);
        
        // Draw X and F separately with rotation effect
        painter.text(x_pos, egui::Align2::CENTER_CENTER, "X", xf_font.clone(), red_color);
        painter.text(f_pos, egui::Align2::CENTER_CENTER, "F", xf_font, red_color);
    }

    /// Rotate a 3D vertex around Y and X axes
    fn rotate_vertex(&self, vertex: [f32; 3]) -> [f32; 3] {
        let [x, y, z] = vertex;
        
        // Rotate around Y axis
        let cos_y = self.rotation_y.cos();
        let sin_y = self.rotation_y.sin();
        let x1 = x * cos_y - z * sin_y;
        let z1 = x * sin_y + z * cos_y;
        
        // Rotate around X axis
        let cos_x = self.rotation_x.cos();
        let sin_x = self.rotation_x.sin();
        let y1 = y * cos_x - z1 * sin_x;
        let z2 = y * sin_x + z1 * cos_x;
        
        [x1, y1, z2]
    }

    /// Project 3D point to 2D screen coordinates
    fn project_to_2d(&self, vertex: [f32; 3], center: Pos2) -> Pos2 {
        let [x, y, z] = vertex;
        
        // Simple perspective projection
        // Distance from camera
        let distance = 300.0;
        let scale = distance / (distance + z);
        
        // Project to 2D
        let x_2d = x * scale;
        let y_2d = y * scale;
        
        // Translate to center
        Pos2::new(center.x + x_2d, center.y - y_2d) // Negative y because screen Y is down
    }

    /// Get current rotation angles (for debugging)
    pub fn get_rotation(&self) -> (f32, f32) {
        (self.rotation_x, self.rotation_y)
    }

    /// Reset rotation
    pub fn reset(&mut self) {
        self.rotation_x = 0.0;
        self.rotation_y = 0.0;
    }
}

impl Default for RotatingCube {
    fn default() -> Self {
        Self::new()
    }
}

