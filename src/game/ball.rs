use piston::input::GenericEvent;
use graphics::{ellipse, Context, Graphics};
use na::{norm_squared, Vector2};
use physics::{rigid_bounce, AnimatedObject, Collidable, CollisionInfo, SimpleCollidable};
use util::default_vector2;
use resource::{ResourceManage, Result};

const BALL_CAPACITY: f32 = 34.;
pub const BALL_DEFAULT_SIZE: f32 = 28.;
const TOO_MUCH_SPEED_SQR: f32 = 22.;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ball {
    #[serde(default = "default_vector2")] pos: Vector2<f32>,
    #[serde(skip)]
    #[serde(default = "default_vector2")]
    vel: Vector2<f32>,
    #[serde(default = "Ball::default_size")] size: f32,
}

impl Ball {
    pub fn default_size() -> f32 {
        BALL_DEFAULT_SIZE
    }

    pub fn new<P>(position: P, size: f32) -> Ball
    where
        P: Into<Vector2<f32>>,
    {
        Ball {
            pos: position.into(),
            vel: default_vector2(),
            size,
        }
    }

    pub fn with_default_size<P>(pos: P) -> Ball
    where
        P: Into<Vector2<f32>>,
    {
        Ball::new(pos, BALL_DEFAULT_SIZE)
    }

    pub fn position(&self) -> Vector2<f32> {
        self.pos
    }

    pub fn velocity(&self) -> Vector2<f32> {
        self.vel
    }

    pub fn set_position<P>(&mut self, pos: P)
    where
        P: Into<Vector2<f32>>,
    {
        self.pos = pos.into();
    }

    pub fn add_position<P>(&mut self, pos: P)
    where
        P: Into<Vector2<f32>>,
    {
        self.pos += pos.into();
    }

    pub fn speed_sqr(&self) -> f32 {
        norm_squared(&self.vel)
    }

    /// Get the size (diameter) of the ball.
    #[inline]
    pub fn size(&self) -> f32 {
        self.size
    }

    #[inline]
    pub fn capacity(&self) -> f32 {
        BALL_CAPACITY
    }

    pub fn thrust<V>(&mut self, thrust: V)
    where
        V: Into<Vector2<f32>>,
    {
        self.vel += thrust.into()
    }

    pub fn decay_velocity(&mut self, factor: f32) {
        self.vel -= self.vel * factor;
    }

    pub fn set_velocity<V>(&mut self, velocity: V)
    where
        V: Into<Vector2<f32>>,
    {
        self.vel = velocity.into();
    }

    pub fn add_velocity<V>(&mut self, velocity: V)
    where
        V: Into<Vector2<f32>>,
    {
        self.vel += velocity.into();
    }

    pub fn flip_vx(&mut self) {
        self.vel[0] = -self.vel[0];
    }

    pub fn flip_vy(&mut self) {
        self.vel[1] = -self.vel[1];
    }

    pub fn add_size(&mut self, size_delta: f32) {
        self.size = f32::max(0., self.size + size_delta);
    }

    pub fn is_dead(&self) -> bool {
        self.size < 4. || self.size > BALL_CAPACITY + 2.
    }

    pub fn update_position(&mut self, factor: f32) {
        self.pos += self.vel * factor;
    }

    pub fn maximize_size(&mut self) {
        self.size = BALL_CAPACITY;
    }

