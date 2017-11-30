use std::marker::PhantomData;
use na::{norm_squared, Vector2};
use physics::{AnimatedObject, SimpleCollidable};
use graphics::{ellipse, Context, DrawState, Graphics, Image, Transformed, ImageSize};
use resource::{GameTexture, ResourceManage, Result, SpriteManage};
use resource::sprite::AssetId;
use level::info::{PumpInfo, MineInfo, GemInfo, FinishInfo};

pub const PUMP_SIZE: f32 = 34.0;

pub struct Pump<R>
where
    R: ResourceManage,
{
    pos: Vector2<f32>,
    gfx_img: GameTexture<R>,
    time_to_pump: f32,
    rot: f32,
    phantom: PhantomData<R>,
}

impl<R> Pump<R>
where
    R: ResourceManage
{
    pub fn new(info: PumpInfo, resource_manager: R) -> Result<Self> {
        let gfx_img = resource_manager.sprite().get_sprite(AssetId::Pump)?;
        Ok(Pump {
            pos: info.pos,
            gfx_img,
            time_to_pump: 0.,
            rot: 0.,
            phantom: PhantomData,
        })
    }

    pub fn update(&mut self, factor: f32) {
        self.rot += 0.025 * factor;
        const TWO_PI: f32 = 2. * 3.14159265358979;
        if self.rot > TWO_PI {
            self.rot -= TWO_PI;
        }
        if self.time_to_pump > 0. {
            self.time_to_pump -= factor;
        }
    }

    pub fn draw<G>(&self, ctx: Context, g: &mut G)
    where
        G: Graphics<Texture=GameTexture<R>>
    {
        let size = PUMP_SIZE as f64;
        let (x, y) = (self.pos[0] as f64, self.pos[1] as f64);
        let (w, h) = self.gfx_img.get_size();
        let (w, h) = (w as f32, h as f32);
        let (hw, hh) = (w / 2., h / 2.);
        let w_scale = PUMP_SIZE / w;
        let h_scale = PUMP_SIZE / h;

        ellipse(
            [1.0, 1.0, 0.8, 0.25],
            [
                x - size / 2. + 2., y - size / 2. + 2.,
                size - 4., size - 4.
            ],
            ctx.transform,
            g,
        );

        let ctx = ctx
            .trans(x as f64, y as f64)
            .scale(w_scale.into(), h_scale.into())
            .rot_rad(self.rot.into())
            .trans(-hw as f64, -hh as f64);
        Image::new().draw(&self.gfx_img, &DrawState::default(), ctx.transform, g);
    }
}

impl<'a, R> SimpleCollidable for &'a mut Pump<R>
where
    R: ResourceManage,
{
    fn test_circle_collision_simple(&self, position: Vector2<f32>, radius: f32) -> bool {
        let d = PUMP_SIZE / 2. + radius - 2.;
        norm_squared(&(self.pos - position)) <= d * d
    }

    fn on_collision_simple<A>(&mut self, ball: &mut A)
    where
        A: AnimatedObject,
    {
        // TODO when doing sounds, reproduce something here

        if self.time_to_pump <= 0. {
            // pump!
            ball.heal(1.0);
            self.time_to_pump += 22.;
        }
    }
}



pub const MINE_SIZE: f32 = 6.0;

pub struct Mine<R>
where
    R: ResourceManage,
{
    pos: Vector2<f32>,
    gfx_img: GameTexture<R>,
    res: R,
}

impl<R> Mine<R>
where
    R: ResourceManage,
{
    pub fn new(info: MineInfo, resource_manager: R) -> Result<Self> {
        let gfx_img = resource_manager.sprite().get_sprite(AssetId::Mine)?;
        Ok(Mine {
            pos: info.pos,
            gfx_img,
            res: resource_manager,
        })
    }

    pub fn draw<G: Graphics>(&self, ctx: Context, gfx: &mut G)
    where
        G: Graphics<Texture=GameTexture<R>>
    {
        let (x, y) = (self.pos[0] as f64, self.pos[1] as f64);
        let hsize = (MINE_SIZE / 2.) as f64;
        Image::new().draw(
            &self.gfx_img,
            &DrawState::default(),
            ctx.transform.trans(x - hsize - 2., y - hsize - 2.),
            gfx
        );
    }
}

