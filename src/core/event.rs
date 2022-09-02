/// Game events
#[derive(Clone, Copy)]
pub enum Event {
    Offside(u32),
    Foul(u32, f32), // player id and foul severity
    YellowCard(u32),
    RedCard(u32),
    Substitution(u32, u32), // player id and new player id
    Goal(u32, Option<u32>), // goal scorer with optional assist
    ThrowIn,
    GoalKick,
    Corner,
    FreeKick,
    Penalty,
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Event::Offside(id) => write!(f, "Offside: {}", id),
            Event::Foul(id, severity) => write!(f, "Foul: {} severity: {}", id, severity),
            Event::YellowCard(id) => write!(f, "Yellow card: {}", id),
            Event::RedCard(id) => write!(f, "Red card: {}", id),
            Event::Substitution(id, new_id) => write!(f, "Substitution: {} -> {}", id, new_id),
            Event::Goal(id, assist) => write!(f, "Goal: {} assist: {:?}", id, assist),
            Event::ThrowIn => write!(f, "Throw in"),
            Event::GoalKick => write!(f, "Goal kick"),
            Event::Corner => write!(f, "Corner"),
            Event::FreeKick => write!(f, "Free kick"),
            Event::Penalty => write!(f, "Penalty"),
        }
    }
}
