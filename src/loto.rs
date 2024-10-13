use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
};

use teloxide::{
    adaptors::Throttle,
    payloads::{SendDiceSetters, SendMessageSetters, SendPollSetters, UnpinChatMessageSetters},
    requests::Requester,
    types::{Dice, DiceEmoji, Message, MessageDice, MessageKind, PollAnswer, UserId},
    Bot,
};

use crate::state::State;
use crate::utils::{get_usernames, BotType, DialogueType, HandlerResult};

pub(crate) async fn start_loto(
    bot: BotType,
    dialogue: DialogueType,
    poll_answers: Arc<Mutex<HashMap<UserId, u8>>>,
    msg: Message,
) -> HandlerResult {
    poll_answers
        .lock()
        .unwrap_or_else(|err| err.into_inner())
        .clear();

    let state = dialogue.get().await?.ok_or("No state")?;
    let mut poll = bot
        .send_poll(
            msg.chat.id,
            "Placez vos paris! Vous avez 1 minute.",
            (1..=6).map(|x| x.to_string()),
        )
        .is_anonymous(false);

    if let Some(thread_msg_id) = msg.thread_id {
        poll = poll.message_thread_id(thread_msg_id);
    }

    let poll = poll.await?;
    bot.pin_chat_message(msg.chat.id, poll.id).await?;

    dialogue
        .update(state.to_receiving_poll_answers(poll))
        .await?;

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        let _ = draw_loto(bot, dialogue, msg, poll_answers).await;
    });

    Ok(())
}

async fn draw_loto(
    bot: BotType,
    dialogue: DialogueType,
    msg: Message,
    poll_answers: Arc<Mutex<HashMap<UserId, u8>>>,
) -> HandlerResult {
    let mut state = dialogue.get().await?.ok_or("No state")?;

    let dice_value = draw_die(&bot, &msg).await?;

    let (winner_ids, winners) =
        get_poll_winners(&state, &bot, &msg, poll_answers, dice_value).await?;

    tokio::time::sleep(std::time::Duration::from_secs(4)).await;

    for winner_id in &winner_ids {
        state.insert(winner_id, 500);
    }
    dialogue.update(state.to_idle()).await?;

    announce_winners(winners, bot, msg).await?;

    Ok(())
}

async fn announce_winners(
    winners: Vec<String>,
    bot: Throttle<Bot>,
    msg: Message,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let content = match &*winners {
        [] => "Et les heureux gagnants sont 🥁🥁🥁...  personne 😢".to_string(),
        _ => "Et les gagnants sont 🥁🥁🥁... ".to_string() + &winners.join(", "),
    };
    let mut message = bot.send_message(msg.chat.id, content);
    if let Some(thread_msg_id) = msg.thread_id {
        message = message.message_thread_id(thread_msg_id);
    }
    message.await?;
    Ok(())
}

async fn get_poll_winners(
    state: &State,
    bot: &Throttle<Bot>,
    msg: &Message,
    poll_answers: Arc<Mutex<HashMap<UserId, u8>>>,
    dice_value: u8,
) -> Result<(Vec<UserId>, Vec<String>), Box<dyn Error + Send + Sync>> {
    let poll = match *state {
        State::ReceivingPollAnswers { ref poll, .. } => poll,
        _ => return Err("Invalid state".into()),
    };
    bot.stop_poll(msg.chat.id, poll.id).await?;
    bot.unpin_chat_message(msg.chat.id)
        .message_id(poll.id)
        .await?;
    let winner_ids = get_winner_ids(
        &poll_answers
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .to_owned(),
        dice_value,
    );
    let winners = get_usernames(bot, &msg.chat.id, &winner_ids).await;
    Ok((winner_ids, winners))
}

async fn draw_die(
    bot: &teloxide::adaptors::Throttle<teloxide::Bot>,
    msg: &Message,
) -> Result<u8, Box<dyn Error + Send + Sync>> {
    let mut message = bot.send_message(
        msg.chat.id,
        "Les paris sont fermés. C'est l'heure du lancé...",
    );
    let mut dice = bot.send_dice(msg.chat.id);
    if let Some(thread_msg_id) = msg.thread_id {
        message = message.message_thread_id(thread_msg_id);
        dice = dice.message_thread_id(thread_msg_id);
    }
    message.await?;
    let dice = dice.await?;
    let dice_value = match dice.kind {
        MessageKind::Dice(MessageDice {
            dice:
                Dice {
                    emoji: DiceEmoji::Dice,
                    value,
                },
        }) => value,
        _ => return Err("How the fuck did telegram turn a dice into something else ?".into()),
    };
    Ok(dice_value)
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
    match pa {
        PollAnswer {
            option_ids,
            voter,
            poll_id: _,
        } => {
            let mut poll_answers = poll_answers.lock().unwrap_or_else(|err| err.into_inner());
            let voter = voter.user().ok_or("Voter vanished from channel")?;
            if let Some(option_id) = option_ids.first() {
                poll_answers.insert(voter.id, *option_id + 1);
            } else {
                // Remove the user's answer if they removed their vote
                poll_answers.remove(&voter.id);
            }
        }
    };
    Ok(())
}

pub(crate) async fn reset_roll(dialogue: DialogueType) -> HandlerResult {
    let state = dialogue.get().await?.ok_or("No state")?;
    dialogue.update(state.to_idle()).await?;
    Ok(())
}
