use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};

use log::info;
use teloxide::dispatching::dialogue::{self, InMemStorage};
use teloxide::dispatching::UpdateHandler;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

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
                .branch(case![Command::Roll].endpoint(start_poll))
                .branch(dptree::endpoint(invalid_state)),
        );

    let message_handler = Update::filter_message().branch(command_handler);

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

async fn start_poll(
    bot: Bot,
    dialogue: Dialogue<State, InMemStorage<State>>,
    poll_answers: Arc<Mutex<HashMap<UserId, u8>>>,
    msg: Message,
) -> HandlerResult {
    let poll = bot
        .send_poll(msg.chat.id, "Roll a dice!", (1..=6).map(|x| x.to_string()))
        .is_anonymous(false)
        .await?;
    poll_answers.lock().unwrap().clear();
    dialogue.update(State::ReceivingPollAnswers).await?;

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        bot.stop_poll(msg.chat.id, poll.id).await.unwrap();
        bot.send_message(msg.chat.id, "Poll closed. Rolling the dice...")
            .await
            .unwrap();
        let dice = bot.send_dice(msg.chat.id).await.unwrap();
        let dice_value = dice.dice().unwrap().value;

        let winner_ids = get_winners(poll_answers.lock().unwrap().to_owned(), dice_value);

        let mut winners = vec![];
        for id in winner_ids.into_iter() {
            winners.push(
                bot.get_chat_member(msg.chat.id, id)
                    .await
                    .unwrap()
                    .user
                    .username
                    .unwrap(),
            );
        }

        let winners = winners;

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        bot.send_message(
            msg.chat.id,
            format!(
                "The dice rolled: {}\nWinners: {}",
                dice_value,
                winners.join(", ")
            ),
        )
        .await
        .unwrap();

        dialogue.update(State::Idle).await.unwrap();

        dialogue
    });

    Ok(())
}

fn get_winners(poll_answers: HashMap<UserId, u8>, dice_value: u8) -> Vec<UserId> {
    poll_answers
        .iter()
        .filter_map(|(user_id, answer)| {
            if *answer == dice_value {
                Some(*user_id)
            } else {
                None
            }
        })
        .collect()
}

async fn register_answer(
    _bot: Bot,
    poll_answers: Arc<Mutex<HashMap<UserId, u8>>>,
    pa: PollAnswer,
) -> HandlerResult {
    poll_answers.lock().unwrap().insert(
        pa.voter.user().unwrap().id,
        pa.option_ids.first().unwrap() + 1,
    );
    Ok(())
}

async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Commande indisponible")
        .await?;

    Ok(())
}
