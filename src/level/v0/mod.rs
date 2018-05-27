//! Module for legacy level version: v1
pub mod info;
pub mod map;
use self::info::*;
use self::map::Map;
use super::{
    GameLevel as CurrentGameLevel, GameLevelBuilder as CurrentGameLevelBuilder, CURRENT_VERSION,
};
use na::Vector2;
use util::DynResult;

/// Game level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLevel {
    name: String,
    map: Map,
    ball_pos: Vector2<f32>,
    #[serde(default)]
    walls: Vec<WallInfo>,
    #[serde(default)]
    pumps: Vec<PumpInfo>,
    #[serde(default)]
    mines: Vec<MineInfo>,
    #[serde(default)]
    gems: Vec<GemInfo>,
    #[serde(default)]
    finish: Option<FinishInfo>,
}

impl Default for GameLevel {
    fn default() -> GameLevel {
        GameLevel {
            name: "No Name".to_string(),
            ball_pos: [36., 36.].into(),
            map: Map::default(),
            walls: Vec::new(),
            pumps: Vec::new(),
            mines: Vec::new(),
            gems: Vec::new(),
            finish: None,
        }
    }
}

impl GameLevel {
    pub fn upgrade(self) -> DynResult<CurrentGameLevel> {
        let mut lvl = CurrentGameLevelBuilder::default();
        lvl.name(self.name);
        lvl.version(CURRENT_VERSION.to_string());
        lvl.ball_pos(Vector2::from([
            self.ball_pos[0] as i32,
            self.ball_pos[1] as i32,
        ]));
        lvl.map(self.map.upgrade());
        lvl.walls(self.walls.into_iter().map(|x| x.upgrade()).collect());
        lvl.pumps(self.pumps.into_iter().map(|x| x.upgrade()).collect());
        lvl.mines(self.mines.into_iter().map(|x| x.upgrade()).collect());
        lvl.gems(self.gems.into_iter().map(|x| x.upgrade()).collect());
        lvl.finish(self.finish.map(|x| x.upgrade()));

        let lvl = lvl.build().map_err(::failure::err_msg)?;
        Ok(lvl)
    }
}
