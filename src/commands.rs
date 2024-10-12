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
    let player = msg.clone().from.unwrap().id;
    let state = dialogue.get().await?.unwrap();
    let player_money = state.get(&player);
    let mut message = bot
        .send_message(
            msg.chat.id,
            format!(
                "@{}, tu as {}💵!",
                msg.from.unwrap().username.unwrap(),
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
    let state = dialogue.get().await?.unwrap();
    let leaderboard = state.leaderboard();
    let mut message = "Classement ForbeSupélec:\n".to_owned();
    for &(user_id, money) in leaderboard.iter().take(10) {
        message.push_str(&format!(
            "{}: {}💵\n",
            get_username(&bot, msg.chat.id, &user_id).await,
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
