use graphics::{Context, Graphics};
use graphics::character::CharacterCache;
use piston::input::{GenericEvent, UpdateArgs};
use resource::{ResourceManage, SpriteManage};

pub trait Controller {
    type Res: ResourceManage;
    const NEEDS_HI_RES: bool = false;

    fn event<E>(&mut self, e: &E) -> Option<ControllerAction>
    where
        E: GenericEvent;

    fn update(&mut self, args: UpdateArgs) -> Option<ControllerAction>;

    fn render<C, G>(&self, c: Context, cache: &mut C, g: &mut G)
    where
        C: CharacterCache<Texture=<<Self::Res as ResourceManage>::Sprite as SpriteManage>::Texture>,
        G: Graphics<Texture=<<Self::Res as ResourceManage>::Sprite as SpriteManage>::Texture>;

    fn render_hires<C, G>(&self, c: Context, cache: &mut C, g: &mut G)
    where
        C: CharacterCache<Texture=<<Self::Res as ResourceManage>::Sprite as SpriteManage>::Texture>,
        G: Graphics<Texture=<<Self::Res as ResourceManage>::Sprite as SpriteManage>::Texture> {
        // render nothing by default
    }

    fn exit(&mut self) {
        // do nothing by default
    }
}

pub type LevelId = u16;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ControllerAction {
    Exit,
    OpenEditor(Option<String>),
    LoadGame(LevelId),
    LoadTitleScreen,
}
