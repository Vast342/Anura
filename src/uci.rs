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
use std::time::Instant;

use crate::{
    board::Board,
    mcts::{search::Engine, time::Limiters},
    movegen::lookups::BENCH_FENS,
    nets::policy::PolicyAccumulator,
    perft::{perft, run_perft_suite},
    tunable::Tunables,
    types::{moves::Move, MoveList},
};

const BENCH_DEPTH: u32 = 6;

pub enum CommandTypes {
    Uci,
    IsReady,
    Position,
    NewGame,
    Go,
    PrintState,
    Value,
    Perft,
    SplitPerft,
    PerftSuite,
    MakeMove,
    SetOption,
    Bench,
    GetFen,
    Policy,
    Tunables,
    Empty,
    Invalid,
    Quit,
}

pub struct UciOptions {
    pub more_info: bool,
    pub tree_size: u64,
    pub thread_count: u64,
}

impl UciOptions {
    fn new() -> Self {
        Self {
            more_info: false,
            tree_size: u64::MAX,
            thread_count: 1,
        }
    }
}

pub struct Manager {
    board: Board,
    engine: Engine,
    options: UciOptions,
    limiter: Limiters,
    tunables: Tunables,
    // TT
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}

impl Manager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            engine: Engine::new(),
            options: UciOptions::new(),
            limiter: Limiters::new(),
            tunables: Tunables::new(),
        }
    }
    // read line from stdin and then interpret it
    pub fn get_command(&mut self) -> bool {
        let mut buffer = String::new();

        io::stdin()
            .read_line(&mut buffer)
            .expect("failed to read from stdin");

        let (command, _new_line) = buffer.split_at(buffer.len() - 1);
        self.uci_interpret_command(command)
    }

    #[must_use]
    pub fn parse(&self, command: &str) -> CommandTypes {
        let mut command_split = command.split_ascii_whitespace();
        let Some(first_token) = command_split.next() else {
            return CommandTypes::Empty;
        };

        match first_token {
            "uci" => CommandTypes::Uci,
            "isready" => CommandTypes::IsReady,
            "position" => CommandTypes::Position,
            "ucinewgame" => CommandTypes::NewGame,
            "go" => CommandTypes::Go,
            "quit" => CommandTypes::Quit,
            "printstate" | "show" | "print" => CommandTypes::PrintState,
            "value" => CommandTypes::Value,
            "perft" => CommandTypes::Perft,
            "splitperft" => CommandTypes::SplitPerft,
            "perftsuite" => CommandTypes::PerftSuite,
            "makemove" => CommandTypes::MakeMove,
            "setoption" => CommandTypes::SetOption,
            "tunables" => CommandTypes::Tunables,
            "bench" => CommandTypes::Bench,
            "policy" => CommandTypes::Policy,
            _ => CommandTypes::Invalid,
        }
    }

    pub fn get_fen(&self) {
        println!("{}", self.board.get_fen());
    }

    pub fn uci_interpret_command(&mut self, command_text: &str) -> bool {
        let command = self.parse(command_text);

        match command {
            CommandTypes::Uci => self.uci_uci(),
            CommandTypes::IsReady => println!("readyok"),
            CommandTypes::Position => self.position(command_text),
            CommandTypes::Go => self.go(command_text),
            CommandTypes::NewGame => self.new_game(),
            CommandTypes::Invalid => println!("invalid or unsupported (for now) command"),
            CommandTypes::PrintState => self.board.print_state(),
            CommandTypes::Value => println!("evaluation {}", self.board.evaluate()),
            CommandTypes::Perft => self.perft(command_text),
            CommandTypes::SplitPerft => self.split_perft(command_text),
            CommandTypes::MakeMove => self.make_move(command_text),
            CommandTypes::SetOption => self.set_option(command_text),
            CommandTypes::PerftSuite => self.perft_suite(),
            CommandTypes::Bench => self.bench(),
            CommandTypes::GetFen => self.get_fen(),
            CommandTypes::Policy => self.output_policy(command_text),
            CommandTypes::Tunables => self.tunables.list(),
            CommandTypes::Quit => return false,
            _ => panic!("invalid command type"),
        }
        true
    }

    pub fn set_option(&mut self, command_text: &str) {
        let command_sections: Vec<&str> = command_text.split_ascii_whitespace().collect();
        match command_sections[2] {
            "Hash" => {
                self.options.tree_size = command_sections[4]
                    .parse::<u64>()
                    .expect("not a parsable hash size");
                self.engine.resize(self.options.tree_size as usize);
            }
            "Threads" => {
                self.options.thread_count = command_sections[4]
                    .parse::<u64>()
                    .expect("not a parsable thread count");
            }
            "MoreInfo" => {
                self.options.more_info = command_sections[4]
                    .parse::<bool>()
                    .expect("not a parsable hash size");
            }
            #[cfg(feature = "tunable")]
            _ => {
                self.tunables
                    .set(
                        command_sections[2],
                        command_sections[4]
                            .parse::<i32>()
                            .expect("not a parsable i32"),
                    )
                    .expect("teehee");
            }
            #[cfg(not(feature = "tunable"))]
            _ => panic!("Invalid option: {}", command_sections[2]),
        }
    }

    pub fn new_game(&mut self) {
        self.engine.new_game();
    }

    pub fn output_policy(&mut self, command_text: &str) {
        let mut moves = MoveList::new();
        self.board.get_moves(&mut moves);
        let command_split: Vec<&str> = command_text.split_ascii_whitespace().collect();
        let output_count = if command_split.len() != 1 {
            command_split[1]
                .parse::<usize>()
                .expect("invalid number of moves to write")
        } else {
            moves.len()
        };

        // get policy values
        let mut policy_acc = PolicyAccumulator::default();
        self.board.policy_load(&mut policy_acc);
        let mut tuples = vec![];
        let mut policy_sum: f32 = 0.0;
        for i in 0..moves.len() {
            tuples.push((
                moves[i],
                self.board.get_policy(moves[i], &mut policy_acc).exp(),
            ));
            policy_sum += tuples[i].1;
        }
        // normalize
        // could prob do some like .iter().enumerate() shenanigans here but ehhhhhh
        for i in 0..moves.len() {
            tuples[i].1 /= policy_sum;
        }
        // sort
        tuples.sort_by(|a, b| b.1.partial_cmp(&a.1).expect("bruh what"));
        // print
        for i in 0..output_count {
            println!("{}: {}", tuples[i].0, tuples[i].1);
        }
    }

    pub fn bench(&mut self) {
        let mut total = 0;
        let start = Instant::now();
        let mut board: Board = Board::new();
        let mut limiters = Limiters::new();
        limiters.load_values(0, 0, 0, BENCH_DEPTH, 0);
        for string in BENCH_FENS {
            board.load_fen(string);
            self.engine.search(
                board.clone(),
                limiters,
                false,
                &self.options,
                &self.tunables,
            );
            total += self.engine.nodes;
        }
        let duration = start.elapsed();
        println!(
            "{} nodes {} nps",
            total,
            (total as f64 / duration.as_secs_f64()) as u64
        );
    }

    pub fn go(&mut self, command_text: &str) {
        let command_sections: Vec<&str> = command_text.split_ascii_whitespace().collect();
        self.limiter.load_values(0, 0, 0, 0, 0);
        let mut i: usize = 1;
        let mut time: u128 = 0;
        let mut inc: u128 = 0;
        let mut nodes = 0;
        let mut depth = 0;
        let mut btime: u128 = 0;
        let mut wtime: u128 = 0;
        let mut binc: u128 = 0;
        let mut winc: u128 = 0;
        let mut movetime: u128 = 0;
        while i < command_sections.len() {
            match command_sections[i] {
                "depth" => {
                    i += 1;
                    if i >= command_sections.len() {
                        panic!("missing depth");
                    }

                    depth = command_sections[i]
                        .parse::<u32>()
                        .expect("not a parsable depth");
                }
                "nodes" => {
                    i += 1;
                    if i >= command_sections.len() {
                        eprintln!("Missing node count");
                        return;
                    }

                    nodes = command_sections[i]
                        .parse::<u128>()
                        .expect("not a parsable node limit");
                }
                "wtime" | "btime" | "winc" | "binc" => {
                    let token = command_sections[i];

                    i += 1;
                    if i >= command_sections.len() {
                        eprintln!("Missing {}", token);
                        return;
                    }

                    let Ok(value) = command_sections[i].parse::<i128>() else {
                        eprintln!("Invalid {} '{}'", token, command_sections[i]);
                        return;
                    };

                    match token {
                        "btime" => btime = value as u128,
                        "wtime" => wtime = value as u128,
                        "binc" => binc = value as u128,
                        "winc" => winc = value as u128,
                        _ => unreachable!(),
                    }
                }
                "movetime" => {
                    i += 1;
                    if i >= command_sections.len() {
                        eprintln!("missing movetime");
                        return;
                    }

                    movetime = command_sections[i]
                        .parse::<u128>()
                        .expect("not a parsable move time");
                }
                "infinite" => (),
                _ => println!("invalid go limiter: {}", command_sections[i]),
            }

            i += 1;
        }
        if self.board.ctm == 0 && btime != 0 {
            time = btime;
            inc = binc;
        } else if self.board.ctm == 1 && wtime != 0 {
            time = wtime;
            inc = winc;
        }
        self.limiter.load_values(time, inc, nodes, depth, movetime);
        let best_move = self.engine.search(
            self.board.clone(),
            self.limiter,
            true,
            &self.options,
            &self.tunables,
        );
        println!("bestmove {best_move}");
    }

    pub fn perft(&mut self, command_text: &str) {
        let mut command_split = command_text.split_ascii_whitespace();
        let _first_token = command_split.next().expect("not enough tokens");
        let second_token = command_split.next().expect("not enough tokens");
        let start = Instant::now();
        let nodes = perft(
            &mut self.board,
            second_token.parse().expect("invalid perft depth"),
        );
        let duration = start.elapsed();
        println!(
            "{} nodes {} nps",
            nodes,
            nodes as f64 / duration.as_secs_f64()
        );
    }

    pub fn split_perft(&mut self, command_text: &str) {
        let mut command_split = command_text.split_ascii_whitespace();
        let _first_token = command_split.next().expect("not enough tokens");
        let second_token = command_split.next().expect("not enough tokens");
        let depth: u8 = second_token.parse::<u8>().expect("invalid perft depth") - 1;
        let mut list: MoveList = MoveList::new();
        self.board.get_moves(&mut list);
        let mut total: u64 = 0;
        let start = Instant::now();
        for mov in list {
            self.board.make_move(mov);
            let nodes = perft(&mut self.board, depth);
            total += nodes;
            self.board.undo_move();
            println!("{mov}: {nodes}");
        }
        let duration = start.elapsed();
        println!("total: ");
        println!(
            "{} nodes {} nps",
            total,
            total as f64 / duration.as_secs_f64()
        );
    }

    pub fn perft_suite(&self) {
        run_perft_suite();
    }

    pub fn make_move(&mut self, command_text: &str) {
        let mut command_split = command_text.split_ascii_whitespace();
        let _first_token = command_split.next().expect("not enough tokens");
        let second_token = command_split.next().expect("not enough tokens");
        self.board
            .make_move(Move::from_text(second_token, &self.board));
    }

    pub fn position(&mut self, command_text: &str) {
        let mut command_split = command_text.split_ascii_whitespace();
        let _first_token = command_split.next().expect("not enough tokens");
        let second_token = command_split.next().expect("not enough tokens");
        let mut fen: String;
        if second_token == "startpos" {
            fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();
        } else if second_token == "kiwipete" {
            fen =
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string();
        } else {
            let third_token = command_split.next().expect("not enough tokens");
            fen = third_token.to_owned()
                + " "
                + command_split.next().expect("not enough tokens")
                + " "
                + command_split.next().expect("not enough tokens")
                + " "
                + command_split.next().expect("not enough tokens")
                + " ";
            let next_token = command_split.next();
            if let Some(string) = next_token {
                if string != "moves" {
                    fen += &(string.to_owned()
                        + " "
                        + command_split.next().expect("not enough tokens"));
                }
            }
        }
        self.board = Board::new();
        self.board.load_fen(&fen);
        // if there are moves
        if let Some(_moves_token) = command_split.next() {
            // loop through the rest of the moves
            for move_text in command_split {
                let mov: Move = Move::from_text(move_text, &self.board);
                self.board.make_move(mov);
            }
        }
    }

    // identify itself
    pub fn uci_uci(&self) {
        println!("id name Anura {}", env!("CARGO_PKG_VERSION"));
        println!("id author Vast");
        println!("option name Hash type spin default 32 min 1 max 1048576");
        println!("option name Threads type spin default 1 min 1 max 1048576");
        println!("option name MoreInfo type check default false");
        #[cfg(feature = "tunable")]
        self.tunables.print_options();
        println!("uciok");
    }
}
