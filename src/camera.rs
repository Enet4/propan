use na::Vector2;
use util::clamp;

#[derive(Debug)]
pub struct Camera {
    pos: Vector2<f32>, // top-left
    width: f32,
    height: f32,
    half_width: f32,
    half_height: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Camera::new([0., 0.].into(), ::WIDTH as f32, ::HEIGHT as f32)
    }
}

impl Camera {
    pub fn new(pos: Vector2<f32>, width: f32, height: f32) -> Self {
        assert!(width > 0.);
        assert!(height > 0.);
        Camera {
            pos,
            width,
            height,
            half_width: width / 2.,
            half_height: height / 2.,
        }
    }

    pub fn position(&self) -> Vector2<f32> {
        self.pos
    }

    pub fn round_position(&mut self) {
        self.pos[0] = self.pos[0].round();
        self.pos[1] = self.pos[1].round();
    }

    pub fn focus_on(&mut self, focus: Vector2<f32>, map_dim: Vector2<f32>) {
        let point = self.focus_point(focus, map_dim);
        self.pos = [point[0] - self.half_width, point[1] - self.half_height].into();
    }

    pub fn pan<V>(&mut self, v: V)
    where
        V: Into<Vector2<f32>>,
    {
        self.pos += v.into();
    }

    pub fn clamp_to_bounds(&mut self, map_dim: Vector2<f32>) {
        let (hw, hh) = (self.half_width, self.half_height);
        self.pos[0] = clamp(self.pos[0], 0., map_dim[0] - hw);
        self.pos[1] = clamp(self.pos[1], 0., map_dim[1] - hh);
    }

    /// Obtain the coordinates that a camera should center on with the given
    /// focus point and display dimensions
    fn focus_point(&self, focus: Vector2<f32>, dim: Vector2<f32>) -> Vector2<f32> {
        let (hw, hh) = (self.half_width, self.half_height);
        let (x, y) = (focus[0], focus[1]);

        let x = clamp(x, hw, dim[0] - hw);
        let y = clamp(y, hh, dim[1] - hh);
        [x, y].into()
    }
}
