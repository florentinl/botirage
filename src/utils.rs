use teloxide::{
    requests::Requester,
    types::{ChatId, UserId},
    Bot,
};

pub(crate) type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub(crate) async fn get_username(bot: &Bot, chat_id: ChatId, user_id: &UserId) -> String {
    let user = bot.get_chat_member(chat_id, *user_id).await.unwrap().user;
    match user.username {
        Some(username) => username,
        None => format!("{}(mets un pseudo stp)", user.first_name),
    }
}

pub(crate) async fn get_usernames(bot: &Bot, chat_id: &ChatId, ids: &[UserId]) -> Vec<String> {
    let mut winners = vec![];
    for id in ids.into_iter() {
        winners.push(get_username(bot, *chat_id, &id).await);
    }
    winners
}
