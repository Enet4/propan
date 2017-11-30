use std::collections::HashMap;
use std::cell::RefMut;
use std::path::Path;
use graphics::ImageSize;
use gfx::{Factory, Resources};
use gfx_graphics::{Flip, Filter, Texture as GfxTexture, TextureSettings};
use na::Vector2;
use super::{ResourceError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AssetId {
    Background,
    Logo,
    Pump,
    Mine,
    Gem,
    Flag,
    Check,
    Other(u32),
}

pub trait SpriteManage {
    type Texture: ImageSize;

    fn new_sprite_from_path<P: AsRef<Path>>(&mut self, id: AssetId, path: P) -> Result<()>;

    fn get_sprite(&self, id: AssetId) -> Result<Self::Texture>;

    fn get_sprite_dimensions(&self, id: AssetId) -> Option<Vector2<f32>> {
        self.get_sprite(id).ok()
            .map(|t| t.get_size())
            .map(|(w, h)| [w as f32, h as f32].into())
    }

    fn max_texture_id(&self) -> u32;

    fn free_sprite(&mut self, id: AssetId) -> Result<()>;

    fn free_all(&mut self) -> Result<()>;
}
impl<'a, 'g, T: SpriteManage> SpriteManage for &'a mut T {
    type Texture = T::Texture;

    fn new_sprite_from_path<P: AsRef<Path>>(&mut self, id: AssetId, path: P) -> Result<()> {
        (**self).new_sprite_from_path(id, path)
    }

    fn get_sprite(&self, id: AssetId) -> Result<Self::Texture> {
        (**self).get_sprite(id)
    }

    fn max_texture_id(&self) -> u32 {
        (**self).max_texture_id()
    }

    fn free_sprite(&mut self, id: AssetId) -> Result<()> {
        (**self).free_sprite(id)
    }

    fn free_all(&mut self) -> Result<()> {
        (**self).free_all()
    }
}
impl<'a, 'g, T: SpriteManage> SpriteManage for RefMut<'a, T> {
    type Texture = T::Texture;

    fn new_sprite_from_path<P: AsRef<Path>>(&mut self, id: AssetId, path: P) -> Result<()> {
        (**self).new_sprite_from_path(id, path)
    }

    fn get_sprite(&self, id: AssetId) -> Result<Self::Texture> {
        (**self).get_sprite(id)
    }

    fn max_texture_id(&self) -> u32 {
        (**self).max_texture_id()
    }

    fn free_sprite(&mut self, id: AssetId) -> Result<()> {
        (**self).free_sprite(id)
    }

    fn free_all(&mut self) -> Result<()> {
        (**self).free_all()
    }
}

#[derive(Debug)]
pub struct SpriteManager<F, R>
where
    R: Resources,
{
    factory: F,
    loaded_sprites: HashMap<AssetId, GfxTexture<R>>,
    max_id: u32,
}

impl<F, R> SpriteManager<F, R>
where
    F: Factory<R>,
    R: Resources,
{
    pub fn new(params: F) -> Result<Self> {
        Ok(SpriteManager {
            factory: params,
            loaded_sprites: HashMap::new(),
            max_id: 0,
        })
    }
}

impl<F, R> SpriteManage for SpriteManager<F, R>
where
    F: Factory<R>,
    R: Resources,
{
    type Texture = GfxTexture<R>;

    fn new_sprite_from_path<P: AsRef<Path>>(&mut self, id: AssetId, path: P) -> Result<()> {
        let mut tex_settings = TextureSettings::new();
        tex_settings.set_filter(Filter::Nearest);
        let tex =
            GfxTexture::from_path(&mut self.factory, path, Flip::None, &tex_settings)
                .map_err(|e| ResourceError::GfxResource { msg: e })?;
        self.loaded_sprites.insert(id, tex);
        if let AssetId::Other(i) = id {
            self.max_id = u32::max(self.max_id, i + 1);
        }
        Ok(())
    }

    fn get_sprite(&self, id: AssetId) -> Result<Self::Texture> {
        self.loaded_sprites
            .get(&id)
            .map(|x| x.clone())
            .ok_or_else(|| ResourceError::NoSprite { id })
    }

    fn max_texture_id(&self) -> u32 {
        self.max_id
    }

    fn free_sprite(&mut self, id: AssetId) -> Result<()> {
        if self.loaded_sprites.remove(&id).is_some() {
            if let AssetId::Other(i) = id {
                if i < self.max_id - 1 { return Ok(()); }
                let mut i = i;
                while i > 0 && !self.loaded_sprites.contains_key(&AssetId::Other(i - 1)) {
                    i -= 1;
                }
                self.max_id = 1;
            }
            Ok(())
        } else {
            Err(ResourceError::NoSprite { id })
        }
    }

    fn free_all(&mut self) -> Result<()> {
        self.loaded_sprites.clear();
        self.max_id = 0;
        Ok(())
    }
}
