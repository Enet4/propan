use std::path::{Path, PathBuf};
use na::Vector2;
use camera::Camera;
use level::*;
use level::info::*;
use game::{entities, wall};
use game::ball::{Ball, BallController, BALL_DEFAULT_SIZE};
use graphics::{clear, ellipse, rectangle, Context, Graphics, Transformed};
use graphics::character::CharacterCache;
use piston::input::{GenericEvent, UpdateArgs};
use controller::{Controller, ControllerAction};
use resource::{GameTexture, ResourceManage, Result, SpriteAssetId, SpriteManage};
use physics::{Collidable, SimpleCollidable};

mod placeholder;
use self::placeholder::*;

const VERSION: &str = "1.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum EditState {
    Idle,
    Panning,
}

impl Default for EditState {
    fn default() -> Self {
        EditState::Idle
    }
}

pub struct LevelEditorController<R>
where
    R: ResourceManage + Copy,
{
    level: GameLevel,
    res: R,
    ball: BallController<R>,
    walls: Vec<wall::Wall<R>>,
    pumps: Vec<entities::Pump<R>>,
    mines: Vec<entities::Mine<R>>,
    gems: Vec<entities::Gem<R>>,
    finish: Option<entities::Finish<R>>,
    camera: Camera,
    // the physical cursor, relative to display
    cursor: Vector2<f32>,
    // the logical cursor, relative to map
    logical_cursor: Vector2<f32>,
    state: EditState,
    placeholder: ObjectPlaceholder,
}


