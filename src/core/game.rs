use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use rand::{rngs::ThreadRng, thread_rng, Rng};

use super::action;
use super::event;
use super::field;
use super::player;
use super::tactics;
use super::{position, squad, style};

pub struct Game<'a> {
    home: squad::Squad<'a>,
    away: squad::Squad<'a>,
    home_stats: GameStats,
    away_stats: GameStats,
    home_actions: Vec<(&'a player::Player, action::Action)>,
    away_actions: Vec<(&'a player::Player, action::Action)>,
    events: Vec<event::Event>,
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
            home_actions: vec![],
            away_actions: vec![],
            events: vec![],
            rng: Arc::new(RwLock::new(thread_rng())),
        }
    }

    pub fn play(&mut self) {
        self.play_half();
        self.halftime();
        self.play_half();
    }

    pub fn get_home_stats(&self) -> GameStats {
        self.home_stats.to_owned()
    }
    pub fn get_away_stats(&self) -> GameStats {
        self.away_stats.to_owned()
    }

    fn halftime(&mut self) {}

    fn play_half(&mut self) {
        let mut events = vec![];
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

        // tick game
        {
            let mut home_touches = 0;
            let mut away_touches = 0;
            for _ in 0..45 {
                // manager make tactics change/subs
                // get team with possession
                let (home_poss, _) = self.get_poss(
                    &self.home,
                    home_players.clone().into_iter(),
                    &self.away,
                    away_players.clone().into_iter(),
                );

                let team;
                let players;
                let opp_players;
                if self.rng.write().unwrap().gen_bool(home_poss as f64 * 0.01) {
                    team = &self.home;
                    players = home_players.clone().into_iter();
                    opp_players = away_players.clone().into_iter();
                    home_touches += 1;
                } else {
                    team = &self.away;
                    players = away_players.clone().into_iter();
                    opp_players = home_players.clone().into_iter();
                    away_touches += 1;
                }
                // get field zone
                let zone = self.get_fieldzone(team);
                // get player with ball
                let player = self.get_player_with_ball(players.clone().into_iter(), &zone);
                // enter action loop
                {
                    let mut prev_action = None;
                    loop {
                        if let Some(action) = self.get_player_next_action(
                            prev_action,
                            player,
                            &zone,
                            &self.home.tactics,
                            players.clone().into_iter(),
                        ) {
                            // has action, check if action can be executed
                            prev_action = Some(action);
                            if !self.action_success(
                                &action,
                                player,
                                opp_players.clone().into_iter(),
                            ) {
                                // action fail
                                break;
                            };
                            // action success
                            if let Some(ev) = self.get_event_from_action(&action) {
                                // has event
                                events.push(ev);
                            };
                        } else {
                            // no actions
                            break;
                        }
                    }
                }
                // random event
            }
            // update possession
            {
                home_stats.possession = home_touches as f32 / (home_touches + away_touches) as f32;
                away_stats.possession = 1.0 - home_stats.possession;
            }
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
            // events
            self.events.extend(events);
        }
    }

    /// get an event
    fn get_event(&self) -> Option<event::Event> {
        None
    }

    fn get_action(&self) -> Option<action::Action> {
        None
    }

    fn get_players(
        &self,
        team: &'a squad::Squad,
        stats: GameStats,
    ) -> impl Iterator<Item = &&player::Player> {
        team.players
            .iter()
            .filter(move |p| !stats.red_cards.contains(&p.id))
    }

    fn get_poss(
        &self,
        a_team: &squad::Squad,
        a_players: impl Iterator<Item = &'a &'a player::Player>,
        b_team: &squad::Squad,
        b_players: impl Iterator<Item = &'a &'a player::Player>,
    ) -> (f32, f32) {
        let home_score = self.get_team_poss_score(a_team, a_players);
        let away_score = self.get_team_poss_score(b_team, b_players);

        let home_poss = (home_score as f32 / (home_score + away_score)) * 100.0;
        (home_poss, 100.0 - home_poss)
    }

    /// get team posession score
    /// based on: recycling possession, individual technique ,tactical understanding ,movements, ball retention, pressure
    fn get_team_poss_score(
        &self,
        team: &squad::Squad,
        players: impl Iterator<Item = &'a &'a player::Player>,
    ) -> f32 {
        let mut recyc = 0.0;
        let mut tact = 0.0;
        let mut mov = 0.0;
        let mut ret = 0.0;
        let mut press = 0.0;
        let mut tech = 0.0;
        let mut total_tact_und = 0.0f32;
        let mut total_players = 0u8;
        for p in players {
            let player_tact_und = (p.tactical as f32 + p.decision_making as f32 * 0.7) / 2.0;
            total_tact_und += player_tact_und;
            recyc += (p.passing as f32 + p.decision_making as f32 * 0.8) / 2.0;
            tact += player_tact_und;
            mov += (p.attack_positioning as f32
                + p.tactical as f32 * 0.8
                + p.decision_making as f32 * 0.7
                + p.stamina as f32 * 0.7)
                / 4.0;
            ret += (p.technique as f32 * 0.9
                + p.fitness as f32 * 0.6
                + p.strength as f32 * 0.5
                + p.vision as f32 * 0.7
                + p.decision_making as f32 * 0.8)
                / 5.0;
            press += (p.stamina as f32 * 0.9
                + p.defensive_positioning as f32 * 0.9
                + p.pace as f32 * 0.8
                + p.tackling as f32 * 0.8)
                / 4.0;
            tech += (p.technique as f32 + p.passing as f32 + p.stamina as f32) / 3.0;
            total_players += 1;
        }
        // tactics: higher def_line + higher aggression + lower compactness = higher press
        let team_press = (team.tactics.defense_line as f32 + team.tactics.aggression as f32 * 0.9)
            / (team.tactics.compactness as f32 * 0.8)
            / 3.0;
        // tactics: lower buildup + shorter pass range + shoot less often = higher possession
        let mut team_poss =
            (1.0 / (team.tactics.build_up_speed as f32 + team.tactics.pass_range as f32)) * 500.0;
        if team.tactics.shoot_more_often {
            team_poss *= 0.8;
        }

        let team_tactics = (team_press + team_poss)
            * ((team.manager.tactical as f32 + total_tact_und / total_players as f32) / 2.0);

        recyc + tact + mov + ret + press + tech + team_tactics
    }

    fn get_fieldzone(&self, team: &squad::Squad) -> field::FieldZone {
        // central: 20,50,20->10  , balanced: 30,30,30->10,  left: 50,30,10 -> 10  right: 10,30,50 -> 10
        // central: 1,3,8,10 ,  balanced: 1,4,7,10    left: 1,6,9,10, right:   1,2,5,10
        let mut rng = self.rng.write().unwrap();
        let rnd = rng.gen_range(1u8..10);

        let rate: [u8; 4];
        match team.tactics.attack_width {
            tactics::Width::Left => rate = [1, 6, 9, 10],
            tactics::Width::Central => rate = [1, 3, 8, 10],
            tactics::Width::Right => rate = [1, 2, 5, 10],
            tactics::Width::Balanced => rate = [1, 4, 7, 10],
        }

        let zone_map = [
            field::FieldZone::Box,
            field::FieldZone::Left,
            field::FieldZone::Center,
            field::FieldZone::Right,
        ];

        for (i, r) in rate.iter().enumerate() {
            if rnd <= *r {
                return zone_map[i];
            }
        }
        field::FieldZone::Center
    }

    fn get_player_with_ball(
        &self,
        players: impl Iterator<Item = &'a &'a player::Player>,
        zone: &field::FieldZone,
    ) -> &'a player::Player {
        let mut rng = self.rng.write().unwrap();
        let mut players_prob = vec![];
        let mut total_prob = 0.0;
        for &p in players {
            let mut zone_prob = HashMap::new();
            match p.playstyle {
                style::PlayStyle::False9 => {
                    zone_prob.insert(field::FieldZone::Box, 0.7);
                    zone_prob.insert(field::FieldZone::Center, 1.3);
                }
                style::PlayStyle::Inverted => {
                    zone_prob.insert(field::FieldZone::Left, 0.7);
                    zone_prob.insert(field::FieldZone::Right, 0.7);
                    zone_prob.insert(field::FieldZone::Center, 1.3);
                }
                style::PlayStyle::Wide => {
                    zone_prob.insert(field::FieldZone::Left, 1.3);
                    zone_prob.insert(field::FieldZone::Right, 1.3);
                    zone_prob.insert(field::FieldZone::Center, 0.7);
                }
                style::PlayStyle::BoxToBox => {
                    zone_prob.insert(field::FieldZone::Center, 1.3);
                    zone_prob.insert(field::FieldZone::Box, 1.3);
                }
                _ => {}
            }
            let base_prob = match p.position {
                position::Position::Goalkeeper => {
                    // goalkeeper will only operate in center
                    match zone {
                        field::FieldZone::Center => 0.1,
                        _ => 0.0,
                    }
                }
                position::Position::CenterBack => match zone {
                    field::FieldZone::Center => 0.2,
                    _ => 0.1,
                },
                position::Position::LeftBack | position::Position::LeftWingBack => match zone {
                    field::FieldZone::Left => 0.7,
                    field::FieldZone::Right => 0.01,
                    _ => 0.2,
                },
                position::Position::RightBack | position::Position::RightWingBack => match zone {
                    field::FieldZone::Right => 0.7,
                    field::FieldZone::Left => 0.01,
                    _ => 0.2,
                },

                position::Position::LeftMidfield => match zone {
                    field::FieldZone::Left => 0.9,
                    field::FieldZone::Right => 0.01,
                    _ => 0.2,
                },
                position::Position::RightMidfield => match zone {
                    field::FieldZone::Left => 0.01,
                    field::FieldZone::Right => 0.9,
                    _ => 0.2,
                },

                position::Position::DefensiveMidfield | position::Position::CenterMidfield => {
                    match zone {
                        field::FieldZone::Center => 0.9,
                        _ => 0.2,
                    }
                }
                position::Position::AttackingMidfield => match zone {
                    field::FieldZone::Center => 0.9,
                    field::FieldZone::Box => 0.5,
                    _ => 0.2,
                },

                position::Position::LeftWing => match zone {
                    field::FieldZone::Left => 0.7,
                    field::FieldZone::Box => 0.5,
                    field::FieldZone::Right => 0.01,
                    _ => 0.2,
                },
                position::Position::RightWing => match zone {
                    field::FieldZone::Left => 0.01,
                    field::FieldZone::Box => 0.5,
                    field::FieldZone::Right => 0.7,
                    _ => 0.2,
                },

                position::Position::Striker => match zone {
                    field::FieldZone::Box => 0.9,
                    _ => 0.2,
                },
            };
            let prob = base_prob
                * zone_prob.get(zone).unwrap_or(&1.0)
                * rng.gen_range(0.8..1.2)
                * (p.attack_positioning as f32 * 0.9
                    + p.defensive_positioning as f32 * 0.8
                    + p.tactical as f32 * 0.8
                    + p.decision_making as f32 * 0.8
                    + p.stamina as f32 * 0.5
                    + p.technique as f32 * 0.7
                    + p.passing as f32 * 0.7
                    + p.vision as f32 * 0.6
                    + p.strength as f32 * 0.5
                    + p.pace as f32 * 0.4
                    + p.tackling as f32 * 0.4)
                / 11.0;
            players_prob.push((p, prob));
            total_prob += prob;
        }

        // find player with highest probability from list
        players_prob.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap()); // sort list in ascending order
        let rnd = rng.gen_range(0.0..total_prob);

        let mut x = 0.0;
        let mut ret_player = players_prob[0].0;
        for (p, s) in players_prob.iter() {
            if rnd <= (*s + x) {
                ret_player = p;
                break;
            }
            x += *s;
        }

        ret_player
    }

    fn get_player_next_action(
        &self,
        prev_action: Option<action::Action>,
        player: &player::Player,
        zone: &field::FieldZone,
        tactics: &tactics::Tactics,
        players: impl Iterator<Item = &'a &'a player::Player>,
    ) -> Option<action::Action> {
        None
    }

    fn action_success(
        &self,
        action: &action::Action,
        player: &player::Player,
        opp_players: impl Iterator<Item = &'a &'a player::Player>,
    ) -> bool {
        false
    }

    fn get_event_from_action(&self, action: &action::Action) -> Option<event::Event> {
        None
    }
}
