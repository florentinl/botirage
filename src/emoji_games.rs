use teloxide::{
    payloads::SetMessageReactionSetters,
    requests::Requester,
    types::{Dice, DiceEmoji, Message, MessageDice, MessageKind, ReactionType},
};

use crate::utils::{BotType, DialogueType, HandlerResult};

pub(crate) async fn emoji_games_handler(
    bot: BotType,
    dialogue: DialogueType,
    msg: Message,
) -> HandlerResult {
    let mut state = dialogue.get().await?.ok_or("No state")?;
    let player = msg.from.ok_or("The message poster has disappeared")?;
    if state.get(&player.id) < &1 {
        bot.send_message(
            msg.chat.id,
            format!(
                "@{}, tu n'as plus assez d'argent pour jouer! Essaie de soudoyer le maître du jeu pour obtenir plus de 💵!",
                player.username.unwrap_or(player.first_name)
            ),
        )
        .await?;
        bot.delete_message(msg.chat.id, msg.id).await?;
        return Ok(());
    }

    let dice_message = match msg.kind {
        MessageKind::Dice(MessageDice { dice: dice_message }) => dice_message,
        _ => unreachable!(),
    };

    let (emoji, value) = match dice_message {
        Dice { emoji, value } => (emoji, value),
    };

    let (reaction, score, delay) = match emoji {
        DiceEmoji::SlotMachine => slot_machine_handler(value),
        DiceEmoji::Darts => darts_handler(value),
        DiceEmoji::Basketball => basketball_handler(value),
        DiceEmoji::Bowling => bowling_handler(value),
        DiceEmoji::Football => football_handler(value),
        _ => return Ok(()),
    };

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
        bot.set_message_reaction(msg.chat.id, msg.id)
            .reaction(vec![ReactionType::Emoji {
                emoji: reaction.to_string(),
            }])
            .await
    });

    state.insert(&player.id, score);
    dialogue.update(state).await?;

    Ok(())
}

fn slot_machine_handler(value: u8) -> (&'static str, i64, u64) {
    let value = value - 1;
    let (left, middle, right) = (
        (value >> 4) & 0b11,
        (value >> 2) & 0b11,
        (value >> 0) & 0b11,
    );

    let (reaction, score) = match (left, middle, right) {
        (3, 3, 3) => ("🔥", 30),
        _ if left == middle && left == right => ("🎉", 10),
        _ if left == middle || middle == right || left == right => ("😢", -1),
        _ => ("🥱", -1),
    };

    (reaction, score, 2)
}

fn darts_handler(value: u8) -> (&'static str, i64, u64) {
    let (reaction, score) = match value {
        1 => ("🤡", -2),
        2 => ("🥱", -2),
        3 => ("🤔", -2),
        4 => ("👀", -2),
        5 => ("🙊", -2),
        6 => ("😎", 12),
        _ => unreachable!(),
    };

    (reaction, score, 3)
}

fn basketball_handler(value: u8) -> (&'static str, i64, u64) {
    let (reaction, score) = match value {
        1 => ("🫡", -3),
        2 => ("🥱", -3),
        3 => ("🥴", -3),
        4 => ("🆒", 4),
        5 => ("🤝", 6),
        _ => unreachable!(),
    };

    (reaction, score, 4)
}

fn bowling_handler(value: u8) -> (&'static str, i64, u64) {
    let (reaction, score) = match value {
        1 => ("🌚", -3),
        2 => ("👨‍💻", -3),
        3 => ("🦄", -3),
        4 => ("😨", -3),
        5 => ("🤨", -3),
        6 => ("🗿", 16),
        _ => unreachable!(),
    };

    (reaction, score, 4)
}

fn football_handler(value: u8) -> (&'static str, i64, u64) {
    let (reaction, score) = match value {
        1 => ("🌭", -5),
        2 => ("🐳", -5),
        3 => ("💅", 1),
        4 => ("🙏", 5),
        5 => ("👏", 5),
        _ => unreachable!(),
    };

    (reaction, score, 4)
}
