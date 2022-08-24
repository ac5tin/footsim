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

#[cfg(test)]
mod tests {
    use crate::core::{
        manager, player,
        tactics::{Tactics, Width},
    };

    use super::*;

    #[test]
    fn get_team_possession_score() {
        let squad0 = squad::Squad {
            manager: &manager::Manager {
                id: 1,
                name: "Mikel Arteta".to_owned(),
                team_id: 1,
                country_id: 2,
                tactical: 180,
                management: 185,
                coaching: 170,
            },
            players: [
                &player::Player {
                    id: 1,
                    name: "Aaron Ramsdale".to_owned(),
                    team_id: 1,
                    country_id: 1,
                    morale: 200,
                    form: 180,
                    tactical: 150,
                    leadership: 140,
                    fitness: 200,
                    pace: 120,
                    strength: 160,
                    stamina: 140,
                    passing: 155,
                    technique: 100,
                    heading: 80,
                    set_pieces: 60,
                    tackling: 150,
                    marking: 100,
                    goalkeeping: 170,
                    defensive_positioning: 120,
                    shooting: 30,
                    attack_positioning: 20,
                    position: position::Position::Goalkeeper,
                    playstyle: style::PlayStyle::Default,
                },
                &player::Player {
                    id: 2,
                    name: "Ben White".to_owned(),
                    team_id: 1,
                    country_id: 1,
                    morale: 200,
                    form: 140,
                    tactical: 160,
                    leadership: 110,
                    fitness: 180,
                    pace: 150,
                    strength: 160,
                    stamina: 150,
                    passing: 165,
                    technique: 140,
                    heading: 150,
                    set_pieces: 150,
                    tackling: 190,
                    marking: 170,
                    goalkeeping: 20,
                    defensive_positioning: 160,
                    shooting: 30,
                    attack_positioning: 130,
                    position: position::Position::RightBack,
                    playstyle: style::PlayStyle::Default,
                },
                &player::Player {
                    id: 3,
                    name: "Oleksandr Zinchenko".to_owned(),
                    team_id: 1,
                    country_id: 10,
                    morale: 210,
                    form: 200,
                    tactical: 210,
                    leadership: 100,
                    fitness: 200,
                    pace: 150,
                    strength: 130,
                    stamina: 150,
                    passing: 190,
                    technique: 200,
                    heading: 80,
                    set_pieces: 170,
                    tackling: 170,
                    marking: 160,
                    goalkeeping: 10,
                    defensive_positioning: 160,
                    shooting: 30,
                    attack_positioning: 180,
                    position: position::Position::LeftBack,
                    playstyle: style::PlayStyle::Inverted,
                },
                &player::Player {
                    id: 4,
                    name: "Gabriel Magalhaes".to_owned(),
                    team_id: 1,
                    country_id: 5,
                    morale: 200,
                    form: 140,
                    tactical: 150,
                    leadership: 160,
                    fitness: 180,
                    pace: 140,
                    strength: 190,
                    stamina: 170,
                    passing: 130,
                    technique: 110,
                    heading: 190,
                    set_pieces: 60,
                    tackling: 210,
                    marking: 200,
                    goalkeeping: 10,
                    defensive_positioning: 190,
                    shooting: 30,
                    attack_positioning: 20,
                    position: position::Position::CenterBack,
                    playstyle: style::PlayStyle::Default,
                },
                &player::Player {
                    id: 5,
                    name: "William Saliba".to_owned(),
                    team_id: 1,
                    country_id: 4,
                    morale: 210,
                    form: 240,
                    tactical: 180,
                    leadership: 160,
                    fitness: 220,
                    pace: 200,
                    strength: 230,
                    stamina: 170,
                    passing: 180,
                    technique: 150,
                    heading: 210,
                    set_pieces: 30,
                    tackling: 250,
                    marking: 200,
                    goalkeeping: 10,
                    defensive_positioning: 240,
                    shooting: 30,
                    attack_positioning: 20,
                    position: position::Position::CenterBack,
                    playstyle: style::PlayStyle::Default,
                },
                &player::Player {
                    id: 6,
                    name: "Thomas Partey".to_owned(),
                    team_id: 1,
                    country_id: 40,
                    morale: 170,
                    form: 180,
                    tactical: 190,
                    leadership: 150,
                    fitness: 180,
                    pace: 170,
                    strength: 220,
                    stamina: 160,
                    passing: 190,
                    technique: 180,
                    heading: 180,
                    set_pieces: 150,
                    tackling: 190,
                    marking: 180,
                    goalkeeping: 10,
                    defensive_positioning: 190,
                    shooting: 30,
                    attack_positioning: 20,
                    position: position::Position::DefensiveMidfield,
                    playstyle: style::PlayStyle::Default,
                },
                &player::Player {
                    id: 7,
                    name: "Granit Xhaka".to_owned(),
                    team_id: 1,
                    country_id: 9,
                    morale: 250,
                    form: 230,
                    tactical: 200,
                    leadership: 220,
                    fitness: 210,
                    pace: 160,
                    strength: 170,
                    stamina: 180,
                    passing: 210,
                    technique: 200,
                    heading: 140,
                    set_pieces: 190,
                    tackling: 190,
                    marking: 180,
                    goalkeeping: 10,
                    defensive_positioning: 160,
                    shooting: 170,
                    attack_positioning: 140,
                    position: position::Position::CenterMidfield,
                    playstyle: style::PlayStyle::BoxToBox,
                },
                &player::Player {
                    id: 8,
                    name: "Martin Odegaard".to_owned(),
                    team_id: 1,
                    country_id: 33,
                    morale: 240,
                    form: 240,
                    tactical: 230,
                    leadership: 210,
                    fitness: 200,
                    pace: 170,
                    strength: 140,
                    stamina: 170,
                    passing: 240,
                    technique: 240,
                    heading: 140,
                    set_pieces: 210,
                    tackling: 130,
                    marking: 120,
                    goalkeeping: 10,
                    defensive_positioning: 120,
                    shooting: 170,
                    attack_positioning: 180,
                    position: position::Position::CenterMidfield,
                    playstyle: style::PlayStyle::Playmaker,
                },
                &player::Player {
                    id: 9,
                    name: "Bukayo Saka".to_owned(),
                    team_id: 1,
                    country_id: 1,
                    morale: 210,
                    form: 220,
                    tactical: 190,
                    leadership: 140,
                    fitness: 190,
                    pace: 220,
                    strength: 130,
                    stamina: 160,
                    passing: 190,
                    technique: 210,
                    heading: 120,
                    set_pieces: 150,
                    tackling: 130,
                    marking: 140,
                    goalkeeping: 10,
                    defensive_positioning: 130,
                    shooting: 190,
                    attack_positioning: 200,
                    position: position::Position::RightWing,
                    playstyle: style::PlayStyle::CutInside,
                },
                &player::Player {
                    id: 10,
                    name: "Gabriel Martinelli".to_owned(),
                    team_id: 1,
                    country_id: 5,
                    morale: 250,
                    form: 250,
                    tactical: 210,
                    leadership: 130,
                    fitness: 220,
                    pace: 240,
                    strength: 140,
                    stamina: 140,
                    passing: 180,
                    technique: 250,
                    heading: 140,
                    set_pieces: 130,
                    tackling: 120,
                    marking: 110,
                    goalkeeping: 10,
                    defensive_positioning: 90,
                    shooting: 170,
                    attack_positioning: 230,
                    position: position::Position::RightWing,
                    playstyle: style::PlayStyle::CutInside,
                },
                &player::Player {
                    id: 11,
                    name: "Gabriel Jesus".to_owned(),
                    team_id: 1,
                    country_id: 5,
                    morale: 250,
                    form: 250,
                    tactical: 240,
                    leadership: 150,
                    fitness: 220,
                    pace: 220,
                    strength: 180,
                    stamina: 170,
                    passing: 170,
                    technique: 240,
                    heading: 190,
                    set_pieces: 170,
                    tackling: 150,
                    marking: 110,
                    goalkeeping: 10,
                    defensive_positioning: 100,
                    shooting: 230,
                    attack_positioning: 250,
                    position: position::Position::Striker,
                    playstyle: style::PlayStyle::Default,
                },
            ],
            subs: Vec::new(),
            tactics: Tactics {
                defense_line: 200,
                compactness: 120,
                aggression: 150,
                build_up_speed: 150,
                attack_width: Width::Balanced,
                shoot_more_often: false,
                cross_more_often: false,
                pass_range: 100,
            },
        };

        let p = player::Player::new();
        let squad1 = squad::Squad {
            players: [&p; 11],
            manager: &manager::Manager {
                id: 333,
                name: "john doe".to_owned(),
                team_id: 2,
                country_id: 222,
                tactical: 149,
                management: 13,
                coaching: 120,
            },
            subs: Vec::new(),
            tactics: Tactics {
                defense_line: 200,
                compactness: 100,
                aggression: 150,
                build_up_speed: 200,
                attack_width: Width::Balanced,
                shoot_more_often: false,
                cross_more_often: true,
                pass_range: 109,
            },
        };
        let g = Game::new(squad0.clone(), squad1);

        g.get_team_poss_score(&squad0);
    }
}
