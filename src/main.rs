use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};

use commands::{balance, help, leaderboard};
use emoji_games::{emoji_games_handler, stats_handler};
use log::info;
use loto::{register_answer, reset_roll, start_loto};
use state::State;
use teloxide::adaptors::throttle::Limits;
use teloxide::dispatching::dialogue::serializer::Json;
use teloxide::dispatching::dialogue::{self, ErasedStorage, SqliteStorage, Storage};
use teloxide::dispatching::UpdateHandler;
use teloxide::prelude::*;
use teloxide::types::MessageKind;
use teloxide::utils::command::BotCommands;
use utils::{BotType, DialogueType, HandlerResult};

mod commands;
mod emoji_games;
mod loto;
mod state;
mod utils;

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
    #[command(description = "Réinitialise la loterie", hide)]
    ResetRoll,
    #[command(description = "Regarde ton solde")]
    Balance,
    #[command(description = "Classement des gens les plus riches")]
    Leaderboard,
    #[command(description = "Affiche les stats pour un jeu donné", hide)]
    Stats { emoji: String },
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    info!("Starting bot...");
    let bot = Bot::from_env().throttle(Limits::default());
    bot.set_my_commands(Command::bot_commands()).await.unwrap();
    let poll_answers: Arc<Mutex<HashMap<UserId, u8>>> = Arc::new(Mutex::new(HashMap::default()));

    let path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "./database.db".to_string());

    let storage: Arc<ErasedStorage<State>> =
        SqliteStorage::open(&path, Json).await.unwrap().erase();

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
        .branch(case![Command::Leaderboard].endpoint(leaderboard))
        .branch(case![Command::ResetRoll].endpoint(reset_roll))
        .branch(case![Command::Stats { emoji }].endpoint(stats_handler))
        .branch(
            case![State::Idle {
                player_money,
                game_stats
            }]
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

async fn message_handler(bot: BotType, dialogue: DialogueType, msg: Message) -> HandlerResult {
    if let MessageKind::Dice(_) = msg.kind.clone() {
        emoji_games_handler(bot, dialogue, msg).await?;
    }

    Ok(())
}

async fn invalid_state(bot: BotType, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Commande indisponible")
        .await?;

    Ok(())
}
