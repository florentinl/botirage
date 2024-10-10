use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use teloxide::{
    payloads::SendPollSetters,
    requests::Requester,
    types::{Message, PollAnswer, UserId},
};

use crate::utils::{get_usernames, BotType, DialogueType, HandlerResult};

pub(crate) async fn start_loto(
    bot: BotType,
    dialogue: DialogueType,
    poll_answers: Arc<Mutex<HashMap<UserId, u8>>>,
    msg: Message,
) -> HandlerResult {
    poll_answers.lock().unwrap().clear();

    let state = dialogue.get().await?.unwrap();
    dialogue.update(state.to_receiving_poll_answers()).await?;

    let poll = bot
        .send_poll(
            msg.chat.id,
            "Placez vos paris!",
            (1..=6).map(|x| x.to_string()),
        )
        .is_anonymous(false)
        .await?;

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        let _ = draw_loto(bot, dialogue, msg, poll, poll_answers).await;
    });

    Ok(())
}

async fn draw_loto(
    bot: BotType,
    dialogue: DialogueType,
    msg: Message,
    poll: Message,
    poll_answers: Arc<Mutex<HashMap<UserId, u8>>>,
) -> HandlerResult {
    bot.stop_poll(msg.chat.id, poll.id).await.unwrap();
    bot.send_message(
        msg.chat.id,
        "Les paris sont fermés. C'est l'heure du lancé...",
    )
    .await
    .unwrap();
    let dice = bot.send_dice(msg.chat.id).await.unwrap();
    let dice_value = dice.dice().unwrap().value;

    let winner_ids = get_winner_ids(&poll_answers.lock().unwrap().to_owned(), dice_value);
    let winners = get_usernames(&bot, &msg.chat.id, &winner_ids).await;

    tokio::time::sleep(std::time::Duration::from_secs(4)).await;

    let mut state = dialogue.get().await?.unwrap();
    for winner_id in &winner_ids {
        state.insert(winner_id, 500);
    }
    dialogue.update(state.to_idle()).await.unwrap();

    match &*winners {
        [] => {
            bot.send_message(
                msg.chat.id,
                "Et les heureux gagnants sont 🥁🥁🥁...  personne 😢",
            )
            .await?
        }

        _ => {
            bot.send_message(
                msg.chat.id,
                "Et les gagnants sont 🥁🥁🥁... ".to_owned() + &winners.join(", "),
            )
            .await?
        }
    };

    Ok(())
}

fn get_winner_ids(poll_answers: &HashMap<UserId, u8>, dice_value: u8) -> Vec<UserId> {
    poll_answers
        .iter()
        .filter_map(|(&user_id, &answer)| {
            if answer == dice_value {
                Some(user_id)
            } else {
                None
            }
        })
        .collect()
}

pub(crate) async fn register_answer(
    _bot: BotType,
    poll_answers: Arc<Mutex<HashMap<UserId, u8>>>,
    pa: PollAnswer,
) -> HandlerResult {
    poll_answers.lock().unwrap().insert(
        pa.voter.user().unwrap().id,
        pa.option_ids.first().unwrap() + 1,
    );
    Ok(())
}
