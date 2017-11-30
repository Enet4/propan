use na::Vector2;
use std::fmt::Debug;

pub fn default_vector2<T>() -> Vector2<T>
where
    T: Debug + Copy + Default + PartialEq + 'static
{
    [T::default() ; 2].into()
}

pub fn clamp(v: f32, min: f32, max: f32) -> f32 {
    f32::max(min, f32::min(max, v))
}
