use std::sync::Arc;
use std::sync::RwLock;

use rand::{rngs::ThreadRng, thread_rng, Rng};

use super::player;
use super::tactics;
use super::{position, squad, style};

pub struct Game<'a> {
    home: squad::Squad<'a>,
    away: squad::Squad<'a>,
    home_stats: GameStats,
    away_stats: GameStats,
    rng: Arc<RwLock<ThreadRng>>,
}

#[derive(Default, Clone)]
pub struct GameStats {
    pub possession: f32,
    pub crosses: u8,
    pub shots: u8,
    pub shots_on_target: u8,
    pub goals: u8,
    pub freekicks: u8,
    pub penalties: u8,
    pub corners: u8,
    pub fouls: u8,
    pub yellow_cards: Vec<u32>,
    pub red_cards: Vec<u32>,
}

impl<'a> Game<'a> {
    pub fn new(home_squad: squad::Squad<'a>, away_squad: squad::Squad<'a>) -> Self {
        Self {
            home: home_squad,
            away: away_squad,
            home_stats: GameStats::default(),
            away_stats: GameStats::default(),
            rng: Arc::new(RwLock::new(thread_rng())),
        }
    }

    pub fn play(&mut self) {
        self.play_half();
    }

    pub fn get_home_stats(&self) -> GameStats {
        self.home_stats.to_owned()
    }
    pub fn get_away_stats(&self) -> GameStats {
        self.away_stats.to_owned()
    }

