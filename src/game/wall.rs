use graphics::{Context, DrawState, Graphics, Image, Transformed, ImageSize};
use physics::{AnimatedObject, Collidable, CollisionInfo};
use na::{norm_squared, Vector2};
use resource::{GameTexture, ResourceManage, Result};
use resource::sprite::{AssetId, SpriteManage};
use level::info::WallInfo;

pub struct Wall<R>
where
    R: ResourceManage,
{
    pos: Vector2<f32>,
    dim: Vector2<f32>,
    gfx_tex: GameTexture<R>,
}

impl<R> Wall<R>
where
    R: ResourceManage,
{
    pub fn new(info: WallInfo, res: R) -> Result<Self> {
        let gfx_tex = res.sprite().get_sprite(AssetId::Other(info.texture_id))?;
        Ok(Wall {
            pos: info.pos,
            dim: info.dim,
            gfx_tex,
        })
    }

    pub fn draw<G>(&self, ctx: Context, g: &mut G)
    where
        G: Graphics<Texture=GameTexture<R>>,
    {
        let (x, y) = (self.pos[0] as f64, self.pos[1] as f64);
        let (w, h) = self.gfx_tex.get_size();
        let (w, h) = (w as f32, h as f32);
        let w_scale = self.dim[0] / w;
        let h_scale = self.dim[1] / h;
        let ctx = ctx
            .trans(x as f64, y as f64)
            .scale(w_scale.into(), h_scale.into());
        Image::new().draw(&self.gfx_tex, &DrawState::default(), ctx.transform, g);

    }
}

impl<R> Collidable for Wall<R>
where
    R: ResourceManage,
{
    fn test_circle_collision(&self, position: Vector2<f32>, radius: f32) -> CollisionInfo {
        let br = self.pos + self.dim;
        let nearest_x = f32::max(self.pos[0], f32::min(position[0], br[0]));
        let nearest_y = f32::max(self.pos[1], f32::min(position[1], br[1]));

        let nearest_point: Vector2<_> = [nearest_x, nearest_y].into();
        let delta_vector = position - nearest_point;

        let dist_sqr = norm_squared(&delta_vector);
        if dist_sqr <= radius * radius {
            // adjust delta vector to have overlap magnitude
            let dist = f32::sqrt(dist_sqr);
            let newdistance_inv = (radius - dist) / dist;
            CollisionInfo::Yes(delta_vector * newdistance_inv)
        } else {
            CollisionInfo::No
        }
    }
    
    fn test_point_collision_simple(&self, position: Vector2<f32>) -> bool {
        let tl = self.pos;
        let br = self.pos + self.dim;
        position >= tl && position <= br
    }

    #[inline]
    fn on_collision<A>(&mut self, ball: &mut A, overlap: Vector2<f32>)
    where
        A: AnimatedObject,
    {
        ball.issue_bounce(overlap)
    }
}
