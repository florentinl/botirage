use std::collections::HashMap;

use teloxide::types::UserId;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum State {
    Idle { player_money: HashMap<UserId, i64> },
    ReceivingPollAnswers { player_money: HashMap<UserId, i64> },
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
            Self::ReceivingPollAnswers { player_money } => player_money,
        }
    }

    fn player_money_mut(&mut self) -> &mut HashMap<UserId, i64> {
        match self {
            Self::Idle { player_money } => player_money,
            Self::ReceivingPollAnswers { player_money } => player_money,
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

    pub(crate) fn get(&self, player: &UserId) -> i64 {
        self.player_money().get(player).copied().unwrap_or(1000)
    }

    pub(crate) fn insert(&mut self, player: &UserId, delta_money: i64) {
        let player_money = self.player_money_mut();
        player_money.insert(
            *player,
            player_money.get(&player).unwrap_or(&1000) + delta_money,
        );
    }

    pub(crate) fn to_idle(&self) -> Self {
        Self::Idle {
            player_money: self.player_money().clone(),
        }
    }
}
