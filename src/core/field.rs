#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum FieldZone {
    Left,
    Right,
    Center,
    Box,
}

impl std::fmt::Display for FieldZone {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FieldZone::Left => write!(f, "Left"),
            FieldZone::Right => write!(f, "Right"),
            FieldZone::Center => write!(f, "Center"),
            FieldZone::Box => write!(f, "Box"),
        }
    }
}
