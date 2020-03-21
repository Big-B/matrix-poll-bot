use matrix_bot_api::handlers::{HandleResult, StatelessHandler};
use matrix_bot_api::{ActiveBot, MatrixBot, Message, MessageType};
use rand::Rng;
use serde::Deserialize;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct BotConfig {
    matrix: MatrixInfo,
}

#[derive(Debug, Deserialize)]
struct MatrixInfo {
    access_token: String,
    user_id: String,
    hs_url: String,
}

static HELP_MSG: &str = r#"help - print this help message.
echo - repeat all text after command.
roll - return a random number between 1 and the given number (inclusive)"#;

fn handle_help(bot: &ActiveBot, message: &Message, _tail: &str) -> HandleResult {
    bot.send_message(HELP_MSG, &message.room, MessageType::TextMessage);
    HandleResult::StopHandling
}
fn handle_echo(bot: &ActiveBot, message: &Message, tail: &str) -> HandleResult {
    bot.send_message(tail, &message.room, MessageType::TextMessage);
    HandleResult::StopHandling
}

fn handle_roll(bot: &ActiveBot, message: &Message, tail: &str) -> HandleResult {
    let tail = tail.trim();
    match tail.parse::<u128>() {
        Ok(0) | Err(_) => bot.send_message(
            &format!(
                "Must provide number greater than 0 and less than 2^120, instead of {}",
                tail
            ),
            &message.room,
            MessageType::TextMessage,
        ),
        Ok(x) => {
            let num = rand::thread_rng().gen_range(0, x) + 1;
            bot.send_message(&format!("{}", num), &message.room, MessageType::TextMessage);
        }
    }
    HandleResult::StopHandling
}

fn main() {
    let cfg: BotConfig = serde_yaml::from_reader(File::open("config.yaml").unwrap()).unwrap();
    let mut handler = StatelessHandler::new();
    handler.register_handle("shutdown", |bot, _, _| {
        bot.shutdown();
        HandleResult::ContinueHandling
    });

    handler.register_handle("help", |bot, message, tail| handle_help(bot, message, tail));
    handler.register_handle("echo", |bot, message, tail| handle_echo(bot, message, tail));
    handler.register_handle("roll", |bot, message, tail| handle_roll(bot, message, tail));

    let bot = MatrixBot::new(handler);
    bot.run(
        &cfg.matrix.user_id,
        &cfg.matrix.access_token,
        &cfg.matrix.hs_url,
    );
}
