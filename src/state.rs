use std::collections::HashMap;

use teloxide::types::{Message, UserId};

const DEFAULT_MONEY: i64 = 100;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum State {
    Idle {
        player_money: HashMap<UserId, i64>,
    },
    ReceivingPollAnswers {
        poll: Message,
        player_money: HashMap<UserId, i64>,
    },
}

impl Default for State {
    fn default() -> Self {
        Self::Idle {
            player_money: HashMap::default(),
        }
    }
}

impl State {
    fn player_money(&self) -> &HashMap<UserId, i64> {
        match self {
            Self::Idle { player_money } => player_money,
            Self::ReceivingPollAnswers {
                poll: _,
                player_money,
            } => player_money,
        }
    }

    fn player_money_mut(&mut self) -> &mut HashMap<UserId, i64> {
        match self {
            Self::Idle { player_money } => player_money,
            Self::ReceivingPollAnswers {
                poll: _,
                player_money,
            } => player_money,
        }
    }

    pub(crate) fn leaderboard(&self) -> Vec<(UserId, i64)> {
        let mut leaderboard = self.player_money().iter().collect::<Vec<_>>();
        leaderboard.sort_by_key(|&(_, &money)| -money);
        leaderboard
            .iter()
            .map(|&(&player, &money)| (player, money))
            .collect()
    }

    pub(crate) fn get(&self, player: &UserId) -> &i64 {
        self.player_money().get(player).unwrap_or(&DEFAULT_MONEY)
    }

    pub(crate) fn insert(&mut self, player: &UserId, delta_money: i64) {
        let player_money = self.player_money_mut();
        player_money.insert(
            *player,
            player_money.get(&player).unwrap_or(&DEFAULT_MONEY) + delta_money,
        );
    }

    pub(crate) fn to_idle(self) -> Self {
        match self {
            Self::Idle { .. } => self,
            Self::ReceivingPollAnswers {
                poll: _,
                player_money,
            } => Self::Idle { player_money },
        }
    }

    pub(crate) fn to_receiving_poll_answers(self, poll: Message) -> Self {
        match self {
            Self::ReceivingPollAnswers { .. } => self,
            Self::Idle { player_money } => Self::ReceivingPollAnswers { poll, player_money },
        }
    }
}
