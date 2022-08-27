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
    possession: f32,
    shots: u8,
    shots_on_target: u8,
    goals: u8,
    freekicks: u8,
    penalties: u8,
    corners: u8,
    fouls: u8,
    yellow_cards: Vec<u32>,
    red_cards: Vec<u32>,
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
        let home_players = self.get_players(&self.home, home_stats.clone());
        let away_players = self.get_players(&self.away, away_stats.clone());
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
        // get squad strength
        let home_def = self.get_squad_def_strength(&self.home, &home_stats);
        let away_def = self.get_squad_def_strength(&self.away, &away_stats);

        // modify posession based on red cards
        // based on possession and tactics calculate shots
        let home_shots = self.get_shots(&self.home, &home_stats, away_def);
        let away_shots = self.get_shots(&self.away, &away_stats, home_def);
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
        // based on shots on target calculate goals
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
            let mut rng = self.rng.write().unwrap();
            if rng.gen_bool((player_foul * 0.001) as f64) {
                yellow_cards.push(player.id);
            };
            // red card rate
            let mut rng = self.rng.write().unwrap();
            if rng.gen_bool((player_foul * 0.0005) as f64) {
                red_cards.push(player.id);
            };
        }
        fouls *= stats.possession;
        (fouls.round() as u8, yellow_cards, red_cards)
    }

    /// get number of shots for the team
    /// calculated based on:
    /// - tactics: shoot_more_often
    /// - player creativity, passing,technique
    fn get_shots(&self, team: &squad::Squad, stats: &GameStats, opp_def_str: f32) -> u8 {
        let mut shots: f32 = 0.0;
        for &player in team
            .players
            .iter()
            .filter(|p| !stats.red_cards.contains(&p.id))
        {
            shots += player.creativity as f32 * 0.4;
            shots += player.passing as f32 * 0.15;
            shots += player.technique as f32 * 0.15;
        }

        shots /= opp_def_str;

        if team.tactics.shoot_more_often {
            shots *= 1.5;
        }

        shots *= self.rng.write().unwrap().gen_range(0.5..1.3) * 10.0;

        shots.round() as u8
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

        let freekicks: u8 = rng.gen_range(opp_stats.yellow_cards.len() as u8..opp_stats.fouls);

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
}
