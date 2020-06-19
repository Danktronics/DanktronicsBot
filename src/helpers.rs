use serenity::{
    model::channel::Message,
    Result
};

pub fn process_message_result(result: Result<Message>) {
    if let Err(error) = result {
        println!("Failed to send message: {:?}", error);
    }
}