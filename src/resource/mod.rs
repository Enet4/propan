use std::cell::{RefCell, RefMut};
use gfx_device_gl::{Factory, Resources};

pub mod sprite;
pub mod audio;

pub use self::sprite::{SpriteManage, SpriteManager, AssetId as SpriteAssetId};
pub use self::audio::{AudioManage, AudioManager};

pub type Result<T> = ::std::result::Result<T, ResourceError>;

pub type GameTexture<T> = <<T as ResourceManage>::Sprite as SpriteManage>::Texture;

#[derive(Debug, Fail)]
pub enum ResourceError {
    #[fail(display = "No such sprite for ID #{:?}", id)]
    NoSprite {
        id: sprite::AssetId,
    },
    #[fail(display = "No such audio sample for ID #{:?}", id)]
    NoAudioSample {
        id: (),
    },
    #[fail(display = "gfx error: {}", msg)]
    GfxResource {
        msg: String,
    },
}

pub type ResourceManager = ResourceManagerImpl<SpriteManager<Factory, Resources>, AudioManager>;

pub trait ResourceManage {
    type Sprite: SpriteManage;
    type Audio: AudioManage;

    fn sprite(&self) -> Self::Sprite;
    fn audio(&self) -> Self::Audio;
}

impl<'a, T: ResourceManage> ResourceManage for &'a T {
    type Sprite = T::Sprite;
    type Audio = T::Audio;

    fn sprite(&self) -> Self::Sprite {
        (**self).sprite()
    }
    fn audio(&self) -> Self::Audio {
        (**self).audio()
    }
}

pub struct ResourceManagerImpl<S, A> {
    sprite: RefCell<S>,
    audio: RefCell<A>,
}

impl<S, A> ResourceManagerImpl<S, A>
{
    pub fn new(sprite_manager: S, audio_manager: A) -> Self {
        ResourceManagerImpl {
            sprite: sprite_manager.into(),
            audio: audio_manager.into(),
        }
    }
}

impl<'a, S, A> ResourceManage for &'a ResourceManagerImpl<S, A>
where
    for <'g> S: SpriteManage,
    A: AudioManage,
{
    type Sprite = RefMut<'a, S>;
    type Audio = RefMut<'a, A>;

    fn sprite(&self) -> Self::Sprite {
        self.sprite.borrow_mut()
    }
    fn audio(&self) -> Self::Audio {
        self.audio.borrow_mut()
    }
}
