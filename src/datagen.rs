/*
    Anura
    Copyright (C) 2025 Joseph Pasfield

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

/*
   HOW TO MAKE A DATAGEN BUILD (so i don't forget)
   1: set `go nodes` to use proper datagen tm (nothing to do yet because i'm just using fixed nodes)
   2: set makefile to add `--features datagen`
*/

// make datagen
// ./anura datagen 12 ../AnuraData/Text/ 124598902
// fix yo dang draw detection, `5R2/5Qp1/P6k/7p/8/2P4P/5PP1/6K1 w - - | 1033 | 0.5` is bad

use crate::{
    board::{Board, Position},
    mcts::search::Engine,
    mcts::search::EVAL_SCALE,
    tunable::Tunables,
    types::MoveList,
};
use montyformat::{chess::Castling, MontyFormat, SearchData};
use rand::Rng;
use std::io::{BufRead, BufReader};
use std::{
    fs::File,
    io::{BufWriter, Write},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread::{self},
    time::Instant,
};

pub const NODE_LIMIT: u128 = 1000;

// policy net datapoint, montyformat now
pub type Datapoint = MontyFormat;

pub fn datagen_main(args: Vec<String>) {
    let thread_count: usize = args[2].parse().expect("invalid thread count");
    println!("generating data on {thread_count} threads");
    let draw_count = Arc::new(AtomicU64::new(0));
    let game_count = Arc::new(AtomicU64::new(0));
    let pos_count = Arc::new(AtomicU64::new(0));
    let start = Instant::now();
    let tunables = Tunables::new();
    let mut threads = Vec::new();
    for i in 0..thread_count {
        let game_count_clone = Arc::clone(&game_count);
        let pos_count_clone = Arc::clone(&pos_count);
        let draw_count_clone = Arc::clone(&draw_count);
        let tunables_clone = tunables.clone();
        let value = args[3].clone();
        threads.push(thread::spawn(move || {
            thread_function(
                value,
                1 + i as u8,
                &game_count_clone,
                &pos_count_clone,
                &draw_count_clone,
                start,
                &tunables_clone,
            )
        }));
    }
    for thread in threads {
        thread.join().unwrap();
    }
}

fn thread_function(
    directory: String,
    thread_id: u8,
    game_count: &AtomicU64,
    position_count: &AtomicU64,
    draw_count: &AtomicU64,
    start: Instant,
    tunables: &Tunables,
) {
    let mut board: Board = Board::default();
    board.load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let this_directory = directory + "thread" + &thread_id.to_string() + ".bin";
    let mut writer = BufWriter::new(File::create(this_directory).expect("couldn't create file"));
    loop {
        let mut data: Vec<Datapoint> = vec![];
        let result = run_game(&mut data, board.clone(), tunables);
        if result != 3 {
            dump_to_file(
                data,
                &mut writer,
                game_count,
                position_count,
                draw_count,
                start,
                result,
            );
        }
    }
}

// 0 if black won, 1 if draw, 2 if white won, 3 if error
fn run_game(datapoints: &mut Vec<Datapoint>, mut board: Board, params: &Tunables) -> u8 {
    // 8 random moves
    for _ in 0..8 {
        // generate the moves
        let mut list: MoveList = MoveList::new();
        board.get_moves(&mut list);
        // checkmate or stalemate, doesn't matter which
        // reset
        if list.len() == 0 {
            return 3;
        }

        let index = rand::rng().random_range(0..list.len());
        board.make_move(list[index]);
    }

    let board_state = board.current_state();
    let starting_position = montyformat::chess::Position::from_raw(
        board_state.bb(),
        board.ctm == 0,
        if board_state.ep_index.0 == 64 {
            0
        } else {
            board_state.ep_index.0
        },
        board_state.rights(),
        board_state.hm_clock,
        board.ply as u16,
    );
    let castling = Castling::default();
    let mut game = MontyFormat::new(starting_position, castling);

    let mut engine: Engine = Engine::new();
    // the rest of the moves
    for _ in 0..1000 {
        if board.is_drawn() {
            datapoints.push(game);
            return 1;
        }
        // checkmate check
        // this is more efficient than it is in clarity lol
        // generate the moves
        let mut list: MoveList = MoveList::new();
        board.get_moves(&mut list);

        // checkmate or stalemate
        if list.len() == 0 {
            datapoints.push(game);
            return if board.in_check() {
                // checkmate opponnent wins
                2 - 2 * board.ctm
            } else {
                1
            };
        }

        let (mov, score, mut visit_points) = engine.datagen_search(board.clone(), params);
        board.make_move(mov);
        if board.is_drawn() {
            datapoints.push(game);
            return 1;
        }
        // checkmate check
        // this is more efficient than it is in clarity lol
        // generate the moves
        let mut list: MoveList = MoveList::new();
        board.get_moves(&mut list);

        // checkmate or stalemate
        if list.len() == 0 {
            datapoints.push(game);
            return if board.in_check() {
                // checkmate opponnent wins
                2 - 2 * board.ctm
            } else {
                1
            };
        }
        board.undo_move();
        let state: &Position = board.states.last().expect("bruh");
        let best_move = montyformat::chess::Move::from(mov.to_mf(state));
        // convert to montyformat move
        let mut thing = vec![];
        for point in &mut visit_points {
            thing.push((
                montyformat::chess::Move::from(point.0.to_mf(state)),
                point.1 as u32,
            ));
        }
        let sigmoided_score = 1.0 / (1.0 + (-score as f32 / EVAL_SCALE as f32).exp());
        let data = SearchData::new(best_move, sigmoided_score, Some(thing));
        game.push(data);
        board.make_move(mov);
    }
    let score = board.evaluate_non_stm();
    datapoints.push(game);
    return if score < -100 {
        0
    } else if score > 100 {
        2
    } else {
        1
    };
}

