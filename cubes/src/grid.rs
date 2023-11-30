use geo::{coord, Coord};

#[derive(Default)]
pub struct Grid {
    // Isometric grid
    pub width: f64,
    pub height: f64,
    pub device_pixel_ratio: f64,

    pub light_source: LightSource,
}

pub struct LightSource {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Default for LightSource {
    fn default() -> Self {
        Self {
            x: -300.25,
            y: 300.25,
            z: 600.0,
        }
    }
}

impl Grid {
    pub fn project(&self, x: f64, y: f64, z: f64) -> Coord {
        let (left_offset, top_offset) = self.offset();

        let x = x - y;
        let y = ((x + y * 2.0) / 2.0) - z;

        coord! { x: x + left_offset, y: y + top_offset }
    }

    pub fn cartesian_to_iso(&self, x: f64, y: f64) -> (f64, f64) {
        let (left_offset, top_offset) = self.offset();

        let x = (x - left_offset) * self.device_pixel_ratio / 2.0;
        let y = (y - top_offset) * self.device_pixel_ratio;

        ((x + y), (y - x))
    }

    fn offset(&self) -> (f64, f64) {
        let left_offset = self.width / 2.0;
        let top_offset = (self.height - 600.0) / 2.0;

        (left_offset, top_offset)
    }

    pub fn update_dimensions(&mut self, width: f64, height: f64, device_pixel_ratio: f64) -> bool {
        let width = width * device_pixel_ratio;
        let height = height * device_pixel_ratio;

        let changed = self.width != width || self.height != height;

        self.width = width;
        self.height = height;
        self.device_pixel_ratio = device_pixel_ratio;

        changed
    }
}
