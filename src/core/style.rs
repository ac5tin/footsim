#[derive(PartialEq, Eq)]
pub enum PlayStyle {
    // GK
    Sweeper,
    // CB
    BallPlaying,
    // Wide players
    Wide,
    // Wingers
    CutInside,
    TrackBack,
    // Fullbacks
    Inverted,
    // midfield
    BoxToBox,
    // CF
    False9,
    Playmaker,
    Default,
}
