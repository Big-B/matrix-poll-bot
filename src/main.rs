mod statefull_handler;
mod unique_id_list;
use matrix_bot_api::handlers::{HandleResult, StatelessHandler};
use matrix_bot_api::{ActiveBot, MatrixBot, Message, MessageType};
use rand::Rng;
use serde::Deserialize;
use statefull_handler::StatefullHandler;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::fs::File;
use std::sync::{Arc, Mutex};
use unique_id_list::UniqueIdList;

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

#[derive(Debug, PartialEq, Eq, Hash)]
struct PollItem {
    count: usize,
    description: String,
}

#[derive(Debug)]
struct Poll {
    question: String,
    options: Vec<PollItem>,
}

static HELP_MSG: &str = r#"help - print this help message.
echo - repeat all text after command.
roll - return a random number between 1 and the given number (inclusive)
vote - takes one subcommand out of new, vote, close, or list
    new - start a new poll. Requires a question and at least 2 options separated by lines
    vote - vote on a poll. Takes the poll number and the option.
    close - close a poll. Takes the poll number or question and prints the results
    list - lists the current active polls"#;

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

fn handle_poll(
    bot: &ActiveBot,
    message: &Message,
    tail: &str,
    poll_map: Arc<Mutex<HashMap<String, UniqueIdList<Poll>>>>,
) -> HandleResult {
    let tail = tail.trim();
    if let Some(idx) = tail.find(char::is_whitespace) {
        let (cmd, cmd_tail) = tail.split_at(idx);
        match cmd {
            "vote" => handle_poll_vote(bot, message, cmd_tail, poll_map),
            "close" => handle_poll_close(bot, message, cmd_tail, poll_map),
            "new" => handle_poll_new(bot, message, cmd_tail, poll_map),
            _ => handle_poll_new(bot, message, tail, poll_map),
        }
    } else if tail == "list" {
        handle_poll_list(bot, message, tail, poll_map)
    } else {
        bot.send_message(HELP_MSG, &message.room, MessageType::TextMessage);
        HandleResult::StopHandling
    }
}

fn handle_poll_list(
    bot: &ActiveBot,
    message: &Message,
    _tail: &str,
    poll_map: Arc<Mutex<HashMap<String, UniqueIdList<Poll>>>>,
) -> HandleResult {
    let guard = poll_map.lock().unwrap();
    let mut statement = String::new();
    if let Some(list) = guard.get(&message.room) {
        // Iterate over all the polls and format a string that lists them all
        for (k, v) in list.iter() {
            writeln!(&mut statement, "Poll {}: {}", k, v.question).unwrap();
            for (i, option) in v.options.iter().enumerate() {
                writeln!(&mut statement, "\t{}: {}", i, option.description).unwrap();
            }
        }
    }
    if statement.is_empty() {
        statement.push_str("No active polls.");
    }

    bot.send_message(&statement, &message.room, MessageType::TextMessage);
    HandleResult::StopHandling
}

fn handle_poll_vote(
    bot: &ActiveBot,
    message: &Message,
    tail: &str,
    poll_map: Arc<Mutex<HashMap<String, UniqueIdList<Poll>>>>,
) -> HandleResult {
    let mut statement = String::new();
    let mut guard = poll_map.lock().unwrap();

    let tail: Vec<&str> = tail.trim().split_whitespace().collect();

    // Do some sanity checks
    if !guard.contains_key(&message.room) || tail.len() != 2 {
        statement.push_str("Vote takes 2 arguments, poll number and choice");
    } else {
        // Get the arguments, check to see if it's two usize first
        match (
            tail[0].trim().parse::<usize>(),
            tail[1].trim().parse::<usize>(),
        ) {
            (Ok(poll), Ok(vote)) => {
                // We have a poll number and an option number.
                // Get the poll from the map, and make sure the numbers
                // are valid,a nd then count the vote
                let list = guard.get_mut(&message.room).unwrap();
                if let Some(poll) = list.get_mut(poll) {
                    if let Some(poll_item) = poll.options.get_mut(vote) {
                        poll_item.count += 1;
                        writeln!(
                            &mut statement,
                            "{} voted for {}",
                            message.sender, poll_item.description
                        )
                        .unwrap();
                    } else {
                        statement.push_str("Invalid vote option");
                    }
                } else {
                    statement.push_str("Invalid poll");
                }
            }
            (Ok(poll), _) => {
                // We got a poll number and then... something else. See if we can
                // match the "something else" with the description of one of the options
                let list = guard.get_mut(&message.room).unwrap();
                if let Some(poll) = list.get_mut(poll) {

                    // Combine the collection of str into a single str to use for
                    // comparison
                    let vote_str = tail[1..].join(" ").trim().to_lowercase();
                    for mut option in poll.options.iter_mut() {
                        if option.description == vote_str {
                            option.count += 1;
                            writeln!(
                                &mut statement,
                                "{} voted for {}",
                                message.sender, option.description
                            )
                            .unwrap();
                        }
                    }
                }
            }
            _ => statement.push_str("Vote takes 2 arguments, poll number and choice"),
        }
    }

    // Respond with the generated message
    bot.send_message(&statement, &message.room, MessageType::TextMessage);
    HandleResult::StopHandling
}

