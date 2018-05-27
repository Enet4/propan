use na::Vector2;

#[inline]
pub fn vector_to_i32(v: Vector2<f32>) -> Vector2<i32> {
    Vector2::from([v[0] as i32, v[1] as i32])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GemInfo {
    pub pos: Vector2<f32>,
}

impl GemInfo {
    pub fn upgrade(self) -> ::level::info::GemInfo {
        ::level::info::GemInfo {
            pos: vector_to_i32(self.pos),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpInfo {
    pub pos: Vector2<f32>,
}

impl PumpInfo {
    pub fn upgrade(self) -> ::level::info::PumpInfo {
        ::level::info::PumpInfo {
            pos: vector_to_i32(self.pos),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MineInfo {
    pub pos: Vector2<f32>,
}

impl MineInfo {
    pub fn upgrade(self) -> ::level::info::MineInfo {
        ::level::info::MineInfo {
            pos: vector_to_i32(self.pos),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WallInfo {
    pub pos: Vector2<f32>,
    pub dim: Vector2<f32>,
    #[serde(default)]
    pub texture_id: u32,
}

impl WallInfo {
    pub fn upgrade(self) -> ::level::info::WallInfo {
        ::level::info::WallInfo {
            pos: vector_to_i32(self.pos),
            dim: vector_to_i32(self.dim),
            texture_id: self.texture_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinishInfo {
    pub pos: Vector2<f32>,
    #[serde(default)]
    pub gems_required: u32,
}

impl FinishInfo {
    pub fn upgrade(self) -> ::level::info::FinishInfo {
        ::level::info::FinishInfo {
            pos: vector_to_i32(self.pos),
            gems_required: self.gems_required,
        }
    }
}
