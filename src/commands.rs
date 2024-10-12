use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{Message, ReplyParameters},
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
