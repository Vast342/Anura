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
#[cfg(feature = "datagen")]
use std::{fs::File, io::{BufWriter, Write}, sync::{atomic::{AtomicU64, Ordering}, Arc}, thread::{self}, time::Instant};
#[cfg(feature = "datagen")]
use crate::{board::Board, search::Engine, types::MoveList};
#[cfg(feature = "datagen")]
use rand::Rng;


#[cfg(feature = "datagen")]
pub fn datagen_main(args: Vec<String>) {
    let thread_count: usize = args[2].parse().expect("invalid thread count");
    println!("generating data on {thread_count} threads");
    let draw_count = Arc::new(AtomicU64::new(0));
    let game_count = Arc::new(AtomicU64::new(0));
    let pos_count = Arc::new(AtomicU64::new(0));
    let start = Instant::now();
    let mut threads = Vec::new();
    for i in 0..thread_count {
        let game_count_clone = Arc::clone(&game_count);
        let pos_count_clone = Arc::clone(&pos_count);
        let draw_count_clone = Arc::clone(&draw_count);
        let value = args[3].clone();
        threads.push(thread::spawn(move || {
            thread_function(value, 1 + i as u8, &game_count_clone, &pos_count_clone, &draw_count_clone, start)
        }));
    }
    for thread in threads {
        thread.join().unwrap();
    }
}

#[cfg(feature = "datagen")]
fn thread_function(directory: String, thread_id: u8, game_count: &AtomicU64, position_count: &AtomicU64, draw_count: &AtomicU64, start: Instant) {
    let mut board: Board = Board::new();
    board.load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let this_directory = directory + "thread" + &thread_id.to_string() + ".txt";
    let mut writer = BufWriter::new(File::create(this_directory).expect("couldn't create file"));
    loop {
        let mut data: Vec<String> = vec![];
        let result = run_game(&mut data, board.clone());
        if result != 3 {
            dump_to_file(data, &mut writer, game_count, position_count, draw_count, start, result);
        }
    }
}

#[cfg(feature = "datagen")]
// 0 if black won, 1 if draw, 2 if white won, 3 if error
fn run_game(strings: &mut Vec<String>, mut board: Board) -> u8 {
    // 8 random moves
    for _ in 0..8 {
        // generate the moves
        let mut list: MoveList = MoveList::new();
        board.get_moves(&mut list);
        // checkmate or stalemate, doesn't matter which
        // reset
        if list.len() == 0 {
            println!("hit a checkmate or stalemate in opening generation");
            return 3
        }

        let index = rand::thread_rng().gen_range(0..list.len());
        board.make_move(list[index]);
    }
    let mut engine: Engine = Engine::new();
    // the rest of the moves
    for _ in 0..1000 {
        let (mov, score) = engine.search(board.clone(), 1000, 1000000000, 1000000, false);
        board.make_move(mov);
        if board.is_drawn() {
            return 1
        }
        // checkmate check
        // this is more efficient than it is in clarity lol
        // generate the moves
        let mut list: MoveList = MoveList::new();
        board.get_moves(&mut list);

        // checkmate or stalemate
        if list.len() == 0 {
            if board.in_check() {
                // checkmate opponnent wins
                return 2 - 2 * board.ctm;
            } else {
                return 1
            }
        }

        strings.push(format!("{} | {} | ", board.get_fen(), score * (1 - i32::from(board.ctm) * 2)));
    }
    let score = board.evaluate();
    if score < 0 { return 0 } else if score > 0 { return 2 } else { return 1 }
}

#[cfg(feature = "datagen")]
fn dump_to_file(strings: Vec<String>, writer: &mut BufWriter<File>, game_count: &AtomicU64, position_count: &AtomicU64, draw_count: &AtomicU64, start: Instant, result: u8) {
    game_count.fetch_add(1, Ordering::Relaxed);
    if result == 1 { draw_count.fetch_add(1, Ordering::Relaxed); }
    position_count.fetch_add(strings.len() as u64, Ordering::Relaxed);

    // check stuff in game_count and print stuff if necessary
    let games = game_count.load(Ordering::Relaxed);
    if games % 128 == 0 {
        if games % 1024 == 0 {
            let positions = position_count.load(Ordering::Relaxed);
            println!("games: {games}");
            println!("draws: {}", draw_count.load(Ordering::Relaxed));
            println!("positions: {}", positions);
            println!("pos/sec: {}", positions / start.elapsed().as_secs());
        }
        println!("finished with {games} games");
    }

    // push it to a file
    for mut line in strings {
        line += &(result as f64 / 2.0).to_string();
        line += "\n";
        writer.write_all(line.as_bytes()).expect("failed to write to file");
    }
}