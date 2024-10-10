use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use teloxide::{
    dispatching::dialogue::InMemStorage,
    payloads::SendPollSetters,
    prelude::Dialogue,
    requests::Requester,
    types::{ChatId, Message, PollAnswer, UserId},
    Bot,
};

use crate::{utils::HandlerResult, State};

pub(crate) async fn start_loto(
    bot: Bot,
    dialogue: Dialogue<State, InMemStorage<State>>,
    poll_answers: Arc<Mutex<HashMap<UserId, u8>>>,
    msg: Message,
) -> HandlerResult {
    poll_answers.lock().unwrap().clear();
    let poll = bot
        .send_poll(msg.chat.id, "Roll a dice!", (1..=6).map(|x| x.to_string()))
        .is_anonymous(false)
        .await?;

    dialogue.update(State::ReceivingPollAnswers).await?;

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let _ = draw_loto(bot, dialogue, msg, poll, poll_answers).await;
    });

    Ok(())
}

async fn draw_loto(
    bot: Bot,
    dialogue: Dialogue<State, InMemStorage<State>>,
    msg: Message,
    poll: Message,
    poll_answers: Arc<Mutex<HashMap<UserId, u8>>>,
) -> HandlerResult {
    bot.stop_poll(msg.chat.id, poll.id).await.unwrap();
    bot.send_message(msg.chat.id, "Poll closed. Rolling the dice...")
        .await
        .unwrap();
    let dice = bot.send_dice(msg.chat.id).await.unwrap();
    let dice_value = dice.dice().unwrap().value;

    let winner_ids = get_winner_ids(poll_answers.lock().unwrap().to_owned(), dice_value);
    let winners = get_winner_handles(&bot, msg.chat.id, winner_ids).await;

    tokio::time::sleep(std::time::Duration::from_secs(4)).await;

    match &*winners {
        [] => {
            bot.send_message(msg.chat.id, "And the winners are 🥁🥁🥁...  No one 😢")
                .await?
        }

        _ => {
            bot.send_message(
                msg.chat.id,
                "And the winners are 🥁🥁🥁... ".to_owned() + &winners.join(", "),
            )
            .await?
        }
    };

    dialogue.update(State::Idle).await.unwrap();

    Ok(())
}

fn get_winner_ids(poll_answers: HashMap<UserId, u8>, dice_value: u8) -> Vec<UserId> {
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

async fn get_winner_handles(bot: &Bot, chat_id: ChatId, ids: Vec<UserId>) -> Vec<String> {
    let mut winners = vec![];
    for id in ids.into_iter() {
        winners.push(
            bot.get_chat_member(chat_id, id)
                .await
                .unwrap()
                .user
                .username
                .unwrap(),
        );
    }
    winners
}

pub(crate) async fn register_answer(
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
