use teloxide::{
    dispatching::dialogue::ErasedStorage,
    payloads::SetMessageReactionSetters,
    prelude::Dialogue,
    requests::Requester,
    types::{Dice, DiceEmoji, Message, MessageDice, MessageKind, ReactionType},
    Bot,
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
        bot.send_message(
            msg.chat.id,
            format!(
                "@{}, tu n'as plus assez d'argent pour jouer! Essaie de soudoyer le maître du jeu pour obtenir plus de 💵!",
                msg.from.unwrap().username.unwrap()
            ),
        )
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
        Dice {
            emoji: DiceEmoji::Darts,
            value,
        } => darts_handler(value),
        Dice {
            emoji: DiceEmoji::Basketball,
            value,
        } => basketball_handler(value),
        Dice {
            emoji: DiceEmoji::Bowling,
            value,
        } => bowling_handler(value),
        Dice {
            emoji: DiceEmoji::Football,
            value,
        } => football_handler(value),
        Dice {
            emoji: DiceEmoji::Dice,
            value: _,
        } => {
            bot.send_message(
                msg.chat.id,
                format!(
                    "Attention @{}! Seul le maître du jeu peut lancer les dés!",
                    msg.from.unwrap().username.unwrap()
                ),
            )
            .await?;
            bot.delete_message(msg.chat.id, msg.id).await?;
            return Ok(());
        }
    };

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        bot.set_message_reaction(msg.chat.id, msg.id)
            .reaction(vec![ReactionType::Emoji {
                emoji: reaction.to_string(),
            }])
            .await
    });

    state.insert(&player, score);
    dialogue.update(state).await?;

    Ok(())
}

fn slot_machine_handler(value: u8) -> (&'static str, i64) {
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

fn darts_handler(value: u8) -> (&'static str, i64) {
    match value {
        1 => ("🤡", -1),
        2 => ("🥱", -1),
        3 => ("🤔", -1),
        4 => ("👀", -1),
        5 => ("🙊", -1),
        6 => ("😎", 20),
        _ => unreachable!(),
    }
}

fn basketball_handler(value: u8) -> (&'static str, i64) {
    match value {
        1 => ("🫡", -1),
        2 => ("🥱", -1),
        3 => ("🥴", -1),
        4 => ("🆒", 10),
        5 => ("🤝", 20),
        _ => unreachable!(),
    }
}

fn bowling_handler(value: u8) -> (&'static str, i64) {
    match value {
        1 => ("🌚", -1),
        2 => ("👨‍💻", -1),
        3 => ("🤷‍♀", -1),
        4 => ("😨", 10),
        5 => ("🤨", 20),
        6 => ("🗿", 30),
        _ => unreachable!(),
    }
}

fn football_handler(value: u8) -> (&'static str, i64) {
    match value {
        1 => ("🌭", -1),
        2 => ("🐳", -1),
        3 => ("💅", 5),
        4 => ("🙏", 10),
        5 => ("👏", 20),
        6 => ("🦄", 30),
        _ => unreachable!(),
    }
}