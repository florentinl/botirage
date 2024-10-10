use log::info;
use teloxide::{
    dispatching::dialogue::ErasedStorage,
    payloads::SetMessageReactionSetters,
    prelude::Dialogue,
    requests::Requester,
    types::{Dice, DiceEmoji, Message, MessageDice, MessageKind, ReactionType},
    Bot,
    RequestError::RetryAfter,
};

use crate::{utils::HandlerResult, State};

pub(crate) async fn emoji_games_handler(
    bot: Bot,
    dialogue: Dialogue<State, ErasedStorage<State>>,
    msg: Message,
) -> HandlerResult {
    let player = msg.clone().from.unwrap().id;
    let mut state = dialogue.get().await?.unwrap();
    if state.get(&player) < 1 {
        bot.send_message(msg.chat.id, "You don't have enough money to play!")
            .await?;
        bot.delete_message(msg.chat.id, msg.id).await?;
        return Ok(());
    }

    let dice_message = match msg.kind.clone() {
        MessageKind::Dice(MessageDice { dice: dice_message }) => dice_message,
        _ => unreachable!(),
    };

    let (reaction, score) = match dice_message {
        Dice {
            emoji: DiceEmoji::SlotMachine,
            value,
        } => slot_machine_handler(value),
        _ => todo!(),
    };

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        while let Err(RetryAfter(seconds)) = bot
            .set_message_reaction(msg.chat.id, msg.id)
            .reaction(vec![ReactionType::Emoji {
                emoji: reaction.to_string(),
            }])
            .await
        {
            info!("Retry after {:?}", seconds);
            tokio::time::sleep(seconds.duration()).await;
        }
    });

    state.insert(&player, score);
    dialogue.update(state).await?;

    Ok(())
}

pub fn slot_machine_handler(value: u8) -> (&'static str, i64) {
    let value = value - 1;
    let (left, middle, right) = (
        (value >> 4) & 0b11,
        (value >> 2) & 0b11,
        (value >> 0) & 0b11,
    );

    match (left, middle, right) {
        (3, 3, 3) => ("🔥", 30),
        _ if left == middle && left == right => ("🎉", 10),
        _ if left == middle || middle == right || left == right => ("😢", -1),
        _ => ("🥱", -1),
    }
}
