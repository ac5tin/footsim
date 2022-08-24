pub struct Tactics {
    // high defensive line = more pressure but more dangerous if press fails
    pub defense_line: u8,
    // higher compactness = sacrificing width but harder to penetrate throught the middle
    pub compactness: u8,
    // higher = more likely to tackle
    pub aggression: u8,
    // higher build_up_speed = more risky but more likely to create chances
    pub build_up_speed: u8,
    // area to put more focus during attack
    pub attack_width: Width,
    // shoot more often however more likely to miss
    pub shoot_more_often: bool,
    // higher = more likely to cross
    pub cross_more_often: bool,
    // higher = more long passes but more likely to lose the ball
    pub pass_range: u8,
}

pub enum Width {
    Central,
    Left,
    Right,
    Balanced,
}
