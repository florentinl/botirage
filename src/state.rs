use std::collections::HashMap;

use teloxide::types::{DiceEmoji, UserId};

const DEFAULT_MONEY: i64 = 100;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum State {
    Idle {
        player_money: HashMap<UserId, i64>,
        game_stats: HashMap<DiceEmoji, HashMap<u8, u64>>,
    },
    ReceivingPollAnswers {
        player_money: HashMap<UserId, i64>,
        game_stats: HashMap<DiceEmoji, HashMap<u8, u64>>,
    },
}

impl Default for State {
    fn default() -> Self {
        let mut game_stats = HashMap::default();
        game_stats.insert(DiceEmoji::Dice, HashMap::default());
        game_stats.insert(DiceEmoji::Darts, HashMap::default());
        game_stats.insert(DiceEmoji::Basketball, HashMap::default());
        game_stats.insert(DiceEmoji::Football, HashMap::default());
        game_stats.insert(DiceEmoji::Bowling, HashMap::default());
        game_stats.insert(DiceEmoji::SlotMachine, HashMap::default());

        Self::Idle {
            player_money: HashMap::default(),
            game_stats,
        }
    }
}

impl State {
    fn player_money(&self) -> &HashMap<UserId, i64> {
        match self {
            Self::Idle {
                player_money,
                game_stats: _,
            } => player_money,
            Self::ReceivingPollAnswers {
                player_money,
                game_stats: _,
            } => player_money,
        }
    }

    fn game_stats(&self) -> &HashMap<DiceEmoji, HashMap<u8, u64>> {
        match self {
            Self::Idle {
                player_money: _,
                game_stats,
            } => game_stats,
            Self::ReceivingPollAnswers {
                player_money: _,
                game_stats,
            } => game_stats,
        }
    }

    fn player_money_mut(&mut self) -> &mut HashMap<UserId, i64> {
        match self {
            Self::Idle {
                player_money,
                game_stats: _,
            } => player_money,
            Self::ReceivingPollAnswers {
                player_money,
                game_stats: _,
            } => player_money,
        }
    }

    fn game_stats_mut(&mut self) -> &mut HashMap<DiceEmoji, HashMap<u8, u64>> {
        match self {
            Self::Idle {
                player_money: _,
                game_stats,
            } => game_stats,
            Self::ReceivingPollAnswers {
                player_money: _,
                game_stats,
            } => game_stats,
        }
    }

    pub(crate) fn register_game_result(&mut self, dice: DiceEmoji, dice_value: u8) {
        let game_stats = self.game_stats_mut();
        let game_stat = game_stats.entry(dice).or_default();
        *game_stat.entry(dice_value).or_default() += 1;
    }

    pub(crate) fn leaderboard(&self) -> Vec<(UserId, i64)> {
        let mut leaderboard = self.player_money().iter().collect::<Vec<_>>();
        leaderboard.sort_by_key(|&(_, &money)| -money);
        leaderboard
            .iter()
            .map(|&(&player, &money)| (player, money))
            .collect()
    }

    pub(crate) fn get(&self, player: &UserId) -> i64 {
        self.player_money()
            .get(player)
            .copied()
            .unwrap_or(DEFAULT_MONEY)
    }

    pub(crate) fn insert(&mut self, player: &UserId, delta_money: i64) {
        let player_money = self.player_money_mut();
        player_money.insert(
            *player,
            player_money.get(&player).unwrap_or(&DEFAULT_MONEY) + delta_money,
        );
    }

    pub(crate) fn to_idle(&self) -> Self {
        Self::Idle {
            player_money: self.player_money().clone(),
            game_stats: self.game_stats().clone(),
        }
    }

    pub(crate) fn to_receiving_poll_answers(&self) -> Self {
        Self::ReceivingPollAnswers {
            player_money: self.player_money().clone(),
            game_stats: self.game_stats().clone(),
        }
    }
}
