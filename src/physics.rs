use na::{dot, norm_squared, Vector2};

/// A trait for things that move in the level.
pub trait AnimatedObject {
    /// Request the object to bounce.
    fn issue_bounce(&mut self, overlap: Vector2<f32>);

    /// Adjust the object's position and bounce on a vertical wall.
    fn correct_and_flip_x(&mut self, overlap: f32);

    /// Adjust the object's position and bounce on a horizontal wall.
    fn correct_and_flip_y(&mut self, overlap: f32);

    /// Add some velocity to this object.
    fn add_velocity(&mut self, extra_velocity: Vector2<f32>);

    /// Translate this object's position.
    fn add_position(&mut self, translation: Vector2<f32>);

    /// Add some damage to the object.
    fn damage(&mut self, dmg: f32);

    /// Heal the object.
    fn heal(&mut self, health: f32);

    /// Make the object pick up an item.
    fn pick_up(&mut self, item: ()); // TODO define item better

    /// Obtain information about the object's items.
    fn items(&self) -> u32; // TODO define item better
}

/// Returns the new velocity for a ball which collides with an object.
pub fn rigid_bounce(mut vel: Vector2<f32>, overlap: Vector2<f32>) -> Vector2<f32> {
    // apply velocity transformation
    let v = overlap;
    match (v[0], v[1]) {
        (x, y) if x == 0. && y == 0. => {
            // do nothing
        }
        (x, _) if x == 0. => {
            vel[1] = -vel[1];
        }
        (_, y) if y == 0. => {
            vel[0] = -vel[0];
        }
        _ => {
            // non-trivial case, use vector reflection
            let n = v;
            let v = vel;
            vel = -(n * 2. * dot(&v, &n) / norm_squared(&n) - v);
        }
    }
    vel
}

/// Data type representing information about a test for object collision.
#[derive(Debug, Clone, PartialEq)]
pub enum CollisionInfo {
    /// No collision
    No,
    /// Yes, a collision occurred, with the given maximum distance overlap
    /// vector. If this vector is added to the colliding object's position
    /// (or subtracted to the passive collided object), the position will
    /// be corrected so that they no longer overlap. The vector also works
    /// as a normal of the collision plane.
    Yes(Vector2<f32>),
}

use self::CollisionInfo::{No, Yes};

impl CollisionInfo {
    pub fn is_yes(&self) -> bool {
        match *self {
            No => false,
            Yes(_) => true,
        }
    }
}

/// Trait for things that the ball may collide with, but which require no
/// advanced information about the collision.
pub trait SimpleCollidable {
    /// Test for a collision of this object with a circle. Returns whether a collision happens.
    fn test_circle_collision_simple(&self, position: Vector2<f32>, radius: f32) -> bool;

    /// Test for a collision of this object with a point. Returns whether a collision happens.
    fn test_point_collision_simple(&self, position: Vector2<f32>) -> bool {
        self.test_circle_collision_simple(position, 1e-2)
    }

    /// A function that is called when the ball collides with this object.
    fn on_collision_simple<A>(&mut self, ball: &mut A) where A: AnimatedObject;
}

impl<'a, T: SimpleCollidable> SimpleCollidable for &'a mut T {
    fn test_circle_collision_simple(&self, position: Vector2<f32>, radius: f32) -> bool {
        (**self).test_circle_collision_simple(position, radius)
    }

    fn test_point_collision_simple(&self, position: Vector2<f32>) -> bool {
        (**self).test_point_collision_simple(position)
    }

    fn on_collision_simple<A>(&mut self, ball: &mut A) where A: AnimatedObject {
        (**self).on_collision_simple(ball)
    }
}

/// Trait for things that the ball may collide with. When a collision happens,
/// a vector of overlap is provided from this object, which is useful for calculating
/// stuff.
pub trait Collidable {
    /// Test for a collision of this object with a circle. Returns whether a collision happens.
    fn test_circle_collision_simple(&self, position: Vector2<f32>, radius: f32) -> bool {
        self.test_circle_collision(position, radius).is_yes()
    }

    /// Test for a collision of this object with a point. Returns whether a collision happens.
    fn test_point_collision_simple(&self, position: Vector2<f32>) -> bool {
        self.test_circle_collision(position, 1e-2).is_yes()
    }

    /// Test for a collision of this object with a circle.
    fn test_circle_collision(&self, position: Vector2<f32>, radius: f32) -> CollisionInfo;

    /// A function that is called when the ball collides with this object.
    fn on_collision<A>(&mut self, ball: &mut A, overlap: Vector2<f32>) where A: AnimatedObject;
}

impl<'a, T: Collidable> Collidable for &'a mut T {
    fn test_circle_collision_simple(&self, position: Vector2<f32>, radius: f32) -> bool {
        (**self).test_circle_collision_simple(position, radius)
    }

    fn test_point_collision_simple(&self, position: Vector2<f32>) -> bool {
        (**self).test_point_collision_simple(position)
    }

    fn test_circle_collision(&self, position: Vector2<f32>, radius: f32) -> CollisionInfo {
        (**self).test_circle_collision(position, radius)
    }

    fn on_collision<A>(&mut self, ball: &mut A, overlap: Vector2<f32>) where A: AnimatedObject {
        (**self).on_collision(ball, overlap)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct LeftBorder(pub f32);
impl Collidable for LeftBorder {
    fn test_circle_collision(&self, position: Vector2<f32>, radius: f32) -> CollisionInfo {
        let dx = self.0 + radius - position[0];
        if dx <= 0. {
            No
        } else {
            Yes([dx, 0.].into())
        }
    }

    #[inline]
    fn on_collision<A>(&mut self, ball: &mut A, overlap: Vector2<f32>)
    where
        A: AnimatedObject,
    {
        ball.correct_and_flip_x(overlap[0]);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RightBorder(pub f32);
impl Collidable for RightBorder {
    fn test_circle_collision(&self, position: Vector2<f32>, radius: f32) -> CollisionInfo {
        let dx = self.0 - position[0] - radius;
        if dx > 0. {
            No
        } else {
            Yes([dx, 0.].into())
        }
    }

    #[inline]
    fn on_collision<A>(&mut self, ball: &mut A, overlap: Vector2<f32>)
    where
        A: AnimatedObject,
    {
        ball.correct_and_flip_x(overlap[0]);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct UpBorder(pub f32);
impl Collidable for UpBorder {
    fn test_circle_collision(&self, position: Vector2<f32>, radius: f32) -> CollisionInfo {
        let dy = self.0 + radius - position[1];
        if dy <= 0. {
            No
        } else {
            Yes([0., dy].into())
        }
    }

    #[inline]
    fn on_collision<A>(&mut self, ball: &mut A, overlap: Vector2<f32>)
    where
        A: AnimatedObject,
    {
        ball.correct_and_flip_y(overlap[1]);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DownBorder(pub f32);
impl Collidable for DownBorder {
    fn test_circle_collision(&self, position: Vector2<f32>, radius: f32) -> CollisionInfo {
        let dy = self.0 - position[1] - radius;
        if dy > 0. {
            No
        } else {
            Yes([0., dy].into())
        }
    }

    #[inline]
    fn on_collision<A>(&mut self, ball: &mut A, overlap: Vector2<f32>)
    where
        A: AnimatedObject,
    {
        ball.correct_and_flip_y(overlap[1]);
    }
}
