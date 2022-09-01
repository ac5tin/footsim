use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use rand::{rngs::ThreadRng, thread_rng, Rng};
use strum::IntoEnumIterator;

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
                // enter action loop
                {
                    let mut prev_action = None;
                    loop {
                        // get field zone
                        let zone = self.get_fieldzone(team);
                        // get player with ball
                        let player = self.get_player_with_ball(players.clone().into_iter(), &zone);
                        if let Some((action, threat)) = self.get_player_next_action(
                            prev_action,
                            player,
                            &zone,
                            &self.home.tactics,
                        ) {
                            // has action, check if action can be executed
                            prev_action = Some((action, threat));
                            if !self.action_success(
                                &action,
                                threat,
                                player,
                                &zone,
                                opp_players.clone().into_iter(),
                            ) {
                                // action fail
                                println!(
                                    "{}: {} [zone: {},threat: {}](Fail)",
                                    player.name, action, zone, threat
                                );
                                break;
                            };
                            // action success
                            println!(
                                "{}: {} [zone:{}, threat: {}](Success)",
                                player.name, action, zone, threat
                            );
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

    fn get_defensive_player(
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
                style::PlayStyle::TrackBack => {
                    zone_prob.insert(field::FieldZone::Left, 1.3);
                    zone_prob.insert(field::FieldZone::Right, 1.3);
                }
                style::PlayStyle::BoxToBox => {
                    zone_prob.insert(field::FieldZone::Box, 1.3);
                }
                _ => {}
            };
            let base_prob = match p.position {
                position::Position::Goalkeeper => match zone {
                    field::FieldZone::Box => 1.0,
                    _ => 0.0,
                },
                position::Position::LeftBack | position::Position::LeftWingBack => match zone {
                    field::FieldZone::Right => 1.0,
                    field::FieldZone::Left => 0.1,
                    _ => 0.1,
                },
                position::Position::RightBack | position::Position::RightWingBack => match zone {
                    field::FieldZone::Left => 1.0,
                    field::FieldZone::Right => 0.1,
                    _ => 0.1,
                },
                position::Position::LeftMidfield => match zone {
                    field::FieldZone::Right => 0.7,
                    field::FieldZone::Left => 0.1,
                    _ => 0.2,
                },
                position::Position::RightMidfield => match zone {
                    field::FieldZone::Left => 0.7,
                    field::FieldZone::Right => 0.1,
                    _ => 0.2,
                },

                position::Position::LeftWing => match zone {
                    field::FieldZone::Right => 0.4,
                    field::FieldZone::Left => 0.1,
                    _ => 0.2,
                },
                position::Position::RightWing => match zone {
                    field::FieldZone::Left => 0.4,
                    field::FieldZone::Right => 0.1,
                    _ => 0.2,
                },
                position::Position::CenterBack => match zone {
                    field::FieldZone::Box => 1.0,
                    field::FieldZone::Left => 0.3,
                    field::FieldZone::Right => 0.3,
                    _ => 0.1,
                },
                position::Position::DefensiveMidfield => match zone {
                    field::FieldZone::Box => 0.6,
                    field::FieldZone::Center => 0.9,
                    _ => 0.2,
                },
                position::Position::CenterMidfield => match zone {
                    field::FieldZone::Center => 0.9,
                    _ => 0.2,
                },
                position::Position::AttackingMidfield => match zone {
                    field::FieldZone::Center => 0.7,
                    _ => 0.2,
                },
                position::Position::Striker => match zone {
                    field::FieldZone::Center => 0.5,
                    _ => 0.1,
                },
            };
            let prob = base_prob
                * zone_prob.get(zone).unwrap_or(&1.0)
                * rng.gen_range(0.8..1.2)
                * ((p.defensive_positioning as f32
                    + p.decision_making as f32 * 0.75
                    + p.stamina as f32 * 0.65)
                    / 3.0);
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
        prev_action: Option<(action::Action, f32)>,
        player: &player::Player,
        zone: &field::FieldZone,
        tactics: &tactics::Tactics,
    ) -> Option<(action::Action, f32)> {
        let mut rng = self.rng.write().unwrap();
        let mut act_map = HashMap::new();
        for a in action::Action::iter() {
            // more random actions if decision making is low / stamina is low
            let base = rng.gen_range(0.2..(1.0 / player.decision_making as f32) + 0.2);
            act_map.insert(a, base);
        }
        // position + zone
        {
            match zone {
                field::FieldZone::Left | field::FieldZone::Right => {
                    act_map.insert(action::Action::Cross, 0.9);
                    act_map.insert(action::Action::Dribble, 0.7);
                }
                field::FieldZone::Center => {
                    act_map.insert(action::Action::Pass, 1.0);
                }
                field::FieldZone::Box => {
                    act_map.insert(action::Action::Shoot, 1.0);
                }
            };
        }
        // playstyle
        {
            match player.playstyle {
                style::PlayStyle::Wide => {
                    *act_map.get_mut(&action::Action::Cross).unwrap() *= 1.3;
                }
                style::PlayStyle::Inverted => {
                    *act_map.get_mut(&action::Action::Cross).unwrap() *= 0.7;
                    *act_map.get_mut(&action::Action::Pass).unwrap() *= 1.3;
                    *act_map.get_mut(&action::Action::Dribble).unwrap() *= 1.1;
                }
                style::PlayStyle::False9 => {
                    *act_map.get_mut(&action::Action::Dribble).unwrap() *= 1.3;
                    *act_map.get_mut(&action::Action::Shoot).unwrap() *= 0.8;
                    *act_map.get_mut(&action::Action::Pass).unwrap() *= 1.1;
                }
                style::PlayStyle::CutInside => {
                    *act_map.get_mut(&action::Action::Dribble).unwrap() *= 1.3;
                    *act_map.get_mut(&action::Action::Cross).unwrap() *= 0.6;
                }
                style::PlayStyle::Playmaker | style::PlayStyle::BallPlaying => {
                    *act_map.get_mut(&action::Action::Pass).unwrap() *= 1.5;
                }
                _ => {}
            };
        }

        // tactics
        {
            if tactics.shoot_more_often {
                *act_map.get_mut(&action::Action::Shoot).unwrap() *= 1.3;
            }
            if tactics.cross_more_often {
                *act_map.get_mut(&action::Action::Cross).unwrap() *= 1.3;
            }
        }
        // prev action
        {
            match prev_action {
                Some((action::Action::Cross, _)) => {
                    *act_map.get_mut(&action::Action::Shoot).unwrap() *= 1.5;
                }
                _ => {}
            }
        }
        // highest probability
        let mut total_prob = 0.0;
        for (_, p) in act_map.iter() {
            total_prob += *p;
        }
        // find player with highest probability from list
        let mut players_prob = act_map
            .iter()
            .map(|(a, p)| (*a, *p))
            .collect::<Vec<(action::Action, f32)>>();
        players_prob.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap()); // sort list in ascending order

        let prob = rng.gen_range(0.0..total_prob);
        let mut x = 0.0;
        for (a, s) in players_prob.iter() {
            if prob <= (*s + x) {
                // get threat score
                let mut threat = match a {
                    action::Action::Shoot => {
                        (player.shooting as f32
                            + player.decision_making as f32
                            + player.attack_positioning as f32 * 0.8)
                            / 2.0
                            * *s
                    }
                    action::Action::Pass => {
                        (player.passing as f32
                            + player.decision_making as f32
                            + player.vision as f32
                            + player.creativity as f32)
                            / 4.0
                            * *s
                    }
                    action::Action::Dribble => {
                        (player.technique as f32
                            + player.decision_making as f32
                            + player.pace as f32
                            + player.strength as f32 * 0.7)
                            / 4.0
                            * *s
                    }

                    action::Action::Cross => {
                        (player.passing as f32
                            + player.decision_making as f32 * 0.7
                            + player.vision as f32)
                            / 3.0
                            * *s
                    }
                };
                // addition
                {
                    if *zone == field::FieldZone::Box
                        && prev_action.is_some()
                        && prev_action.unwrap().0 == action::Action::Cross
                    {
                        let t = (player.heading as f32
                            + player.height as f32
                            + player.strength as f32
                            + player.jumping as f32)
                            / 4.0;
                        threat += (t + threat) / 2.0;
                    }
                }
                return Some((*a, threat));
            }
            x += *s;
        }

        None
    }

    fn action_success(
        &self,
        action: &action::Action,
        threat: f32,
        player: &player::Player,
        zone: &field::FieldZone,
        opp_players: impl Iterator<Item = &'a &'a player::Player>,
    ) -> bool {
        let def_score;
        if *action == action::Action::Shoot {
            // goalkeeper
            let def_player = opp_players
                .filter(|p| p.position == position::Position::Goalkeeper)
                .next()
                .unwrap();
            def_score = (def_player.goalkeeping as f32
                + def_player.defensive_positioning as f32 * 0.7
                + def_player.height as f32 * 0.6
                + def_player.decision_making as f32 * 0.6)
                / 4.0;
        } else {
            let def_player = &self.get_defensive_player(opp_players, zone);
            def_score = match zone {
                field::FieldZone::Box => {
                    (def_player.defensive_positioning as f32
                        + def_player.height as f32 * 0.7
                        + def_player.strength as f32 * 0.8
                        + def_player.jumping as f32 * 0.75
                        + def_player.heading as f32 * 0.85)
                        / 5.0
                }
                field::FieldZone::Left | field::FieldZone::Right => {
                    (def_player.defensive_positioning as f32 * 0.9
                        + def_player.pace as f32 * 0.8
                        + def_player.tackling as f32 * 0.9
                        + def_player.decision_making as f32 * 0.7
                        + def_player.stamina as f32 * 0.6)
                        / 5.0
                }
                field::FieldZone::Center => {
                    (def_player.defensive_positioning as f32 * 1.0
                        + def_player.pace as f32 * 0.6
                        + def_player.tackling as f32 * 0.9
                        + def_player.decision_making as f32 * 0.8
                        + def_player.stamina as f32 * 0.7)
                        / 5.0
                }
            }
        };

        let mut rng = self.rng.write().unwrap();
        if rng.gen_bool(threat as f64 / (threat + def_score) as f64) {
            // action success
            return true;
        };
        false
    }

    fn get_event_from_action(&self, action: &action::Action) -> Option<event::Event> {
        None
    }
}
