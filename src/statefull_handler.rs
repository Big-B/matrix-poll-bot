use matrix_bot_api::handlers::{extract_command, HandleResult, Message, MessageHandler};
use matrix_bot_api::ActiveBot;
use std::collections::HashMap;

/// Convenience-handler that can quickly register and call functions
/// without any state (each function-call will result in the same output)
pub struct StatefullHandler<T: Clone> {
    cmd_prefix: String,
    cmd_handles: HashMap<String, fn(&ActiveBot, &Message, &str, T) -> HandleResult>,
    state: T,
}

impl<T: Clone> StatefullHandler<T> {
    pub fn new(state: T) -> StatefullHandler<T> {
        StatefullHandler {
            cmd_prefix: "!".to_string(),
            cmd_handles: HashMap::new(),
            state
        }
    }

    /// Register handles
    /// * command: For which command (excluding the prefix!) the handler should be called
    /// * handler: The handler to be called if the given command was received in the room
    ///
    /// Handler-function:
    /// * bot:     This bot
    /// * message: The message from fractal, containing the room the command was sent in, message body, etc.
    /// * tail:    The message-body without prefix and command (e.g. "!roll 12" -> "12")
    ///
    /// # Example
    /// handler.set_cmd_prefix("BOT:")
    /// handler.register_handle("sayhi", foo);
    /// foo() will be called, when BOT:sayhi is received by the bot
    pub fn register_handle(
        &mut self,
        command: &str,
        handler: fn(bot: &ActiveBot, message: &Message, tail: &str, state: T) -> HandleResult,
    ) {
        self.cmd_handles.insert(command.to_string(), handler);
    }
}

impl<T: Clone> MessageHandler for StatefullHandler<T> {
    fn handle_message(&mut self, bot: &ActiveBot, message: &Message) -> HandleResult {
        match extract_command(&message.body, &self.cmd_prefix) {
            Some(command) => {
                let func = self.cmd_handles.get(command).map(|x| *x);
                match func {
                    Some(func) => {
                        let end_of_prefix = self.cmd_prefix.len() + command.len();
                        func(bot, message, &message.body[end_of_prefix..], self.state.clone())
                    }
                    None => {
                        HandleResult::ContinueHandling
                    }
                }
            }
            None => {
                HandleResult::ContinueHandling /* Doing nothing. Not for us */
            }
        }
    }
}