    pub fn draw<G: Graphics>(&self, ctx: Context, gfx: &mut G) {
        if self.is_dead() {
            return;
        }

        let color = if self.size < 5.5 {
            [0.7, 0.5, 0.9, 1.0]
        } else if self.size > BALL_CAPACITY - 2.5 {
            [0.7, 0.88, 1.0, 0.8]
        } else {
            [0.5, 0.86, 1.0, 1.0]
        };
        let (x, y) = (self.pos[0] as f64, self.pos[1] as f64);
        let hsize = (self.size / 2.) as f64;
        let draw_size = (self.size + 4.) as f64;
        let r = [
            x - hsize - 2.,
            y - hsize - 2.,
            draw_size,
            draw_size,
        ];

        ellipse(color, r, ctx.transform, gfx);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BallController<R> {
    ball: Ball,
    #[serde(skip)] thrust_right: bool,
    #[serde(skip)] thrust_left: bool,
    #[serde(skip)] thrust_up: bool,
    #[serde(skip)] thrust_down: bool,
    #[serde(skip)]
    #[serde(default = "default_vector2")]
    acc_overlaps: Vector2<f32>,
    num_overlaps: usize,
    num_gems: u32,
    resource_manager: R,
}

impl<R> BallController<R>
where
    R: ResourceManage,
{
    /// Creates a new gameboard controller.
    pub fn new(ball: Ball, resource_manager: R) -> Result<Self> {
        Ok(BallController {
            ball,
            thrust_right: false,
            thrust_left: false,
            thrust_up: false,
            thrust_down: false,
            acc_overlaps: default_vector2(),
            num_overlaps: 0,
            num_gems: 0,
            resource_manager,
        })
    }

    /// Handles events.
    pub fn event<E: GenericEvent>(&mut self, e: &E) {
        use piston::input::{Button, ButtonState, Key};
        if let Some(b) = e.button_args() {
            // Set cell value.
            if let Button::Keyboard(k) = b.button {
                match (k, b.state, b.scancode) {
                    (Key::Right, state, _) | (Key::NumPad6, state, _) => {
                        self.thrust_right = state == ButtonState::Press;
                    }
                    (Key::Left, state, _) | (Key::NumPad4, state, _) => {
                        self.thrust_left = state == ButtonState::Press;
                    }
                    (Key::Up, state, _) | (Key::NumPad8, state, _) => {
                        self.thrust_up = state == ButtonState::Press;
                    }
                    (Key::Down, state, _) | (Key::NumPad2, state, _) => {
                        self.thrust_down = state == ButtonState::Press;
                    }
                    _ => {
                        // do nothing
                    }
                }
            }
        }
    }

    pub fn update(&mut self, factor: f32) {
        if self.is_dead() {
            return;
        }

        // resolve collisions
        if self.num_overlaps > 0 {
            // average the collision vectors
            let overlap = self.acc_overlaps / self.num_overlaps as f32;
            self.correct_and_rigid_bounce(overlap);
            // dampen velocity a little bit
            self.ball.decay_velocity(6e-3);
            self.acc_overlaps = default_vector2();
            self.num_overlaps = 0;
        }

        let thrust_force: f32 = 6.0e-2 * factor;
        let mut total_effort = 0;
        if self.thrust_right {
            self.ball.thrust([thrust_force, 0.]);
            total_effort += 1;
        }
        if self.thrust_left {
            self.ball.thrust([-thrust_force, 0.]);
            total_effort += 1;
        }
        if self.thrust_up {
            self.ball.thrust([0., -thrust_force]);
            total_effort += 1;
        }
        if self.thrust_down {
            self.ball.thrust([0., thrust_force]);
            total_effort += 1;
        }

        if self.ball.speed_sqr() > TOO_MUCH_SPEED_SQR {
            self.ball.decay_velocity(5e-3);
        }

        self.ball.update_position(factor);
        self.ball.add_size(total_effort as f32 * -1.2e-2 * factor);
    }

    #[inline]
    pub fn position(&self) -> Vector2<f32> {
        self.ball.position()
    }

    #[inline]
    pub fn size(&self) -> f32 {
        self.ball.size()
    }

    #[inline]
    pub fn set_position(&mut self, pos: Vector2<f32>) {
        self.ball.set_position(pos)
    }

    #[inline]
    pub fn add_position(&mut self, pos: Vector2<f32>) {
        self.ball.add_position(pos)
    }

    #[inline]
    pub fn velocity(&self) -> Vector2<f32> {
        self.ball.velocity()
    }

    #[inline]
    pub fn set_velocity(&mut self, velocity: Vector2<f32>) {
        self.ball.set_velocity(velocity)
    }

    #[inline]
    pub fn add_size(&mut self, extra_size: f32) {
        self.ball.add_size(extra_size)
    }

    #[inline]
    pub fn draw<G: Graphics>(&self, ctx: Context, gfx: &mut G) {
        self.ball.draw(ctx, gfx)
    }

    #[inline]
    pub fn is_dead(&self) -> bool {
        self.ball.is_dead()
    }

    fn correct_and_rigid_bounce(&mut self, overlap: Vector2<f32>) {
        // correct position to not overlap
        self.ball.add_position(overlap);
        // and bounce
        let vel = self.ball.velocity();
        self.ball.set_velocity(rigid_bounce(vel, overlap));
    }

    pub fn handle_collision_with<T>(&mut self, mut object: T)
    where
        T: Collidable,
    {
        let collision = object.test_circle_collision(self.ball.position(), self.ball.size() / 2.);
        if let CollisionInfo::Yes(overlap) = collision {
            object.on_collision(self, overlap);
        }
    }

    pub fn handle_simple_collision_with<T>(&mut self, mut object: T)
    where
        T: SimpleCollidable,
    {
        if object.test_circle_collision_simple(self.ball.position(), self.ball.size() / 2.) {
            object.on_collision_simple(self);
        }
    }
}

impl<R> AnimatedObject for BallController<R>
where
    R: ResourceManage,
{
    fn issue_bounce(&mut self, overlap: Vector2<f32>) {
        self.acc_overlaps += overlap;
        self.num_overlaps += 1;
    }

    fn correct_and_flip_x(&mut self, overlap: f32) {
        // correct position to not overlap
        self.ball.add_position([overlap, 0.]);
        // and bounce horizontally
        self.ball.flip_vx();
    }

    fn correct_and_flip_y(&mut self, overlap: f32) {
        // correct position to not overlap
        self.ball.add_position([0., overlap]);
        // and bounce vertically
        self.ball.flip_vy();
    }

    fn add_velocity(&mut self, extra_velocity: Vector2<f32>) {
        self.ball.add_velocity(extra_velocity)
    }

    fn add_position(&mut self, translation: Vector2<f32>) {
        self.ball.add_position(translation)
    }

    fn damage(&mut self, dmg: f32) {
        self.ball.add_size(-dmg)
    }

    fn heal(&mut self, health: f32) {
        self.ball.add_size(health)
    }

    fn pick_up(&mut self, _item: ()) {
        self.num_gems += 1;
    }

    fn items(&self) -> u32 {
        self.num_gems
    }
}
