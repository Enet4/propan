use na::Vector2;

pub enum ObjectPlaceholder
{
    Wall {
        dim: Vector2<f32>,
        texture_id: u32, 
    },
    Mine,
    Pump,
    Gem,
    Ball,
    Finish,
}

impl ObjectPlaceholder {

    pub fn next(&self) -> ObjectPlaceholder {
        use self::ObjectPlaceholder::*;
        match *self {
            Wall {..} => ObjectPlaceholder::default_mine(),
            Mine => ObjectPlaceholder::default_pump(),
            Pump => ObjectPlaceholder::default_gem(),
            Gem => ObjectPlaceholder::default_ball(),
            Ball => ObjectPlaceholder::default_finish(),
            Finish => ObjectPlaceholder::default_wall(),
        }
    }

    pub fn previous(&self) -> ObjectPlaceholder {
        use self::ObjectPlaceholder::*;
        match *self {
            Wall {..} => ObjectPlaceholder::default_finish(),
            Mine => ObjectPlaceholder::default_wall(),
            Pump => ObjectPlaceholder::default_mine(),
            Gem => ObjectPlaceholder::default_pump(),
            Ball => ObjectPlaceholder::default_gem(),
            Finish => ObjectPlaceholder::default_ball(),
        }
    }

    pub fn default_ball() -> ObjectPlaceholder {
        ObjectPlaceholder::Ball
    }
    pub fn default_wall() -> ObjectPlaceholder {
        ObjectPlaceholder::Wall {
            dim: [48., 48.].into(),
            texture_id: 0,
        }
    }
    pub fn default_mine() -> ObjectPlaceholder {
        ObjectPlaceholder::Mine
    }
    pub fn default_pump() -> ObjectPlaceholder {
        ObjectPlaceholder::Pump
    }
    pub fn default_gem() -> ObjectPlaceholder {
        ObjectPlaceholder::Gem
    }
    pub fn default_finish() -> ObjectPlaceholder {
        ObjectPlaceholder::Finish
    }
}

