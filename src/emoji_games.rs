use teloxide::{
    payloads::SetMessageReactionSetters,
    requests::Requester,
    types::{Message, ReactionType},
    Bot,
};

use crate::utils::HandlerResult;

pub(crate) async fn slot_machine_handler(bot: Bot, msg: Message, value: u8) -> HandlerResult {
    let value = value - 1;
    let (left, middle, right) = (
        (value >> 4) & 0b11,
        (value >> 2) & 0b11,
        (value >> 0) & 0b11,
    );

    tokio::time::sleep(std::time::Duration::from_millis(1600)).await;

    let reaction = match (left, middle, right) {
        (3, 3, 3) => "🔥",
        _ if left == middle && left == right => "🎉",
        _ if left == middle || middle == right || left == right => "😢",
        _ => "🥱",
    };
    bot.set_message_reaction(msg.chat.id, msg.id)
        .reaction(vec![ReactionType::Emoji {
            emoji: reaction.to_string(),
        }])
        .await?;

    Ok(())
}