/// Game level controller.
impl<R> LevelEditorController<R>
where
    R: ResourceManage + Copy,
{
    pub fn new(resource_manager: R) -> Result<Self> {
        let mut lvl = GameLevel::default();
        lvl.set_version(VERSION); 
        LevelEditorController::with_level(lvl, resource_manager)
    }

    pub fn load<P: AsRef<Path>>(path: P, resource_manager: R) -> Result<Self> {
        let level = GameLevel::load(path).unwrap();
        LevelEditorController::with_level(level, resource_manager)
    }

    fn with_level(level: GameLevel, resource_manager: R) -> Result<Self> {
        LevelEditorController::load_base_assets(resource_manager)?;
        let ball = Ball::with_default_size(level.ball_position());
        let ball = BallController::new(ball, resource_manager)?;
        let mut camera = Camera::default();
        camera.focus_on(level.ball_position(), level.map().dimensions_f32());

        let walls: Result<Vec<_>> = level
            .walls()
            .iter()
            .map(|info| wall::Wall::new(info.clone(), resource_manager))
            .collect();

        let pumps: Result<Vec<_>> = level
            .pumps()
            .iter()
            .map(|info| entities::Pump::new(info.clone(), resource_manager))
            .collect();

        let mines: Result<Vec<_>> = level
            .mines()
            .iter()
            .map(|info| entities::Mine::new(info.clone(), resource_manager))
            .collect();

        let gems: Result<Vec<_>> = level
            .gems()
            .iter()
            .map(|info| entities::Gem::new(info.clone(), resource_manager))
            .collect();

        let finish = if let Some(finish_info) = level.finish_flag() {
            Some(entities::Finish::new(
                finish_info.clone(),
                resource_manager,
            )?)
        } else {
            None
        };

        Ok(LevelEditorController {
            level,
            ball,
            camera,
            cursor: [0.0, 0.0].into(),
            logical_cursor: [0.0, 0.0].into(),
            state: Default::default(),
            res: resource_manager,
            walls: walls?,
            mines: mines?,
            pumps: pumps?,
            gems: gems?,
            finish,
            placeholder: ObjectPlaceholder::Wall {
                dim: [48.0, 48.0].into(),
                texture_id: 0,
            },
        })
    }

    fn load_base_assets(resource_manager: R) -> Result<()> {
        let mut sprite = resource_manager.sprite();
        sprite.new_sprite_from_path(SpriteAssetId::Pump, "assets/pump-wheel.png")?;
        sprite.new_sprite_from_path(SpriteAssetId::Gem, "assets/gem.png")?;
        sprite.new_sprite_from_path(SpriteAssetId::Mine, "assets/mine.png")?;
        sprite.new_sprite_from_path(SpriteAssetId::Flag, "assets/flag.png")?;
        sprite.new_sprite_from_path(SpriteAssetId::Check, "assets/check.png")?;
        for i in 0.. {
            let path = format!("assets/{}.png", i);
            match sprite.new_sprite_from_path(SpriteAssetId::Other(i), path) {
                Ok(_) => {}
                Err(_) => {
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    fn place_current_object(&mut self) -> Result<()> {
        let pos = self.logical_cursor;

        match self.placeholder {
            ObjectPlaceholder::Wall { dim, texture_id } => {
                // snap position to 4 pixel grid
                let mut pos = pos;
                pos /= 4.;
                pos[0] = pos[0].round();
                pos[1] = pos[1].round();
                pos *= 4.;
                let pos = Vector2::new(pos[0] as i32, pos[1] as i32);
                let dim = Vector2::new(dim[0] as i32, dim[1] as i32);

                // create info and entity
                let info = WallInfo {
                    pos,
                    dim,
                    texture_id,
                };
                let wall = wall::Wall::new(info.clone(), self.res)?;

                // adjust map to fit
                self.level.map_mut().expand_to_fit(info.pos + info.dim);

                // add to editor
                self.walls.push(wall);
                // and add to level
                self.level.walls_mut().push(info);

                // we're done
                Ok(())
            }
            ObjectPlaceholder::Mine => {
                let pos = Vector2::new(pos[0] as i32, pos[1] as i32);
                let info = MineInfo { pos };
                // add to map
                let mine = entities::Mine::new(info.clone(), self.res)?;
                self.mines.push(mine);
                // and add to level
                self.level.mines_mut().push(info);
                // we're done
                Ok(())
            }
            ObjectPlaceholder::Pump => {
                let pos = Vector2::new(pos[0] as i32, pos[1] as i32);
                let info = PumpInfo { pos };
                // add to map
                let pump = entities::Pump::new(info.clone(), self.res)?;
                self.pumps.push(pump);
                // and add to level
                self.level.pumps_mut().push(info);
                // we're done
                Ok(())
            }
            ObjectPlaceholder::Gem => {
                let pos = Vector2::new(pos[0] as i32, pos[1] as i32);
                let info = GemInfo { pos };
                // add to map
                let gem = entities::Gem::new(info.clone(), self.res)?;
                self.gems.push(gem);
                // add to level
                self.level.gems_mut().push(info);
                // update finish flag with gem count
                if let Some(finish) = self.level.finish_flag_mut() {
                    finish.gems_required += 1;
                }
                // we're done
                Ok(())
            }
            ObjectPlaceholder::Ball => {
                // just redefine the position
                self.ball.set_position(pos);
                self.level.set_ball_position(pos);
                Ok(())
            }
            ObjectPlaceholder::Finish => {
                let pos = Vector2::new(pos[0] as i32, pos[1] as i32);
                if let Some(f) = self.finish.as_mut() {
                    // redefine position in level
                    let info = self.level.finish_flag_mut().unwrap();
                    info.pos = pos;
                    // rebuild flag entity
                    *f = entities::Finish::new(info.clone(), self.res)?;
                    return Ok(());
                }
                let gems_required = self.level.gems().len() as u32;
                let info = FinishInfo { pos, gems_required };
                let finish = entities::Finish::new(info.clone(), self.res)?;
                // add as entity
                self.finish = Some(finish);
                // add to level
                self.level.set_finish_flag(info);
                Ok(())
            }
        }
    }

    fn remove_at(&mut self, logical_pos: Vector2<f32>) -> bool {
        // try to remove a wall
        if let Some(i) = self.walls
            .iter()
            .position(|w| w.test_point_collision_simple(logical_pos))
        {
            // remove entity
            self.walls.remove(i);
            // and remove from level
            self.level.walls_mut().remove(i);
            return true;
        }

        // try to remove a mine
        if let Some(i) = self.mines
            .iter()
            .position(|o| o.test_point_collision_simple(logical_pos))
        {
            // remove entity
            self.mines.remove(i);
            // and remove from level
            self.level.mines_mut().remove(i);
            return true;
        }

        // try to remove a pump
        if let Some(i) = self.pumps
            .iter_mut()
            .position(|o| o.test_point_collision_simple(logical_pos))
        {
            // remove entity
            self.pumps.remove(i);
            // and remove from level
            self.level.pumps_mut().remove(i);
            return true;
        }

        // try to remove a gem
        if let Some(i) = self.gems
            .iter_mut()
            .position(|o| o.test_point_collision_simple(logical_pos))
        {
            // remove entity
            self.gems.remove(i);
            // remove from level
            self.level.gems_mut().remove(i);
            // update gem count in finish flag
            if let Some(finish) = self.level.finish_flag_mut() {
                finish.gems_required -= 1;
            }
            return true;
        }

        // try to remove the finish flag
        if self.finish
            .as_ref()
            .map(|o| o.test_point_collision_simple(logical_pos))
            .unwrap_or(false)
        {
            // remove entity
            self.finish = None;
            // and remove from level
            self.level.clear_finish_flag();
            return true;
        }

        false
    }

    fn save(&mut self) {
        let mut filepath: PathBuf = Default::default();
        let mut s = Default::default();
        self.level.set_version(VERSION);
        for i in 0_u16.. {
            s = format!("levels/{}.json", i);
            let path = Path::new(&s).to_path_buf();
            if !path.exists() {
                filepath = path;
                break;
            }
        }
        self.level.set_name(&*s);
        self.level.save(filepath).unwrap();
        println!("Saved level to {}", s);
    }
}

impl<R> Controller for LevelEditorController<R>
where
    R: ResourceManage + Copy,
{
    type Res = R;
    const NEEDS_HI_RES: bool = true;

    fn event<E: GenericEvent>(&mut self, e: &E) -> Option<ControllerAction> {
        use piston::input::{Button, ButtonState, Key, MouseButton};
        self.ball.event(e);
        if let Some(b) = e.button_args() {
            // Set cell value.
            match (b.button, b.state, b.scancode) {
                (Button::Mouse(MouseButton::Middle), state, _) => {
                    // handle dragging the map for panning
                    match (self.state, state) {
                        (EditState::Idle, ButtonState::Press) => {
                            self.state = EditState::Panning;
                        }
                        (EditState::Panning, ButtonState::Release) => {
                            self.state = EditState::Idle;
                            self.camera.round_position();
                        }
                        _ => {}
                    }
                }
                (Button::Mouse(MouseButton::Left), ButtonState::Release, _) => {
                    // place new object
                    self.place_current_object().unwrap();
                }
                (Button::Mouse(MouseButton::Right), ButtonState::Press, _) => {
                    // attempt to remove an object at the cursor's position
                    let pos = self.logical_cursor.clone();
                    self.remove_at(pos);
                }
                (Button::Keyboard(Key::Escape), ButtonState::Press, _) => {
                    return Some(ControllerAction::LoadTitleScreen);
                }
                (Button::Keyboard(Key::Comma), ButtonState::Press, _) => {
                    if let ObjectPlaceholder::Wall {
                        ref mut dim,
                        ref mut texture_id,
                    } = self.placeholder
                    {
                        // roll wall texture
                        if *texture_id == 0 {
                            *texture_id = self.res.sprite().max_texture_id().saturating_sub(1);
                        } else {
                            *texture_id -= 1;
                        }
                        // update wall dimensions from texture id
                        let tdims = self.res
                            .sprite()
                            .get_sprite_dimensions(SpriteAssetId::Other(*texture_id));
                        if let Some(tex_dim) = tdims {
                            *dim = tex_dim;
                        } else {
                            *texture_id = 0;
                            *dim = [48., 48.].into();
                        }
                    }
                }
                (Button::Keyboard(Key::Period), ButtonState::Press, _) => {
                    if let ObjectPlaceholder::Wall {
                        ref mut dim,
                        ref mut texture_id,
                    } = self.placeholder
                    {
                        *texture_id += 1;
                        // roll wall texture
                        if *texture_id == self.res.sprite().max_texture_id() {
                            *texture_id = 0;
                        }
                        // update wall dimensions from texture id
                        let tdims = self.res
                            .sprite()
                            .get_sprite_dimensions(SpriteAssetId::Other(*texture_id));
                        if let Some(tex_dim) = tdims {
                            *dim = tex_dim;
                        } else {
                            *texture_id = 0;
                            *dim = [48., 48.].into();
                        }
                    }
                }
                (_k, _state, _scancode) => {}
            }
        }
        if let Some(m) = e.mouse_cursor_args() {
            let newcursor: Vector2<f32> = [m[0] as f32, m[1] as f32].into();
            let pixel_scale = 2.;
            if self.state == EditState::Panning {
                let mut delta = self.cursor - newcursor;
                delta /= pixel_scale;
                self.camera.pan(delta);
                self.camera.clamp_to_bounds(self.level.map().dimensions_f32());
            }

            self.cursor = newcursor;
            self.logical_cursor = self.camera.position() + self.cursor / pixel_scale;
        }

        if let Some(_m) = e.cursor_args() {
            //println!("{:?}", m);
        }

        if let Some(m) = e.mouse_scroll_args() {
            //println!("{:?}", m);
            let (_x_scroll, y_scroll) = (m[0], m[1]);
            if y_scroll > 0. {
                // next item in placeholder
                self.placeholder = self.placeholder.next();
            } else if y_scroll < 0. {
                self.placeholder = self.placeholder.previous();
            }
            if y_scroll != 0. {
                if let ObjectPlaceholder::Wall {
                    ref mut dim,
                    ref mut texture_id,
                } = self.placeholder
                {
                    // update wall dimensions from texture id
                    let tdims = self.res
                        .sprite()
                        .get_sprite_dimensions(SpriteAssetId::Other(*texture_id));
                    if let Some(tex_dim) = tdims {
                        *dim = tex_dim;
                    } else {
                        *texture_id = 0;
                        *dim = [48., 48.].into();
                    }
                }
            }
        }

        if let Some(k) = e.text_args() {
            if k == "S" || k == "s" {
                // save here
                self.save();
            }
        }

        None
    }

    fn update(&mut self, _u: UpdateArgs) -> Option<ControllerAction> {
        None
    }

    fn render<C, G>(&self, c: Context, _cache: &mut C, g: &mut G)
    where
        C: CharacterCache<Texture = GameTexture<R>>,
        G: Graphics<Texture = GameTexture<R>>,
    {
        clear([0.6, 0.6, 0.6, 1.0], g);
        // use camera focus to define a position
        let camera_pos = self.camera.position();
        let c = c.trans((-camera_pos[0]).into(), (-camera_pos[1]).into());

        for wall in &self.walls {
            wall.draw(c, g);
        }
        for mine in &self.mines {
            mine.draw(c, g);
        }
        for gem in &self.gems {
            gem.draw(c, g);
        }
        self.ball.draw(c, g);
        for pump in &self.pumps {
            pump.draw(c, g);
        }

        if let Some(finish) = self.finish.as_ref() {
            finish.draw(c, g);
        }
    }

    fn render_hires<C, G>(&self, c: Context, _cache: &mut C, g: &mut G)
    where
        C: CharacterCache<Texture = GameTexture<R>>,
        G: Graphics<Texture = GameTexture<R>>,
    {
        let mut point = self.logical_cursor - self.camera.position();
        let viewport = c.viewport.unwrap();
        let pixel_scale_w = viewport.window_size[0] as f32 / ::WIDTH as f32;
        let pixel_scale_h = viewport.window_size[1] as f32 / ::HEIGHT as f32;
        let pixel_scale = Vector2::from([pixel_scale_w, pixel_scale_h]);
        match self.placeholder {
            ObjectPlaceholder::Wall { dim, .. } => {
                let color = [0.25, 0.265, 0.3, 0.75];
                // snap point to 4 pixel grid
                point /= 4.;
                point[0] = point[0].round();
                point[1] = point[1].round();
                point *= 4.;

                let point = [point[0] * pixel_scale_w, point[1] * pixel_scale_h];
                let (x, y) = (point[0] as f64, point[1] as f64);
                let r = [
                    x,
                    y,
                    (dim[0] * pixel_scale_w) as f64,
                    (dim[1] * pixel_scale_h) as f64,
                ];
                rectangle(color, r, c.transform, g);
            }
            ObjectPlaceholder::Mine => {
                let color = [0.5, 0.3, 0.3, 0.75];
                let (x, y) = ((point[0] * pixel_scale_w) as f64, (point[1] * pixel_scale_h) as f64);
                let size = (entities::MINE_SIZE + 4.) * pixel_scale;
                let size = Vector2::from([size[0] as f64, size[1] as f64]);
                let hsize = size / 2.;
                let r = [(x - hsize[0]), (y - hsize[1]), size[0], size[1]];
                ellipse(color, r, c.transform, g);
            }
            ObjectPlaceholder::Pump => {
                let color = [1., 1., 0.25, 0.75];
                let r = point_to_rect(point, [entities::PUMP_SIZE, entities::PUMP_SIZE], pixel_scale);
                ellipse(color, r, c.transform, g);
            }
            ObjectPlaceholder::Gem => {
                let color = [0.8, 0.2, 0.7, 0.75];
                let r = point_to_rect(point, [entities::GEM_SIZE_W, entities::GEM_SIZE_H], pixel_scale);
                ellipse(color, r, c.transform, g);
            }
            ObjectPlaceholder::Ball => {
                let color = [0.5, 0.86, 1.0, 0.75];
                let r = point_to_rect(point, [BALL_DEFAULT_SIZE, BALL_DEFAULT_SIZE], pixel_scale);
                ellipse(color, r, c.transform, g);
            }
            ObjectPlaceholder::Finish => {
                let color = [1.0, 1.0, 1.0, 1.0];
                let r = point_to_rect(point, [8., 24.], pixel_scale);
                ellipse(color, r, c.transform, g);
            }
        }
    }

    fn exit(&mut self) {}
}

fn point_to_rect(point: Vector2<f32>, item_dims: [f32; 2], pixel_scale: Vector2<f32>) -> [f64; 4] {
    let (x, y) = ((point[0] * pixel_scale[0]) as f64, (point[1] * pixel_scale[1]) as f64);
    let size_w = item_dims[0] * pixel_scale[0];
    let size_h = item_dims[1] * pixel_scale[1];
    let hsize_w = (size_w * 0.5) as f64;
    let hsize_h = (size_h * 0.5) as f64;
    [(x - hsize_w as f64), (y - hsize_h as f64), size_w as f64, size_h as f64]
}