fn dump_to_file(
    datapoints: Vec<Datapoint>,
    writer: &mut BufWriter<File>,
    game_count: &AtomicU64,
    position_count: &AtomicU64,
    draw_count: &AtomicU64,
    start: Instant,
    result: u8,
) {
    game_count.fetch_add(1, Ordering::Relaxed);
    if result == 1 {
        draw_count.fetch_add(1, Ordering::Relaxed);
    }
    position_count.fetch_add(datapoints[0].moves.len() as u64, Ordering::Relaxed);
    // check stuff in game_count and print stuff if necessary
    let games = game_count.load(Ordering::Relaxed);
    if games % 128 == 0 {
        if games % 1024 == 0 {
            let positions = position_count.load(Ordering::Relaxed);
            println!("games: {games}");
            println!("draws: {}", draw_count.load(Ordering::Relaxed));
            println!("positions: {}", positions);
            println!("pos/sec: {}", positions / start.elapsed().as_secs());
            println!("games/sec: {}", games / start.elapsed().as_secs());
            println!("pos/game: {}", positions / games);
        }
        println!("finished with {games} games");
    }

    // push it to a file
    for point in &datapoints {
        let mut stuff = vec![];
        point.serialise_into_buffer(&mut stuff).unwrap();
        writer.write_all(&stuff).expect("failed to write to file");
        stuff.clear();
    }
}

// genfens, since I will be using OB for anura's datagen
use rand::rngs::StdRng;
use rand::SeedableRng;

fn get_opening<R: Rng>(start_fen: &str, rng: &mut R) -> Option<String> {
    let mut board = Board::default();
    board.load_fen(start_fen);
    // 8 random moves
    for _ in 0..8 {
        // generate the moves
        let mut list: MoveList = MoveList::new();
        board.get_moves(&mut list);
        // checkmate or stalemate, doesn't matter which
        // reset
        if list.len() == 0 {
            return None;
        }

        let index = rng.random_range(0..list.len());
        board.make_move(list[index]);
    }
    // final checkmate check
    let mut list: MoveList = MoveList::new();
    board.get_moves(&mut list);
    if list.len() == 0 {
        return None;
    }
    Some(board.get_fen(true))
}

fn rand_from_vector<R: Rng>(book: &Vec<String>, rng: &mut R) -> String {
    let len = book.len();
    let idx = rng.random_range(0..len);
    book[idx].clone()
}

pub fn gen_fens(args: Vec<String>) {
    // command is like ./engine "genfens N seed S book <None|Books/my_book.epd> <?extra>" "quit"
    let command_segments = args[1]
        .split_ascii_whitespace()
        .skip(1)
        .collect::<Vec<&str>>();
    let max_fens = command_segments[0].parse::<u64>().expect("Invalid number");
    let seed = command_segments[2].parse::<u64>().expect("Invalid Seed");
    let mut rng = StdRng::seed_from_u64(seed);
    let book_token = command_segments[4];
    let book: Vec<String> = if book_token == "None" {
        vec!["rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()]
    } else {
        BufReader::new(File::open(book_token).expect("Failed to open file"))
            .lines()
            .collect::<Result<Vec<String>, _>>()
            .expect("Failed to read lines")
    };

    let mut written_fens = 0;
    while written_fens < max_fens {
        let fen_option = get_opening(&rand_from_vector(&book, &mut rng), &mut rng);
        match fen_option {
            Some(fen) => {
                written_fens += 1;
                println!("info string genfens {fen}");
            }
            None => {}
        }
    }
}

// idk
pub const MIN_KLD: f64 = 0.000001;
