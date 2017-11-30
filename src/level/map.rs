use na::Vector2;
use physics::{UpBorder, LeftBorder, RightBorder, DownBorder};

const DEFAULT_WIDTH: f32 = 320.;
const DEFAULT_HEIGHT: f32 = 200.;

/// Data type for the game map, containing moving things n stuff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    width: f32,
    height: f32,
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
    pub fn dimensions(&self) -> Vector2<f32> {
        [self.width, self.height].into()
    }

    pub fn expand_to_fit(&mut self, dim: Vector2<f32>) {
        self.width = f32::max(self.width, dim[0]);
        self.height = f32::max(self.height, dim[1]);
    }

    pub fn up_border(&self) -> UpBorder {
        UpBorder(0.)
    }

    pub fn down_border(&self) -> DownBorder {
        DownBorder(self.height)
    }

    pub fn left_border(&self) -> LeftBorder {
        LeftBorder(0.)
    }

    pub fn right_border(&self) -> RightBorder {
        RightBorder(self.width)
    }
}
