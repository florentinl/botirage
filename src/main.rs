use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};

use emoji_games::emoji_games_handler;
use log::info;
use loto::{register_answer, start_loto};
use teloxide::dispatching::dialogue::serializer::Json;
use teloxide::dispatching::dialogue::{self, ErasedStorage, SqliteStorage, Storage};
use teloxide::dispatching::UpdateHandler;
use teloxide::prelude::*;
use teloxide::types::{MessageKind, ReactionType, ReplyParameters};
use teloxide::utils::command::BotCommands;
use utils::{get_username, HandlerResult};

mod emoji_games;
mod loto;
mod utils;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
enum State {
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

    fn get(&self, player: &UserId) -> i64 {
        self.player_money().get(player).copied().unwrap_or(1000)
    }

    fn insert(&mut self, player: &UserId, delta_money: i64) {
        let player_money = self.player_money_mut();
        player_money.insert(
            *player,
            player_money.get(&player).unwrap_or(&1000) + delta_money,
        );
    }
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Les commandes suivantes sont disponibles:"
)]
enum Command {
    #[command(description = "Affiche ce texte")]
    Help,
    #[command(description = "(bêta) Lance une loterie")]
    Roll,
    #[command(description = "Regarde ton solde")]
    Balance,
    #[command(description = "(dev) free money")]
    Money,
    #[command(description = "Classement des gens les plus riches")]
    Leaderboard,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    info!("Starting bot...");
    let bot = Bot::from_env();
    bot.set_my_commands(Command::bot_commands()).await.unwrap();
    let poll_answers: Arc<Mutex<HashMap<UserId, u8>>> = Arc::new(Mutex::new(HashMap::default()));

    let storage: Arc<ErasedStorage<State>> = SqliteStorage::open("database.db", Json)
        .await
        .unwrap()
        .erase();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![storage, poll_answers])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<Box<dyn Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Help].endpoint(help))
        .branch(case![Command::Balance].endpoint(balance))
        .branch(case![Command::Money].endpoint(money))
        .branch(case![Command::Leaderboard].endpoint(leaderboard))
        .branch(
            case![State::Idle { player_money }]
                .branch(case![Command::Roll].endpoint(start_loto))
                .branch(dptree::endpoint(invalid_state)),
        );

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(dptree::endpoint(message_handler));

    let poll_handler = Update::filter_poll_answer().endpoint(register_answer);

    dptree::entry()
        .branch(poll_handler)
        .branch(dialogue::enter::<Update, ErasedStorage<State>, State, _>().branch(message_handler))
}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn balance(
    bot: Bot,
    dialogue: Dialogue<State, ErasedStorage<State>>,
    msg: Message,
) -> HandlerResult {
    let player = msg.clone().from.unwrap().id;
    let state = dialogue.get().await?.unwrap();
    let player_money = state.player_money().get(&player).unwrap_or(&1000);
    bot.send_message(
        msg.chat.id,
        format!(
            "@{}, tu as {}💵!",
            msg.from.unwrap().username.unwrap(),
            player_money
        ),
    )
    .reply_parameters(ReplyParameters::new(msg.id))
    .await?;
    Ok(())
}

async fn money(
    bot: Bot,
    dialogue: Dialogue<State, ErasedStorage<State>>,
    msg: Message,
) -> HandlerResult {
    let player = msg.from.unwrap().id;
    let mut state = dialogue.get().await?.unwrap();
    let player_money_mut = state.player_money_mut();
    player_money_mut.insert(
        player,
        player_money_mut.get(&player).unwrap_or(&1000) + 1000,
    );
    dialogue.update(state).await?;

    bot.set_message_reaction(msg.chat.id, msg.id)
        .reaction(vec![ReactionType::Emoji {
            emoji: "🍾".into()
        }])
        .await?;

    Ok(())
}

async fn leaderboard(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue<State, ErasedStorage<State>>,
) -> HandlerResult {
    let state = dialogue.get().await?.unwrap();
    let player_money = state.player_money();
    let mut leaderboard: Vec<_> = player_money.iter().collect();
    leaderboard.sort_by_key(|(_, &money)| -money);
    let mut message = "Classement ForbeCS:\n".to_owned();
    for &(user_id, money) in leaderboard.iter().take(10) {
        message.push_str(&format!(
            "{}: {}💵\n",
            get_username(&bot, msg.chat.id, user_id).await,
            money
        ));
    }

    bot.send_message(msg.chat.id, message).await?;
    Ok(())
}

async fn message_handler(
    bot: Bot,
    dialogue: Dialogue<State, ErasedStorage<State>>,
    msg: Message,
) -> HandlerResult {
    if let MessageKind::Dice(_) = msg.kind.clone() {
        emoji_games_handler(bot, dialogue, msg).await?;
    }

    Ok(())
}

async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Commande indisponible")
        .await?;

    Ok(())
}
