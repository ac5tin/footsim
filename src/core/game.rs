use super::{position, squad, style};

pub struct Game<'a> {
    home: squad::Squad<'a>,
    away: squad::Squad<'a>,
    home_stats: GameStats,
    away_stats: GameStats,
}

#[derive(Default)]
pub struct GameStats {
    possession: f32,
    shots: u8,
    shots_on_target: u8,
    goals: u8,
    freekicks: u8,
    corners: u8,
    fouls: u8,
    yellow_cards: u8,
    red_cards: u8,
}

impl<'a> Game<'a> {
    pub fn new(home_squad: squad::Squad<'a>, away_squad: squad::Squad<'a>) -> Self {
        Self {
            home: home_squad,
            away: away_squad,
            home_stats: GameStats::default(),
            away_stats: GameStats::default(),
        }
    }

    pub fn play(&self) {}

    fn play_half(&self) {
        // calculate possession of each team
        let (home_poss, away_poss) = self.get_possession();
        // based on possession calculate shots
        // calculate corners
        // calculate fouls based on possession
        // based on fouls calculate freekicks and yellow cards and red cards
        // calculate freekicks
        // based on shots and corners and freekicks calculate shots on target
        // based on shots on target calculate goals
    }

    /// return posession for each team
    /// calculated based on:
    /// - home / away
    /// - tactics
    /// - manager
    /// - players
    ///     - technical abilities
    ///     - tactics adaptabilities
    ///     - fitness
    ///     - morale
    ///     - form
    ///     - stamina
    /// first value is home team, second value is away team
    fn get_possession(&self) -> (f32, f32) {
        // (tactics + formation + player playstyle) * tactics success rate * quality of players * home adv
        // home team
        let home_score = self.get_team_poss_score(&self.home) * 1.1;
        let away_score = self.get_team_poss_score(&self.away);

        let total = home_score + away_score;

        (home_score / total, away_score / total)
    }

    fn get_team_poss_score(&self, squad: &squad::Squad) -> f32 {
        // --- tactics: pressure, buildup, ball retention, pass_range ---
        let pressure = squad.tactics.defense_line as f32
            * (u8::MAX - squad.tactics.compactness + 1) as f32
            * (squad.tactics.aggression as f32 * 0.1)
            * 0.01
            + 1.0;

        let tact_score = pressure
            + (((u8::MAX - squad.tactics.build_up_speed + 1)
                * (u8::MAX - squad.tactics.pass_range + 1)) as f32
                + 1.0)
                * 0.01;
        // tact range: 1.2 -> 23084.97
        // --- formation: number of ppl in the middle of midfield ---
        let mut formation_score = 0.0;

        let mut players_score = 0.0;
        for &p in squad.players.iter() {
            let pos_score = match p.position {
                position::Position::DefensiveMidfield
                | position::Position::LeftMidfield
                | position::Position::RightMidfield
                | position::Position::CenterMidfield
                | position::Position::AttackingMidfield => 5.0,

                _ => 1.0,
            };
            let style_score = match p.playstyle {
                style::PlayStyle::Sweeper => match p.position {
                    position::Position::Goalkeeper => 3.0,
                    _ => 1.0,
                },
                style::PlayStyle::BallPlaying => match p.position {
                    position::Position::CenterBack => 5.0,
                    _ => 1.0,
                },
                style::PlayStyle::Inverted => match p.position {
                    position::Position::LeftBack
                    | position::Position::LeftWing
                    | position::Position::RightBack
                    | position::Position::RightWingBack => 12.0,
                    _ => 1.0,
                },
                style::PlayStyle::Playmaker => match p.position {
                    position::Position::DefensiveMidfield
                    | position::Position::CenterMidfield
                    | position::Position::AttackingMidfield => 8.0,
                    position::Position::Striker => 6.0,
                    _ => 5.0,
                },
                _ => 1.0,
            };
            formation_score += (pos_score * style_score) * 0.01;
            // formation_score range: 0.017 -> 0.96

            // player score
            let player_score_multiplier = match p.position {
                position::Position::Goalkeeper => 0.7,
                position::Position::DefensiveMidfield
                | position::Position::CenterMidfield
                | position::Position::AttackingMidfield => 1.5,
                _ => 1.0,
            };
            players_score += p.passing as f32
                + (p.fitness as f32 * 0.5)
                + (p.stamina as f32 / squad.tactics.defense_line as f32 * 0.5)
                    * (player_score_multiplier
                        * (1.0 + p.form as f32 * 0.01)
                        * (p.morale as f32 * 0.01)
                        * 0.01);
            // players_score range: 0.007 -> 3.82
        }
        // tact_score * formation_score
        // --- tactics success rate: players tactic understanding + manager quality ---
        let manager_score = squad.manager.tactical as f32 + squad.manager.management as f32 * 0.7;
        // 1.7 -> 433.5

        tact_score * formation_score * manager_score * 0.0001 * players_score
    }
}