fn get_poll_results_string(poll: &Poll) -> String {
    let mut results = String::new();
    writeln!(&mut results, "Results of poll: {}", poll.question).unwrap();
    for option in poll.options.iter() {
        writeln!(&mut results, "{}: {}", option.description, option.count).unwrap();
    }
    results
}

fn handle_poll_close(
    bot: &ActiveBot,
    message: &Message,
    tail: &str,
    poll_map: Arc<Mutex<HashMap<String, UniqueIdList<Poll>>>>,
) -> HandleResult {
    let mut statement = String::new();
    let mut guard = poll_map.lock().unwrap();

    // Get the map from the mutex
    if let Some(map) = guard.get_mut(&message.room) {
        // Check to see if we were given a number
        if let Ok(poll_num) = tail.trim().parse::<usize>() {
            match map.remove(poll_num) {
                Some(poll) => {
                    writeln!(&mut statement, "Poll {} removed.", poll_num).unwrap();
                    statement += &get_poll_results_string(&poll);
                }
                None => writeln!(&mut statement, "Invalid Index.").unwrap(),
            }
        } else {
            // We weren't given a number, see if we were given the question
            let mut idxs: Vec<usize> = Vec::new();
            for (k, v) in map.iter() {
                if v.question.to_lowercase() == tail.to_lowercase() {
                    idxs.push(*k);
                }
            }

            // Check what we found
            if idxs.is_empty() {
                writeln!(&mut statement, "Couldn't find matching poll.").unwrap();
            } else {
                // If we found indexes, remove them
                for idx in idxs {
                    let poll = map.remove(idx).unwrap();
                    writeln!(&mut statement, "Poll {} removed.", poll.question).unwrap();
                    statement += &get_poll_results_string(&poll);
                }
            }
        }
    } else {
        writeln!(&mut statement, "No polls in this room.").unwrap();
    }
    bot.send_message(&statement, &message.room, MessageType::TextMessage);
    HandleResult::StopHandling
}

fn handle_poll_new(
    bot: &ActiveBot,
    message: &Message,
    tail: &str,
    poll_map: Arc<Mutex<HashMap<String, UniqueIdList<Poll>>>>,
) -> HandleResult {
    // Make sure there's a question mark on the first line
    if let Some(idx) = tail.find('?') {
        let (question, tail) = tail.split_at(idx + 1);
        let question = question.trim();
        let mut poll = Poll {
            question: question.to_owned(),
            options: Vec::new(),
        };

        // Use a set to deduplicate entries
        let mut set = HashSet::new();

        // Check for valid options, add those to the hashmap
        for line in tail.trim().lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Don't care if it's inserted or not
            let _ = set.insert(PollItem {
                count: 0,
                description: line.to_owned(),
            });
        }

        // Collect into a vector
        poll.options = set.into_iter().collect();

        // Error check
        if poll.options.len() < 2 {
            bot.send_message(
                "Need at least two options!",
                &message.room,
                MessageType::TextMessage,
            );
            return HandleResult::StopHandling;
        }

        // Use the poll map to add the given poll to the given state. Use the
        // room ID as the map key.
        let mut guard = poll_map.lock().unwrap();
        let list = guard
            .entry(message.room.clone())
            .or_insert_with(UniqueIdList::new);
        let index = list.insert(poll);

        // Generate an output string to present to the room
        let mut statement = String::new();
        writeln!(&mut statement, "Poll #{}", index).unwrap();
        writeln!(&mut statement, "{}", question).unwrap();
        for (i, option) in list.get(index).unwrap().options.iter().enumerate() {
            writeln!(&mut statement, "{}. {}", i, option.description).unwrap();
        }

        // Send message to the users
        bot.send_message(&statement, &message.room, MessageType::TextMessage);
    } else {
        bot.send_message(
            "Didn't detect a question.",
            &message.room,
            MessageType::TextMessage,
        );
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

    let poll_map: Arc<Mutex<HashMap<String, UniqueIdList<Poll>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let mut state_handler = StatefullHandler::new(poll_map);

    handler.register_handle("help", |b, m, t| handle_help(b, m, t));
    handler.register_handle("echo", |b, m, t| handle_echo(b, m, t));
    handler.register_handle("roll", |b, m, t| handle_roll(b, m, t));
    state_handler.register_handle("poll", |b, m, t, s| handle_poll(b, m, t, s));

    let mut bot = MatrixBot::new(handler);
    bot.add_handler(state_handler);
    bot.run(
        &cfg.matrix.user_id,
        &cfg.matrix.access_token,
        &cfg.matrix.hs_url,
    );
}
