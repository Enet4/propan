use std::cell::RefMut;
use super::Result;

pub trait AudioManage {

}
impl<'a, T: AudioManage> AudioManage for &'a T {}
impl<'a, T: AudioManage> AudioManage for &'a mut T {}
impl<'a, T: AudioManage> AudioManage for RefMut<'a, T> {}

pub struct AudioManager {
}

impl AudioManager {

    pub fn new(_: ()) -> Result<Self> {
        // TODO
        Ok(AudioManager{})
    }
}

impl AudioManage for AudioManager {}
