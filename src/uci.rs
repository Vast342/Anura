/*
    Anura
    Copyright (C) 2024 Joseph Pasfield

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::io;

use crate::board::Board;

pub enum CommandTypes {
    Uci,
    IsReady,
    Position,
    NewGame,
    PrintState,
    Evaluate,
    Empty,
    Invalid,
    Quit,
}

pub struct Manager {
    board: Board,
    // engine: Engine (SOON)
    // TT
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}

impl Manager {
    #[must_use] pub fn new() -> Self {
        let mut b: Board = Board::new();
        b.load_fen("8/8/8/8/8/8/8/8 w - - 0 1");
        Self{board: b}
    }
    // read line from stdin and then interpret it
    pub fn get_command(&mut self) -> bool{
        let mut buffer = String::new();

        io::stdin()
            .read_line(&mut buffer)
            .expect("failed to read from stdin");

        let (command, _new_line) = buffer.split_at(buffer.len() - 1);
        self.uci_interpret_command(command)
    }

    #[must_use] pub fn parse(&self, command: &str) -> CommandTypes {
        let mut command_split = command.split_ascii_whitespace();
        let Some(first_token) = command_split.next() else { return CommandTypes::Empty };

        match first_token {
            "uci" => CommandTypes::Uci,
            "isready" => CommandTypes::IsReady,
            "position" => CommandTypes::Position,
            "quit" => CommandTypes::Quit,
            "printstate" | "show" | "print" => CommandTypes::PrintState,
            "evaluate" => CommandTypes::Evaluate,
            _ => CommandTypes::Invalid,
        }
    }

    pub fn uci_interpret_command(&mut self, command_text: &str) -> bool {
        let command = self.parse(command_text);

        match command {
            CommandTypes::Uci => self.uci_uci(),
            CommandTypes::IsReady => println!("readyok"),
            CommandTypes::Position => self.position(command_text),
            CommandTypes::Invalid => println!("invalid or unsupported (for now) command"),
            CommandTypes::PrintState => self.board.print_state(),
            CommandTypes::Evaluate => println!("evaluation {}", self.board.evaluate()),
            CommandTypes::Quit => return false,
            _ => panic!("invalid command type"),
        }
        true
    }

    pub fn position(&mut self, command_text: &str) {
        let mut command_split = command_text.split_ascii_whitespace();
        let _first_token = command_split.next().expect("not enough tokens");
        let second_token = command_split.next().expect("not enough tokens");
        let mut fen: String;
        if second_token == "startpos" {
            fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();
        } else if second_token == "kiwipete" {
            fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string();
        } else {
            let third_token = command_split.next().expect("not enough tokens");
            fen = third_token.to_owned() + " "
                + command_split.next().expect("not enough tokens") + " " 
                + command_split.next().expect("not enough tokens") + " " 
                + command_split.next().expect("not enough tokens") + " ";
            let next_token = command_split.next();
            if let Some(string) = next_token { if string != "moves" {
                fen += &(string.to_owned() + " "
                     + command_split.next().expect("not enough tokens"));
            } }
        }
        self.board = Board::new();
        self.board.load_fen(&fen);
        // parse moves
    }

    // identify itself
    pub fn uci_uci(&self) {
        println!("id name Anura {}", env!("CARGO_PKG_VERSION"));
        println!("id author Vast");
        println!("option name Hash type spin default 0 min 0 max 0");
        println!("option name Threads type spin default 1 min 1 max 1");
    }
}