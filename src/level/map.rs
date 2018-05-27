use na::Vector2;
use physics::{UpBorder, LeftBorder, RightBorder, DownBorder};

const DEFAULT_WIDTH: u32 = 320;
const DEFAULT_HEIGHT: u32 = 200;

/// Data type for the game map, containing moving things n stuff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    width: u32,
    height: u32,
}

impl Default for Map {
    fn default() -> Map {
        Map {
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
        }
    }
}

impl Map {
    pub fn new(width: u32, height: u32) -> Self {
        assert!(width > 0);
        assert!(height > 0);
        Map { width, height }
    }

    pub fn dimensions_f32(&self) -> Vector2<f32> {
        Vector2::new(self.width as f32, self.height as f32)
    }

    pub fn expand_to_fit(&mut self, dim: Vector2<i32>) {
        if dim[0] > 0 {
            self.width = u32::max(self.width, dim[0] as u32);
        }
        if dim[1] > 0 {
            self.height = u32::max(self.height, dim[1] as u32);
        }
    }

    pub fn up_border(&self) -> UpBorder {
        UpBorder(0.)
    }

    pub fn down_border(&self) -> DownBorder {
        DownBorder(self.height as f32)
    }

    pub fn left_border(&self) -> LeftBorder {
        LeftBorder(0.)
    }

    pub fn right_border(&self) -> RightBorder {
        RightBorder(self.width as f32)
    }
}
