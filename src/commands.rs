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
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

pub(crate) async fn balance(bot: BotType, dialogue: DialogueType, msg: Message) -> HandlerResult {
    let player = msg.clone().from.unwrap().id;
    let state = dialogue.get().await?.unwrap();
    let player_money = state.get(&player);
    bot.send_message(
        msg.chat.id,
        format!(
            "@{}, tu as {}💵!",
            msg.from.unwrap().username.unwrap(),
            player_money
        ),
    )
    .reply_parameters(ReplyParameters::new(msg.id))
    .await?;
    Ok(())
}

pub(crate) async fn leaderboard(
    bot: BotType,
    dialogue: DialogueType,
    msg: Message,
) -> HandlerResult {
    let state = dialogue.get().await?.unwrap();
    let leaderboard = state.leaderboard();
    let mut message = "Classement ForbeCS:\n".to_owned();
    for &(user_id, money) in leaderboard.iter().take(10) {
        message.push_str(&format!(
            "{}: {}💵\n",
            get_username(&bot, msg.chat.id, &user_id).await,
            money
        ));
    }

    bot.send_message(msg.chat.id, message).await?;
    Ok(())
}
