use std::io;

pub enum CommandTypes {
    Uci,
    IsReady,
    Empty,
    Invalid,
}

// read line from stdin and then interpret it
pub fn uci_main() {
    let mut buffer = String::new();

    io::stdin()
        .read_line(&mut buffer)
        .unwrap();

    if buffer.is_empty() {
        return;
    }

    let (command, _new_line) = buffer.split_at(buffer.len() - 1);
    uci_interpret_command(command);
}

pub fn parse(command: &str) -> CommandTypes {
    let mut command_split = command.split_ascii_whitespace();
    let first_token = match command_split.next() {
        Some(string) => string,
        None => return CommandTypes::Empty,
    };

    match first_token {
        "uci" => CommandTypes::Uci,
        "isready" => CommandTypes::IsReady,
        _ => CommandTypes::Invalid,
    }
}

pub fn uci_interpret_command(command_text: &str) {
    let command = parse(command_text);

    match command {
        CommandTypes::Uci => uci_uci(),
        CommandTypes::IsReady => println!("readyok"),
        CommandTypes::Invalid => println!("invalid or unsupported (for now) command"),
        _ => assert!(false),
    }
}

// identify itself
pub fn uci_uci() {
    println!("id name Anura {}", env!("CARGO_PKG_VERSION"));
    println!("id author Vast");
    println!("option name Hash type spin default 0 min 0 max 0");
    println!("option name Threads type spin default 1 min 1 max 1");
}