    fn play_half(&mut self) {
        let (mut home_stats, mut away_stats) = (GameStats::default(), GameStats::default());
        // carry over stats
        {
            // bookings
            home_stats.yellow_cards = self.home_stats.yellow_cards.clone();
            away_stats.yellow_cards = self.away_stats.yellow_cards.clone();
            home_stats.red_cards = self.home_stats.red_cards.clone();
            away_stats.red_cards = self.away_stats.red_cards.clone();
            // goals
            home_stats.goals = self.home_stats.goals;
            away_stats.goals = self.away_stats.goals;
        }
        // get players
        let home_players = self
            .get_players(&self.home, home_stats.clone())
            .collect::<Vec<_>>();
        let away_players = self
            .get_players(&self.away, away_stats.clone())
            .collect::<Vec<_>>();
        // --- get squad strength --
        let home_def = self.get_squad_def_strength(&self.home, &home_stats);
        let away_def = self.get_squad_def_strength(&self.away, &away_stats);
        // get aerial threat , defense
        let home_aerial_threat = self.get_atk_aerial(home_players.clone().into_iter());
        let home_aerial_def = self.get_def_aerial(home_players.clone().into_iter());
        let away_aerial_threat = self.get_atk_aerial(away_players.clone().into_iter());
        let away_aerial_def = self.get_def_aerial(away_players.clone().into_iter());
        let home_wide_atk = self.get_wide_atk(&self.home, home_players.clone().into_iter());
        let away_wide_atk = self.get_wide_atk(&self.away, away_players.clone().into_iter());
        let home_wide_def = self.get_wide_def(&self.home, home_players.clone().into_iter());
        let away_wide_def = self.get_wide_def(&self.home, away_players.clone().into_iter());
        // --- calculations ---
        // calculate possession of each team
        let (home_poss, away_poss) =
            self.get_possession(&self.home, &self.away, &home_stats, &away_stats);
        {
            // modify stats
            home_stats.possession = home_poss;
            away_stats.possession = away_poss;
        }
        // calculate fouls based on possession
        // based on fouls calculate freekicks and yellow cards and red cards
        let (home_fouls, home_yellows, home_reds) = self.get_fouls(&self.home, &home_stats);
        let (away_fouls, away_yellows, away_reds) = self.get_fouls(&self.away, &away_stats);
        {
            // modify stats
            home_stats.fouls += home_fouls;
            home_stats.yellow_cards.extend(home_yellows);
            home_stats.red_cards.extend(home_reds);
            away_stats.fouls += away_fouls;
            away_stats.yellow_cards.extend(away_yellows);
            away_stats.red_cards.extend(away_reds);
        }

        // modify posession based on red cards
        // based on possession get crosses
        let home_crosses = self.get_crosses(&self.home, &home_stats, home_wide_atk, away_wide_def);
        let away_crosses = self.get_crosses(&self.away, &away_stats, away_wide_atk, home_wide_def);
        {
            // modify stats
            home_stats.crosses += home_crosses;
            away_stats.crosses += away_crosses;
        }
        // based on possession and tactics calculate shots
        let home_shots = self.get_shots(
            &self.home,
            home_players.clone().into_iter(),
            &home_stats,
            home_aerial_threat,
            away_aerial_def,
            away_def,
        );
        let away_shots = self.get_shots(
            &self.away,
            away_players.clone().into_iter(),
            &away_stats,
            away_aerial_threat,
            home_aerial_def,
            home_def,
        );
        {
            // modify stats
            home_stats.shots += home_shots;
            away_stats.shots += away_shots;
        }
        // calculate setpieces: corners, freekicks, penalties(based on fouls)
        let (home_ck, home_fk, home_pn) = self.get_set_pieces(&self.home, &home_stats, &away_stats);
        let (away_ck, away_fk, away_pn) = self.get_set_pieces(&self.away, &away_stats, &home_stats);
        {
            // modify stats
            home_stats.corners += home_ck;
            home_stats.freekicks += home_fk;
            home_stats.penalties += home_pn;
            away_stats.corners += away_ck;
            away_stats.freekicks += away_fk;
            away_stats.penalties += away_pn;
        }
        // based on shots and corners and freekicks calculate shots on target
        let home_sot =
            self.get_shots_on_target(&self.home, home_players.clone().into_iter(), &home_stats);
        let away_sot =
            self.get_shots_on_target(&self.away, away_players.clone().into_iter(), &away_stats);
        {
            // modify stats
            home_stats.shots_on_target += home_sot;
            away_stats.shots_on_target += away_sot;
        }
        // based on shots on target calculate goals
        let home_goals = self.get_goals(&self.away, &home_stats);
        let away_goals = self.get_goals(&self.home, &away_stats);
        {
            // modify stats
            home_stats.goals += home_goals;
            away_stats.goals += away_goals;
        }
        // add game_half stats back to game stats
        {
            self.home_stats.possession = home_stats.possession;
            self.home_stats.shots += home_stats.shots;
            self.home_stats.shots_on_target += home_stats.shots_on_target;
            self.home_stats.goals += home_stats.goals;
            self.home_stats.freekicks += home_stats.freekicks;
            self.home_stats.penalties += home_stats.penalties;
            self.home_stats.corners += home_stats.corners;
            self.home_stats.fouls += home_stats.fouls;
            self.home_stats.yellow_cards.extend(home_stats.yellow_cards);
            self.home_stats.red_cards.extend(home_stats.red_cards);

            self.away_stats.possession = away_stats.possession;
            self.away_stats.shots += away_stats.shots;
            self.away_stats.shots_on_target += away_stats.shots_on_target;
            self.away_stats.goals += away_stats.goals;
            self.away_stats.freekicks += away_stats.freekicks;
            self.away_stats.penalties += away_stats.penalties;
            self.away_stats.corners += away_stats.corners;
            self.away_stats.fouls += away_stats.fouls;
            self.away_stats.yellow_cards.extend(away_stats.yellow_cards);
            self.away_stats.red_cards.extend(away_stats.red_cards);
        }
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
    fn get_possession(
        &self,
        home_team: &squad::Squad,
        away_team: &squad::Squad,
        home_stats: &GameStats,
        away_stats: &GameStats,
    ) -> (f32, f32) {
        // (tactics + formation + player playstyle) * tactics success rate * quality of players * home adv
        // home team
        let home_score = self.get_team_poss_score(home_team, home_stats) * 1.1;
        let away_score = self.get_team_poss_score(away_team, away_stats);

        let total = home_score + away_score;

        (home_score / total, away_score / total)
    }

    /// return fouls, yellow and red cards for each team
    /// calculated based on:
    /// - stamina
    /// - decision making
    /// - tactics
    /// - possession
    /// - existing cards
    ///
    fn get_fouls(&self, team: &squad::Squad, stats: &GameStats) -> (u8, Vec<u32>, Vec<u32>) {
        let mut rng = self.rng.write().unwrap();
        let mut fouls: f32 = 0.0;
        let mut yellow_cards: Vec<u32> = Vec::new();
        let mut red_cards: Vec<u32> = Vec::new();
        for &player in team
            .players
            .iter()
            .filter(|p| !stats.red_cards.contains(&p.id))
        {
            // less stamina = more easily tired = more chance to commit a foul
            let mut player_foul: f32 = 0.0;
            player_foul += u8::MAX as f32 / player.stamina as f32 * 0.1;
            player_foul += u8::MAX as f32 / player.decision_making as f32 * 0.4;
            player_foul += team.tactics.aggression as f32 / player.tackling as f32 * 0.1;
            // yellow_card rate
            if rng.gen_bool((player_foul * 0.001) as f64) {
                yellow_cards.push(player.id);
            };
            // red card rate
            if rng.gen_bool((player_foul * 0.0005) as f64) {
                red_cards.push(player.id);
            };
        }
        fouls *= stats.possession;
        (fouls.round() as u8, yellow_cards, red_cards)
    }

    /// get number of shots for the team
    /// calculated based on:
    /// - tactics: shoot_more_often, cross_more_often
    /// - player creativity, passing,technique
    /// - box aerial battle
    fn get_shots(
        &self,
        team: &squad::Squad,
        players: impl Iterator<Item = &'a &'a player::Player>,
        stats: &GameStats,
        aerial_atk: f32,
        opp_aerial_def: f32,
        opp_def_str: f32,
    ) -> u8 {
        let mut rng = self.rng.write().unwrap();
        let mut shots: f32 = 0.0;
        for player in players {
            shots += player.creativity as f32 * 0.4;
            shots += player.passing as f32 * 0.15;
            shots += player.technique as f32 * 0.15;
        }

        shots /= opp_def_str;

        if team.tactics.shoot_more_often {
            shots *= 1.5;
        }

        shots *= rng.gen_range(0.5..1.3) * 10.0;

        let mut total = shots.round() as u8;

        // aerial duels (crosses)
        for _ in 0..stats.crosses {
            if rng.gen_bool((aerial_atk / opp_aerial_def) as f64 * 0.5) {
                total += 1;
            }
        }
        total
    }

    fn get_shots_on_target(
        &self,
        team: &squad::Squad,
        players: impl Iterator<Item = &'a &'a player::Player>,
        stats: &GameStats,
    ) -> u8 {
        let mut rng = self.rng.write().unwrap();
        let mut shooting_acc = 0.01;
        let mut i = 0;
        for p in players {
            let multiplier = match p.position {
                position::Position::Striker => 1.0,
                position::Position::LeftWing | position::Position::RightWing => 0.8,
                position::Position::LeftMidfield
                | position::Position::AttackingMidfield
                | position::Position::RightMidfield => 0.7,
                position::Position::CenterMidfield => 0.6,
                position::Position::Goalkeeper => continue,
                _ => 0.2,
            };
            i += 1;
            shooting_acc += p.shooting as f32 * multiplier;
        }
        shooting_acc /= i as f32;

        if team.tactics.shoot_more_often {
            shooting_acc *= 0.8;
        }

        shooting_acc *= 0.01;

        let mut total = 0;
        for _ in 0..stats.shots {
            let rnd = rng.gen_range(0.8..1.3);
            let mut chance = shooting_acc as f64 * rnd;
            if chance >= 1.0 {
                chance = 0.99;
            }
            if rng.gen_bool(chance) {
                total += 1;
            }
        }
        total
    }

    fn get_goals(&self, opp: &squad::Squad, stats: &GameStats) -> u8 {
        let mut rng = self.rng.write().unwrap();
        let keeper = opp
            .players
            .iter()
            .find(|p| p.position == position::Position::Goalkeeper)
            .unwrap();

        let mut goals = 0;
        for _ in 0..stats.shots_on_target {
            // can opponent keeper save the shot
            if !rng.gen_bool(keeper.goalkeeping as f64 * 0.0035) {
                // keeper fails to make a save
                goals += 1;
            }
        }
        goals
    }

    /// get number of crosses for the team
    /// factors:
    /// - tactics: cross_more_often, attack width
    /// - opp_tactics: compactness
    /// - wide players passing ability
    /// - opp wide players defending ability
    fn get_crosses(
        &self,
        team: &squad::Squad,
        stats: &GameStats,
        wide_atk: (f32, f32),
        opp_wide_def: (f32, f32),
    ) -> u8 {
        let mut rng = self.rng.write().unwrap();

        let mut crosses: f32 = rng.gen_range(1.0..30.0);
        match team.tactics.attack_width {
            tactics::Width::Central => {
                crosses *= 0.7;
            }
            tactics::Width::Balanced => {
                crosses *=
                    ((wide_atk.0 + wide_atk.1) / 2.0) / ((opp_wide_def.0 + opp_wide_def.1) / 2.0);
            }
            tactics::Width::Left => {
                crosses *= wide_atk.0 / opp_wide_def.1;
            }
            tactics::Width::Right => {
                crosses *= wide_atk.1 / opp_wide_def.0;
            }
        }
        if team.tactics.cross_more_often {
            crosses *= 2.0;
        }
        crosses *= stats.possession;
        crosses.round() as u8
    }

    /// get number of corners, freekicks and penalties
    /// calculated based on:
    /// - tactics: shoot_more_often
    /// - fouls of opponent team
    fn get_set_pieces(
        &self,
        team: &squad::Squad,
        stats: &GameStats,
        opp_stats: &GameStats,
    ) -> (u8, u8, u8) {
        let mut rng = self.rng.write().unwrap();
        let mut corner_rate: f32 = 10.0; // base rate
        if team.tactics.shoot_more_often {
            corner_rate += 10.0;
        }
        if team.tactics.attack_width != tactics::Width::Central {
            corner_rate += 5.0;
        }
        corner_rate *= stats.possession;
        if corner_rate > 3.0 {
            corner_rate += rng.gen_range(-2.5..3.5);
        }

        let corners: u8 = rng.gen_range(0..corner_rate.round() as u8);

        let mut freekicks: u8 = 0;
        if opp_stats.fouls > 0 {
            freekicks = rng.gen_range(opp_stats.yellow_cards.len() as u8..opp_stats.fouls);
        }

        let penalties: f32 = opp_stats.fouls as f32 * rng.gen_range(0.01..0.1);

        (corners, freekicks, penalties.round() as u8)
    }

    fn get_team_poss_score(&self, squad: &squad::Squad, stats: &GameStats) -> f32 {
        // --- tactics: pressure, buildup, ball retention, pass_range ---
        let pressure = squad.tactics.defense_line as f32
            * (u8::MAX - squad.tactics.compactness + 1) as f32
            * (squad.tactics.aggression as f32 * 0.1)
            * 0.01
            + 1.0;

        let tact_score = pressure
            + (((u8::MAX - squad.tactics.build_up_speed + 1) as f32
                * (u8::MAX - squad.tactics.pass_range + 1) as f32)
                + 1.0)
                * 0.01;
        // tact range: 1.2 -> 23084.97
        // --- formation: number of ppl in the middle of midfield ---
        let mut formation_score = 0.0;

        let mut players_score = 0.0;
        for &p in squad
            .players
            .iter()
            .filter(|p| !stats.red_cards.contains(&p.id))
        {
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
                + (p.technique as f32 * 0.75)
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

    /// get squad defensive strength score
    /// return defense_strength
    fn get_squad_def_strength(&self, team: &squad::Squad, stats: &GameStats) -> f32 {
        let mut def = 0.0;
        for &p in team
            .players
            .iter()
            .filter(|p| !stats.red_cards.contains(&p.id))
        {
            if p.position == position::Position::Goalkeeper {
                def += p.goalkeeping as f32 * 0.5;
                continue;
            }
            let multiplier = match p.position {
                position::Position::CenterBack => 0.7,
                position::Position::LeftBack | position::Position::RightBack => 0.5,
                position::Position::LeftWingBack | position::Position::RightWingBack => 0.4,
                position::Position::DefensiveMidfield => 0.35,
                position::Position::CenterMidfield => 0.3,
                _ => 0.1,
            };
            def += (p.defensive_positioning as f32 + p.tackling as f32 + p.marking as f32)
                * multiplier;
        }
        def
    }

    fn get_players(
        &self,
        team: &'a squad::Squad,
        stats: GameStats,
    ) -> impl Iterator<Item = &'a &'a player::Player> {
        team.players
            .iter()
            .filter(move |p| !stats.red_cards.contains(&p.id))
    }

    /// get defensive aerial strength
    /// box defensive capabilities to deal with high balls
    /// factors:
    /// - height
    /// - jumping
    /// - strength
    /// - heading
    /// - defensive positioning
    /// - marking
    fn get_def_aerial(&self, players: impl Iterator<Item = &'a &'a player::Player>) -> f32 {
        let mut def = 0.0;
        for p in players {
            if p.position == position::Position::Goalkeeper {
                def += p.goalkeeping as f32 + p.jumping as f32 + p.height as f32;
                continue;
            }
            let multiplier = match p.position {
                position::Position::CenterBack => 1.0,
                position::Position::LeftBack | position::Position::RightBack => 0.6,
                position::Position::LeftWingBack | position::Position::RightWingBack => 0.5,
                position::Position::DefensiveMidfield => 0.35,
                position::Position::CenterMidfield => 0.2,
                _ => 0.01,
            };
            def += (p.height as f32
                + p.jumping as f32
                + p.strength as f32 * 0.9
                + p.heading as f32 * 0.7
                + p.defensive_positioning as f32 * 0.6
                + p.marking as f32 * 0.5)
                * multiplier;
        }
        // back5-max: 5035, back 4-max: 3837
        def
    }

    /// get attacking aerial strength
    /// box attacking capabilities to deal with high balls
    /// factors:
    /// - height
    /// - jumping
    /// - strength
    /// - heading
    /// - attack positioning
    fn get_atk_aerial(&self, players: impl Iterator<Item = &'a &'a player::Player>) -> f32 {
        let mut atk = 0.0;
        for p in players {
            {
                let multiplier = match p.position {
                    position::Position::Striker => 1.0,
                    position::Position::RightWing | position::Position::LeftWing => 0.6,
                    position::Position::AttackingMidfield
                    | position::Position::LeftMidfield
                    | position::Position::RightMidfield => 0.3,
                    position::Position::CenterMidfield => 0.2,
                    _ => 0.01,
                };
                atk += (p.height as f32
                    + p.jumping as f32
                    + p.strength as f32 * 0.9
                    + p.heading as f32 * 0.7
                    + p.attack_positioning as f32 * 0.6)
                    * multiplier;
                // front 4 with 2 strikers-max: 3428
            }
        }
        atk
    }

    /// get thread score on the flanks
    /// return left side thread, right side thread
    /// factors:
    /// - tactics: attack_width
    /// - players: technique, pace, attack_positioning, playstyle
    fn get_wide_atk(
        &self,
        team: &squad::Squad,
        players: impl Iterator<Item = &'a &'a player::Player>,
    ) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;
        for p in players {
            let mut ability =
                (p.technique as f32 + p.pace as f32 + p.attack_positioning as f32) / 3.0;
            ability *= match p.playstyle {
                style::PlayStyle::Inverted => 0.7,  // inverted fullbacks
                style::PlayStyle::TrackBack => 0.8, // wingers that trackback
                _ => 1.0,
            };
            let multiplier = match p.position {
                position::Position::LeftWing | position::Position::RightWing => 1.0,
                position::Position::LeftMidfield | position::Position::RightMidfield => 0.9,
                position::Position::LeftWingBack | position::Position::RightWingBack => 0.7,
                position::Position::LeftBack | position::Position::RightBack => 0.7,
                _ => continue,
            };
            ability *= multiplier;

            match p.position {
                position::Position::LeftWing
                | position::Position::LeftMidfield
                | position::Position::LeftWingBack
                | position::Position::LeftBack => {
                    left += ability;
                }
                position::Position::RightWing
                | position::Position::RightMidfield
                | position::Position::RightWingBack
                | position::Position::RightBack => {
                    right += ability;
                }
                _ => continue,
            }
        }
        if team.tactics.attack_width == tactics::Width::Central {
            left *= 0.7;
            right *= 0.7;
        };
        (left, right)
    }

    /// get defensive capabilities on the flanks
    fn get_wide_def(
        &self,
        team: &squad::Squad,
        players: impl Iterator<Item = &'a &'a player::Player>,
    ) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;
        for p in players {
            let mut ability = (p.tackling as f32
                + p.marking as f32
                + p.pace as f32
                + p.attack_positioning as f32)
                / 4.0;
            ability *= match p.playstyle {
                style::PlayStyle::Wide => 1.0,      // wide fullbacks
                style::PlayStyle::TrackBack => 0.7, // wingers that track back
                _ => 0.5,
            };
            let multiplier = match p.position {
                position::Position::LeftWing | position::Position::RightWing => 1.0,
                position::Position::LeftMidfield | position::Position::RightMidfield => 0.9,
                position::Position::LeftWingBack | position::Position::RightWingBack => 0.7,
                position::Position::LeftBack | position::Position::RightBack => 0.7,
                _ => continue,
            };
            ability *= multiplier;

            match p.position {
                position::Position::LeftWing
                | position::Position::LeftMidfield
                | position::Position::LeftWingBack
                | position::Position::LeftBack => {
                    left += ability;
                }
                position::Position::RightWing
                | position::Position::RightMidfield
                | position::Position::RightWingBack
                | position::Position::RightBack => {
                    right += ability;
                }
                _ => continue,
            }
        }

        let cmpt = team.tactics.compactness as f32 / u8::MAX as f32;
        left *= cmpt;
        right *= cmpt;
        (left, right)
    }
}
