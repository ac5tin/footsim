/// Game events
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
