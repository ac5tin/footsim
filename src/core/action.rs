use strum_macros::EnumIter;

/// Player action with a thread score (0 - 1.0)
/// higher threat score means more dangerous action
/// e.g. pass with 0.95 means a threatening pass (killer pass that will probably result in a shot on goal)
/// e.g. shoot with 0.96 means shooting from a very dangerous distance/position
/// e.g. dribble with 0.3 means dribbling from a relatively safe position
#[derive(Clone, Copy, Hash, Eq, PartialEq, EnumIter, Debug)]
pub enum Action {
    // with ball
    Pass,    // player with higher creativity can produce passes with higher threat score
    Shoot,   // a threat score = shooting from a dangerous position
    Cross,   // threat score of the cross
    Dribble, // higher thread score = dribbling into a dangerous position (e.g. in the box)
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Action::Pass => write!(f, "Pass"),
            Action::Shoot => write!(f, "Shoot"),
            Action::Cross => write!(f, "Cross"),
            Action::Dribble => write!(f, "Dribble"),
        }
    }
}
