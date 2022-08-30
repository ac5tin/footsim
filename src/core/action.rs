/// Player action with a thread score (0 - 1.0)
/// higher threat score means more dangerous action
/// e.g. pass with 0.95 means a threatening pass (killer pass that will probably result in a shot on goal)
/// e.g. shoot with 0.96 means shooting from a very dangerous distance/position
/// e.g. dribble with 0.3 means dribbling from a relatively safe position
pub enum Action {
    // with ball
    Pass(u32, f32), // player with higher creativity can produce passes with higher threat score
    Shoot(f32),     // a threat score = shooting from a dangerous position
    Cross(f32),     // threat score of the cross
    Dribble(f32),   // higher thread score = dribbling into a dangerous position (e.g. in the box)
}
