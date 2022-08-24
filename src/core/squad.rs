use super::{manager, player, tactics};

pub struct Squad<'a> {
    pub manager: &'a manager::Manager,
    pub players: [&'a player::Player; 11],
    pub subs: Vec<&'a player::Player>,
    pub tactics: tactics::Tactics,
}
