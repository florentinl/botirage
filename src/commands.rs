use teloxide::{
    payloads::{SendMessageSetters, SetMessageReactionSetters},
    requests::Requester,
    types::{Message, ReactionType, ReplyParameters},
    utils::command::BotCommands,
};

use crate::{
    utils::{get_username, BotType, DialogueType, HandlerResult},
    Command,
};

pub(crate) async fn help(bot: BotType, msg: Message) -> HandlerResult {
    let mut message = bot.send_message(msg.chat.id, Command::descriptions().to_string());
    if let Some(thread_msg_id) = msg.thread_id {
        message = message.message_thread_id(thread_msg_id);
    }
    message.await?;

    Ok(())
}

pub(crate) async fn balance(bot: BotType, dialogue: DialogueType, msg: Message) -> HandlerResult {
    let player = msg.from.ok_or("The message poster has disappeared")?;
    let state = dialogue.get().await?.ok_or("No state")?;
    let player_money = state.get(&player.id);
    let mut message = bot
        .send_message(
            msg.chat.id,
            format!(
                "@{}, tu as {}💵!",
                player.username.unwrap_or(player.first_name),
                player_money
            ),
        )
        .reply_parameters(ReplyParameters::new(msg.id));
    if let Some(thread_msg_id) = msg.thread_id {
        message = message.message_thread_id(thread_msg_id);
    }
    message.await?;

    Ok(())
}

pub(crate) async fn leaderboard(
    bot: BotType,
    dialogue: DialogueType,
    msg: Message,
) -> HandlerResult {
    let state = dialogue.get().await?.ok_or("No state")?;
    let leaderboard = state.leaderboard();
    let mut message = "Classement ForbeSupélec:\n".to_owned();
    for &(user_id, money) in leaderboard.iter().take(10) {
        message.push_str(&format!(
            "{}: {}💵\n",
            get_username(&bot, msg.chat.id, &user_id)
                .await
                .unwrap_or("____".to_string()),
            money
        ));
    }

    let mut message = bot.send_message(msg.chat.id, message);
    if let Some(thread_msg_id) = msg.thread_id {
        message = message.message_thread_id(thread_msg_id);
    }
    message.await?;

    Ok(())
}

pub(crate) async fn give_money(
    bot: BotType,
    dialogue: DialogueType,
    msg: Message,
) -> HandlerResult {
    if msg.clone().from.map(|user| user.id.0) != Some(1908102113) {
        bot.set_message_reaction(msg.chat.id, msg.id)
            .reaction(vec![ReactionType::Emoji {
                emoji: "🤣".to_string(),
            }])
            .await?;

        return Ok(()); // Only the bot owner can give money
    }

    // Ensure that the msg is a reply to another message
    let reply_to = msg.reply_to_message().ok_or("No message to reply to");
    let player = match reply_to {
        Ok(reply) => reply
            .to_owned()
            .from
            .ok_or("The message poster has disappeared")?,
        Err(_) => {
            bot.set_message_reaction(msg.chat.id, msg.id)
                .reaction(vec![ReactionType::Emoji {
                    emoji: "🤔".to_string(),
                }])
                .await?;

            return Ok(());
        }
    };

    let mut state = dialogue.get().await?.ok_or("No state")?;
    state.insert(&player.id, 100);

    bot.set_message_reaction(msg.chat.id, msg.id)
        .reaction(vec![ReactionType::Emoji {
            emoji: "👍".to_string(),
        }])
        .await?;

    dialogue.update(state).await?;

    Ok(())
}