impl<'a, R> SimpleCollidable for &'a Mine<R>
where
    R: ResourceManage,
{
    fn test_circle_collision_simple(&self, position: Vector2<f32>, radius: f32) -> bool {
        let d = MINE_SIZE / 2. + radius + 1.;
        norm_squared(&(self.pos - position)) <= d * d
    }

    fn on_collision_simple<A>(&mut self, ball: &mut A)
    where
        A: AnimatedObject,
    {
        // TODO when doing sounds, reproduce something here
        // TODO particles
        ball.damage(2.5);
    }
}

pub const GEM_SIZE: f32 = 24.;

pub struct Gem<R>
where
    R: ResourceManage,
{
    pos: Vector2<f32>,
    gfx_img: GameTexture<R>,
    picked_up: bool,
}


impl<R> Gem<R>
where
    R: ResourceManage
{
    pub fn new(info: GemInfo, resource_manager: R) -> Result<Self> {
        let gfx_img = resource_manager.sprite().get_sprite(AssetId::Gem)?;
        Ok(Gem {
            pos: info.pos,
            gfx_img,
            picked_up: false,
        })
    }

    pub fn draw<G>(&self, ctx: Context, g: &mut G)
    where
        G: Graphics<Texture=GameTexture<R>>
    {
        if self.picked_up {
            return;
        }

        let size = GEM_SIZE as f64;
        let (x, y) = (self.pos[0] as f64, self.pos[1] as f64);
        let (w, h) = self.gfx_img.get_size();
        let (w, h) = (w as f32, h as f32);
        let (hw, hh) = (w / 2., h / 2.);
        let w_scale = GEM_SIZE / w;
        let h_scale = GEM_SIZE / h;

        let ctx = ctx
            .trans(x as f64, y as f64)
            .scale(w_scale.into(), h_scale.into())
            .trans(-hw as f64, -hh as f64);
        Image::new().draw(&self.gfx_img, &DrawState::default(), ctx.transform, g);
    }
}

impl<R> SimpleCollidable for Gem<R>
where
    R: ResourceManage
{
    fn test_circle_collision_simple(&self, position: Vector2<f32>, radius: f32) -> bool {
        if self.picked_up { return false; }

        let d = GEM_SIZE / 2. + radius + 1.;
        norm_squared(&(self.pos - position)) <= d * d
    }

    fn on_collision_simple<A>(&mut self, ball: &mut A)
    where
        A: AnimatedObject
    {
        if self.picked_up { return; }

        let item = ();
        ball.pick_up(item);
        self.picked_up = true;
    }
}

pub const FINISH_SIZE: f32 = 24.;

pub struct Finish<R>
where
    R: ResourceManage,
{
    pos: Vector2<f32>,
    gfx_img: GameTexture<R>,
    gfx_img_check: GameTexture<R>,
    picked_up: bool,
    gems_required: u32,
}


impl<R> Finish<R>
where
    R: ResourceManage
{
    pub fn new(info: FinishInfo, resource_manager: R) -> Result<Self> {
        let gfx_img = resource_manager.sprite().get_sprite(AssetId::Flag)?;
        let gfx_img_check = resource_manager.sprite().get_sprite(AssetId::Check)?;
        Ok(Finish {
            pos: info.pos,
            gfx_img,
            gfx_img_check,
            picked_up: false,
            gems_required: info.gems_required,
        })
    }

    pub fn draw<G>(&self, ctx: Context, g: &mut G)
    where
        G: Graphics<Texture=GameTexture<R>>
    {
        let (x, y) = (self.pos[0] as f64, self.pos[1] as f64);
        let (w, h) = self.gfx_img.get_size();
        let (w, h) = (w as f32, h as f32);
        let (hw, hh) = (w / 2., h / 2.);

        let ctx = ctx
            .trans(x - hw as f64, y - hh as f64);
        let img = if self.picked_up {
            &self.gfx_img_check
        } else {
            &self.gfx_img
        };
        Image::new().draw(img, &DrawState::default(), ctx.transform, g);
    }
}

impl<R> SimpleCollidable for Finish<R>
where
    R: ResourceManage
{
    fn test_circle_collision_simple(&self, position: Vector2<f32>, radius: f32) -> bool {
        if self.picked_up { return false; }
        let d = FINISH_SIZE / 2. + radius + 1.;
        norm_squared(&(self.pos - position)) <= d * d
    }

    fn on_collision_simple<A>(&mut self, ball: &mut A)
    where
        A: AnimatedObject
    {
        if !self.picked_up && ball.items() == self.gems_required {
            self.picked_up = true;
        }
    }
}
