use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};

use emoji_games::slot_machine_handler;
use log::info;
use loto::{register_answer, start_loto};
use teloxide::dispatching::dialogue::{self, InMemStorage};
use teloxide::dispatching::UpdateHandler;
use teloxide::prelude::*;
use teloxide::types::{Dice, DiceEmoji, MessageDice, MessageKind};
use teloxide::utils::command::BotCommands;
use utils::HandlerResult;

mod emoji_games;
mod loto;
mod utils;

#[derive(Clone, Default, Debug)]
enum State {
    #[default]
    Idle,
    ReceivingPollAnswers,
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "roll a dice.")]
    Roll,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    info!("Starting bot...");
    let bot = Bot::from_env();
    let poll_answers: Arc<Mutex<HashMap<UserId, u8>>> = Arc::new(Mutex::new(HashMap::default()));

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new(), poll_answers])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<Box<dyn Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Help].endpoint(help))
        .branch(
            case![State::Idle]
                .branch(case![Command::Roll].endpoint(start_loto))
                .branch(dptree::endpoint(invalid_state)),
        );

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(dptree::endpoint(message_handler));

    let poll_handler = Update::filter_poll_answer().endpoint(register_answer);

    dptree::entry()
        .branch(poll_handler)
        .branch(dialogue::enter::<Update, InMemStorage<State>, State, _>().branch(message_handler))
}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn message_handler(bot: Bot, msg: Message) -> HandlerResult {
    if let MessageKind::Dice(MessageDice { dice: dice_message }) = msg.kind.clone() {
        match dice_message {
            Dice {
                emoji: DiceEmoji::SlotMachine,
                value,
            } => slot_machine_handler(bot, msg, value).await?,
            _ => todo!(),
        }
    }

    Ok(())
}

async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Commande indisponible")
        .await?;

    Ok(())
}
