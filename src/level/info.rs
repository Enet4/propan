use na::Vector2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GemInfo {
    pub pos: Vector2<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpInfo {
    pub pos: Vector2<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MineInfo {
    pub pos: Vector2<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WallInfo {
    pub pos: Vector2<f32>,
    pub dim: Vector2<f32>,
    #[serde(default)] pub texture_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinishInfo {
    pub pos: Vector2<f32>,
    #[serde(default)] pub gems_required: u32,
}