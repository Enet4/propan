use graphics::character::CharacterCache;
use graphics::{clear, Context, Graphics, Transformed};
use piston::input::{GenericEvent, UpdateArgs};

pub mod ball;
pub mod entities;
pub mod items;
pub mod scene;
pub mod wall;

use self::ball::*;
use self::entities::*;
use self::scene::Scene;
use self::wall::Wall;
use camera::*;
use controller::{Controller, ControllerAction};
use level::GameLevel;
use resource::{GameTexture, ResourceManage, Result, SpriteAssetId, SpriteManage};

pub struct GameController<R>
where
    R: ResourceManage,
{
    level: GameLevel,
    ball: BallController<R>,
    camera: Camera,
    res: R,
    walls: Scene<Wall<R>>,
    pumps: Vec<Pump<R>>,
    mines: Scene<Mine<R>>,
    gems: Scene<Gem<R>>,
    finish: Option<Finish<R>>,
}

/// Game level controller.
impl<R> GameController<R>
where
    R: ResourceManage + Copy,
{
    pub fn new(level: GameLevel, resource_manager: R) -> Result<Self> {
        GameController::load_base_assets(resource_manager)?;

        let ball = Ball::with_default_size(level.ball_position());
        let ball = BallController::new(ball, resource_manager)?;
        let mut camera = Camera::default();
        camera.focus_on(level.ball_position(), level.map().dimensions_f32());

        let walls: Result<Vec<_>> = level
            .walls()
            .iter()
            .map(|info| Wall::new(info.clone(), resource_manager))
            .collect();

        let pumps: Result<Vec<_>> = level
            .pumps()
            .iter()
            .map(|info| Pump::new(info.clone(), resource_manager))
            .collect();

        let mines: Result<Vec<_>> = level
            .mines()
            .iter()
            .map(|info| Mine::new(info.clone(), resource_manager))
            .collect();

        let gems: Result<Vec<_>> = level
            .gems()
            .iter()
            .map(|info| Gem::new(info.clone(), resource_manager))
            .collect();

        let finish = if let Some(finish_info) = level.finish_flag() {
            Some(Finish::new(finish_info.clone(), resource_manager)?)
        } else {
            None
        };

        Ok(GameController {
            level,
            ball,
            camera,
            res: resource_manager,
            walls: Scene::from_objects(walls?),
            mines: Scene::from_objects(mines?),
            pumps: pumps?,
            gems: Scene::from_objects(gems?),
            finish,
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
}

impl<R> Controller for GameController<R>
where
    R: ResourceManage,
{
    type Res = R;

    fn event<E: GenericEvent>(&mut self, e: &E) -> Option<ControllerAction> {
        use piston::input::Button::{Controller, Keyboard};
        use piston::input::{ButtonState, ControllerButton, Key};
        self.ball.event(e);
        if let Some(b) = e.button_args() {
            // Set cell value.
            match (b.button, b.state) {
                (Keyboard(Key::Escape), ButtonState::Press) => {
                    return Some(ControllerAction::LoadTitleScreen);
                }
                (Keyboard(Key::Return), ButtonState::Press)
                | (Keyboard(Key::Space), ButtonState::Press)
                | (Controller(ControllerButton { id: 0, button: 0 }), ButtonState::Press)
                | (Controller(ControllerButton { id: 0, button: 1 }), ButtonState::Press) => {
                    if self.ball.is_dead() || self
                        .finish
                        .as_ref()
                        .map(|f| f.is_picked_up())
                        .unwrap_or(false)
                    {
                        return Some(ControllerAction::LoadTitleScreen);
                    }
                }
                _ => {
                    // do nothing
                }
            }
        }

        if let Some(k) = e.text_args() {
            if k == "E" {
                return Some(ControllerAction::OpenEditor(None));
            }
        }

        None
    }

    fn update(&mut self, u: UpdateArgs) -> Option<ControllerAction> {
        let ticks = 60. * u.dt as f32;

        // update entities
        for pump in &mut self.pumps {
            pump.update(ticks);
        }

        // handle map boundary collision
        self.ball
            .handle_collision_with(self.level.map().left_border());
        self.ball
            .handle_collision_with(self.level.map().right_border());
        self.ball
            .handle_collision_with(self.level.map().up_border());
        self.ball
            .handle_collision_with(self.level.map().down_border());

        // handle collisions with scene
        for wall in self.walls.at_mut(self.ball.position()) {
            self.ball.handle_collision_with(wall);
        }
        // handle contact with pumps
        for pump in &mut self.pumps {
            self.ball.handle_simple_collision_with(pump);
        }
        // handle contact with mines
        for mine in self.mines.at(self.ball.position()) {
            self.ball.handle_simple_collision_with(mine);
        }
        // handle contact with gems
        for gem in self.gems.at_mut(self.ball.position()) {
            self.ball.handle_simple_collision_with(gem);
        }
        // handle contact with finish flag
        if let Some(finish) = self.finish.as_mut() {
            self.ball.handle_simple_collision_with(finish);
        }

        // update the ball
        self.ball.update(ticks);

        // update the camera's position
        let map_dim = self.level.map().dimensions_f32();
        self.camera.soft_focus_on(self.ball.position(), map_dim);

        None
    }

    fn render<C, G>(&self, c: Context, _cache: &mut C, g: &mut G)
    where
        C: CharacterCache<Texture = GameTexture<R>>,
        G: Graphics<Texture = GameTexture<R>>,
    {
        clear([0.4, 0.6, 0.7, 1.0], g);
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
        if let Some(finish) = self.finish.as_ref() {
            finish.draw(c, g);
        }
        self.ball.draw(c, g);
        for pump in &self.pumps {
            pump.draw(c, g);
        }
    }
}
