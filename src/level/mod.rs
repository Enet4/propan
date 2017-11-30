use std::ffi::OsStr;
use std::fs::{File, read_dir};
use std::path::{Path, PathBuf};
use controller::LevelId;
use itertools::process_results;
use na::Vector2;
use serde_json::{from_reader, to_writer_pretty as to_writer};

pub mod info;
pub mod map;
pub use self::map::Map;

use self::info::*;

type DynResult<T> = Result<T, Box<(::std::error::Error + 'static)>>;

pub fn load_all_level_paths<P: AsRef<Path>>(dir: P) -> DynResult<Vec<PathBuf>> {
    let entries = read_dir(dir)?;

    let mut x: Vec<PathBuf> = process_results(entries, |iter| {
        iter.map(|e| e.path())
            .filter(|p| p.is_file())
            .filter(|p| p.extension() == Some(OsStr::new("json")))
            .map(|p| p.to_path_buf())
            .collect()
    })?;
    x.sort();
    Ok(x)
}

pub fn load_all_levels<P: AsRef<Path>>(dir: P) -> DynResult<Vec<GameLevel>> {
    load_all_level_paths(dir)?.into_iter()
        .map(GameLevel::load)
        .collect()
}

/// Game level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLevel {
    name: String,
    map: Map,
    ball_pos: Vector2<f32>,
    #[serde(default)] walls: Vec<WallInfo>,
    #[serde(default)] pumps: Vec<PumpInfo>,
    #[serde(default)] mines: Vec<MineInfo>,
    #[serde(default)] gems: Vec<GemInfo>,
    #[serde(default)] finish: Option<FinishInfo>,
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

    pub fn load<P: AsRef<Path>>(path: P) -> DynResult<Self> {
        let file = File::open(path)?;
        let game = from_reader(file)?;
        Ok(game)
    }

    pub fn load_by_index<P: AsRef<Path>>(dir: P, id: LevelId) -> DynResult<Self> {
        let paths = load_all_level_paths(dir)?;
        match paths.get(id as usize) {
            Some(p) => GameLevel::load(p),
            None => Err("No such Level".into()),
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> DynResult<()> {
        let file = File::create(path)?;
        to_writer(file, self).map_err(From::from)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn map_mut(&mut self) -> &mut Map {
        &mut self.map
    }

    pub fn ball_position(&self) -> Vector2<f32> {
        self.ball_pos
    }

    pub fn ball_position_mut(&mut self) -> &mut Vector2<f32> {
        &mut self.ball_pos
    }

    pub fn walls(&self) -> &[WallInfo] {
        &self.walls
    }

    pub fn walls_mut(&mut self) -> &mut Vec<WallInfo> {
        &mut self.walls
    }

    pub fn pumps(&self) -> &[PumpInfo] {
        &self.pumps
    }

    pub fn pumps_mut(&mut self) -> &mut Vec<PumpInfo> {
        &mut self.pumps
    }

    pub fn mines(&self) -> &[MineInfo] {
        &self.mines
    }

    pub fn mines_mut(&mut self) -> &mut Vec<MineInfo> {
        &mut self.mines
    }

    pub fn gems(&self) -> &[GemInfo] {
        &self.gems
    }

    pub fn gems_mut(&mut self) -> &mut Vec<GemInfo> {
        &mut self.gems
    }

    pub fn finish_flag(&self) -> Option<&FinishInfo> {
        self.finish.as_ref()
    }

    pub fn finish_flag_mut(&mut self) -> Option<&mut FinishInfo> {
        self.finish.as_mut()
    }

    pub fn set_finish_flag(&mut self, f: FinishInfo) {
        self.finish = Some(f);
    }

    pub fn clear_finish_flag(&mut self) {
        self.finish = None;
    }
}
