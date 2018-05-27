use std::ffi::OsStr;
use std::fs::{File, read_dir};
use std::path::{Path, PathBuf};
use controller::LevelId;
use itertools::process_results;
use na::Vector2;
use serde_json::{from_reader, to_writer_pretty as to_writer};

mod v0;
pub mod info;
pub mod map;
pub use self::map::Map;

use self::info::*;

pub const CURRENT_VERSION: &str = "1.0";

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

pub fn load_all_level_headers<P: AsRef<Path>>(dir: P) -> DynResult<Vec<GameLevelHeader>> {
    load_all_level_paths(dir)?.into_iter()
        .map(GameLevelHeader::from_file)
        .collect()
}

/// Serializable data type with the bare minimum compatible subset of game
/// level versions. This is used to check which level version should be
/// loaded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLevelHeader {
    name: String,
    #[serde(default = "GameLevelHeader::default_version")]
    version: String,
}

impl Default for GameLevelHeader {
    fn default() -> Self {
        GameLevelHeader {
            name: "No Name".to_string(),
            version: CURRENT_VERSION.to_string(),
        }
    }
}

impl GameLevelHeader {
    pub fn default_version() -> String {
        "0.1".to_string()
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> DynResult<Self> {
        let file = File::open(path)?;
        let game = from_reader(file)?;
        Ok(game)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn set_version<V: Into<String>>(&mut self, v: V) {
        self.version = v.into();
    }
}

/// Game level.
#[derive(Debug, Clone, Builder, Serialize, Deserialize)]
pub struct GameLevel {
    name: String,
    version: String,
    map: Map,
    ball_pos: Vector2<i32>,
    #[serde(default)] walls: Vec<WallInfo>,
    #[serde(default)] pumps: Vec<PumpInfo>,
    #[serde(default)] mines: Vec<MineInfo>,
    #[serde(default)] gems: Vec<GemInfo>,
    #[serde(default)] finish: Option<FinishInfo>,
}

impl Default for GameLevel {
    fn default() -> GameLevel {
        let header = GameLevelHeader::default();
        GameLevel {
            name: header.name,
            version: header.version,
            ball_pos: [36, 36].into(),
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
        // read as a header first
        let file = File::open(&path)?;
        let header: GameLevelHeader = from_reader(file)?;
        match header.version() {
            "0.1" => {
                // read in legacy format, then convert to new format
                let file = File::open(path)?;
                let game: v0::GameLevel = from_reader(file)?;
                game.upgrade()
            }
            "1.0" => {
                let file = File::open(path)?;
                let game: GameLevel = from_reader(file)?;
                Ok(game)
            }
            v => Err(format!("Unsupported level version {}", v).into())
        }
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

    pub fn set_name<T: Into<String>>(&mut self, name: T) {
        self.name = name.into();
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn set_version<V: Into<String>>(&mut self, v: V) {
        self.version = v.into();
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn map_mut(&mut self) -> &mut Map {
        &mut self.map
    }

    pub fn ball_position(&self) -> Vector2<f32> {
        Vector2::new(self.ball_pos[0] as f32, self.ball_pos[1] as f32)
    }

    pub fn set_ball_position(&mut self, pos: Vector2<f32>) {
        self.ball_pos[0] = pos[0] as i32;
        self.ball_pos[1] = pos[1] as i32;
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
