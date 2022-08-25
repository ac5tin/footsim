use super::{position, style};

pub struct Player {
    pub id: u32,
    pub name: String,
    pub team_id: u32,
    pub country_id: u32,
    // mental
    pub morale: u8,
    pub form: u8,
    pub tactical: u8,
    pub leadership: u8,
    // physical
    pub fitness: u8,
    pub pace: u8,
    pub strength: u8,
    pub stamina: u8,
    // technical
    pub passing: u8,
    pub technique: u8,
    pub heading: u8,
    pub set_pieces: u8,
    // defending
    pub tackling: u8,
    pub marking: u8,
    pub goalkeeping: u8,
    pub defensive_positioning: u8,
    // attacking
    pub shooting: u8,
    pub long_shots: u8,
    pub attack_positioning: u8,
    // position
    pub position: position::Position,
    pub playstyle: style::PlayStyle,
}

impl Player {
    pub fn new() -> Self {
        Self {
            id: 0,
            name: String::new(),
            team_id: 0,
            country_id: 0,
            morale: 0,
            form: 0,
            tactical: 0,
            leadership: 0,
            fitness: 0,
            pace: 0,
            strength: 0,
            stamina: 0,
            passing: 0,
            technique: 0,
            heading: 0,
            set_pieces: 0,
            tackling: 0,
            marking: 0,
            goalkeeping: 0,
            defensive_positioning: 0,
            shooting: 0,
            long_shots: 0,
            attack_positioning: 0,
            position: position::Position::Goalkeeper,
            playstyle: style::PlayStyle::Default,
        }
    }
}
