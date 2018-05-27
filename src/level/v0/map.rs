use na::Vector2;
use level::map::Map as CurrentMap;

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

    pub fn upgrade(self) -> CurrentMap {
        CurrentMap::new(self.width as u32, self.height as u32)
    }
}
