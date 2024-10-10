use teloxide::{
    adaptors::Throttle,
    dispatching::dialogue::ErasedStorage,
    prelude::Dialogue,
    requests::Requester,
    types::{ChatId, UserId},
    Bot,
};

use crate::state::State;

pub(crate) type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
pub(crate) type BotType = Throttle<Bot>;
pub(crate) type DialogueType = Dialogue<State, ErasedStorage<State>>;

pub(crate) async fn get_username(bot: &BotType, chat_id: ChatId, user_id: &UserId) -> String {
    let user = bot.get_chat_member(chat_id, *user_id).await.unwrap().user;
    match user.username {
        Some(username) => username,
        None => format!("{}(mets un pseudo stp)", user.first_name),
    }
}

pub(crate) async fn get_usernames(bot: &BotType, chat_id: &ChatId, ids: &[UserId]) -> Vec<String> {
    let mut winners = vec![];
    for id in ids.into_iter() {
        winners.push(get_username(bot, *chat_id, &id).await);
    }
    winners
}
