use graphics::{Context, DrawState, Graphics, Image, Text, Transformed};
use graphics::character::CharacterCache;
use piston::input::{GenericEvent, UpdateArgs};
use level::load_all_levels;
use resource::{GameTexture, ResourceManage, Result, SpriteAssetId, SpriteManage};
use controller::{Controller, ControllerAction, LevelId};
use level::GameLevel;

const WINDOW_SIZE: usize = 8;

pub struct TitleController<R>
where
    R: ResourceManage,
{
    res: R,
    title_tex: GameTexture<R>,
    logo_tex: GameTexture<R>,
    logo_pos: f64,
    level_list: Vec<GameLevel>,
    selected: Option<u32>,
}

impl<R> TitleController<R>
where
    R: ResourceManage,
{
    pub fn new(res: R) -> Result<Self> {
        let mut sprite = res.sprite();
        sprite.new_sprite_from_path(SpriteAssetId::Background, "assets/title.png")?;
        sprite.new_sprite_from_path(SpriteAssetId::Logo, "assets/logo.png")?;
        let title_tex = sprite.get_sprite(SpriteAssetId::Background)?;
        let logo_tex = sprite.get_sprite(SpriteAssetId::Logo)?;

        Ok(TitleController {
            res,
            title_tex,
            logo_tex,
            logo_pos: -120.0,
            level_list: vec![],
            selected: None,
        })
    }
}
impl<R> Controller for TitleController<R>
where
    R: ResourceManage,
{
    type Res = R;
    const NEEDS_HI_RES: bool = true;

    fn event<E: GenericEvent>(&mut self, e: &E) -> Option<ControllerAction> {
        use piston::input::{Button, ButtonState, Key};
        if let Some(b) = e.button_args() {
            // Set cell value.
            if let Button::Keyboard(k) = b.button {
                match (self.selected.is_some(), k, b.state) {
                    (_, Key::Escape, ButtonState::Press) => {
                        return Some(ControllerAction::Exit);
                    }
                    (false, _, ButtonState::Press) => {
                        // load levels
                        self.level_list = load_all_levels("levels").unwrap();
                        self.selected = Some(0);
                    }
                    (true, Key::Return, ButtonState::Press) |
                    (true, Key::Space, ButtonState::Press) => {
                        return Some(ControllerAction::LoadGame(self.selected.unwrap() as LevelId));
                    }
                    (true, Key::Right, ButtonState::Press) |
                    (true, Key::NumPad6, ButtonState::Press) => {
                        self.selected = self.selected.map(|s| {
                            u32::min(s + WINDOW_SIZE as u32, self.level_list.len() as u32 - 1)
                        });
                    }
                    (true, Key::Left, ButtonState::Press) |
                    (true, Key::NumPad4, ButtonState::Press) => {
                        self.selected = self.selected.map(|s| s.saturating_sub(WINDOW_SIZE as u32));
                    }
                    (true, Key::Up, ButtonState::Press) |
                    (true, Key::NumPad8, ButtonState::Press) => {
                        self.selected = self.selected.map(|s| s.saturating_sub(1));
                    }
                    (true, Key::Down, ButtonState::Press) |
                    (true, Key::NumPad2, ButtonState::Press) => {
                        self.selected = self.selected
                            .map(|s| u32::min(s + 1, self.level_list.len() as u32 - 1));
                    }
                    _ => {}
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
        let ticks = 60. * u.dt as f64;
        self.logo_pos = f64::min(self.logo_pos + 4.0 * ticks, 100.);
        None
    }

    fn render<C, G>(&self, c: Context, _cache: &mut C, g: &mut G)
    where
        C: CharacterCache<Texture = GameTexture<R>>,
        G: Graphics<Texture = GameTexture<R>>,
    {
        Image::new().draw(&self.title_tex, &DrawState::default(), c.transform, g);
        Image::new().draw(
            &self.logo_tex,
            &DrawState::default(),
            c.transform.trans(self.logo_pos, 20.),
            g,
        );

    }

    fn render_hires<C, G>(&self, c: Context, cache: &mut C, g: &mut G)
    where
        C: CharacterCache<Texture = GameTexture<R>>,
        G: Graphics<Texture = GameTexture<R>>,
    {

        if let Some(selected) = self.selected {
            let window_size = WINDOW_SIZE;
            let window_n = selected as usize / WINDOW_SIZE;
            let cw = c.trans(64., 164.);
            for (window_i, (i, lvl)) in self.level_list
                .iter()
                .enumerate()
                .skip(window_n * window_size)
                .take(window_size)
                .enumerate()
            {
                let c = cw.trans(0., 38. * window_i as f64);
                let color = if selected == i as u32 {
                    [1.; 4]
                } else {
                    [1.0, 1.0, 0.25, 1.0]
                };
                let _ = Text::new_color(color, 24).draw(
                    lvl.name(),
                    cache,
                    &DrawState::default(),
                    c.transform,
                    g,
                );
            }

            let _ = Text::new_color([1.; 4], 11).draw(
                "Press Shift+E to enter the level editor",
                cache,
                &DrawState::default(),
                c.transform.trans(442.0, 488.),
                g,
            );
        }
    }

    fn exit(&mut self) {
        self.res
            .sprite()
            .free_sprite(SpriteAssetId::Background)
            .unwrap();
    }
}
