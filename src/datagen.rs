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
// #[cfg(feature = "datagen")]
// #[cfg(feature = "policy")]
use crate::{
    board::{Board, Position},
    mcts::search::Engine,
    types::{bitboard::Bitboard, moves::Move, piece::Piece, square::Square, MoveList},
};
// #[cfg(feature = "datagen")]
#[cfg(feature = "value")]
use crate::{
    board::{Board, Position},
    mcts::search::Engine,
    types::{piece::Piece, square::Square, MoveList},
};
use montyformat::{MontyFormat, SearchData};
// #[cfg(feature = "datagen")]
use rand::Rng;
// #[cfg(feature = "datagen")]
#[allow(unused_imports)]
use std::{
    fs::File,
    io::{BufWriter, Write},
    ops::AddAssign,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread::{self},
    time::Instant,
};
// #[cfg(feature = "datagen")]
pub const NODE_LIMIT: u128 = 1000;

// policy net datapoint, montyformat now
// #[cfg(feature = "datagen")]
// #[cfg(feature = "policy")]
pub type Datapoint = MontyFormat;

// value net datapoint, just text rn
// #[cfg(feature = "datagen")]
#[cfg(feature = "value")]
struct Datapoint(pub String);

// #[cfg(feature = "datagen")]
#[cfg(feature = "value")]
impl AddAssign for Datapoint {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += &rhs.0;
    }
}

// #[cfg(feature = "datagen")]
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
            thread_function(
                value,
                1 + i as u8,
                &game_count_clone,
                &pos_count_clone,
                &draw_count_clone,
                start,
            )
        }));
    }
    for thread in threads {
        thread.join().unwrap();
    }
}

// #[cfg(feature = "datagen")]
fn thread_function(
    directory: String,
    thread_id: u8,
    game_count: &AtomicU64,
    position_count: &AtomicU64,
    draw_count: &AtomicU64,
    start: Instant,
) {
    let mut board: Board = Board::new();
    board.load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let this_directory = directory + "thread" + &thread_id.to_string() + ".txt";
    let mut writer = BufWriter::new(File::create(this_directory).expect("couldn't create file"));
    loop {
        let mut data: Vec<Datapoint> = vec![];
        let result = run_game(&mut data, board.clone());
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

// #[cfg(feature = "datagen")]
#[allow(unused_assignments)]
// 0 if black won, 1 if draw, 2 if white won, 3 if error
fn run_game(_datapoints: &mut Vec<Datapoint>, mut board: Board) -> u8 {
    // 8 random moves
    use crate::types::moves::Move;
    for _ in 0..8 {
        // generate the moves
        let mut list: MoveList = MoveList::new();
        board.get_moves(&mut list);
        // checkmate or stalemate, doesn't matter which
        // reset
        if list.len() == 0 {
            println!("hit a checkmate or stalemate in opening generation");
            return 3;
        }

        let index = rand::thread_rng().gen_range(0..list.len());
        board.make_move(list[index]);
    }

    let board_state = board.current_state();
    let starting_position = montyformat::chess::Position::from_raw(
        board_state.bb(),
        board.ctm == 0,
        board_state.ep_index.0,
        board_state.rights(),
        board_state.hm_clock,
        board.ply as u16,
    );

    let castling = montyformat::chess::Castling::from_raw(&starting_position, [[0, 7], [0, 7]]);

    let mut game = MontyFormat::new(starting_position, castling);

    let mut engine: Engine = Engine::new();
    // the rest of the moves
    for _ in 0..1000 {
        if board.is_drawn() {
            _datapoints.push(game);
            return 1;
        }
        // checkmate check
        // this is more efficient than it is in clarity lol
        // generate the moves
        let mut list: MoveList = MoveList::new();
        board.get_moves(&mut list);

        // checkmate or stalemate
        if list.len() == 0 {
            _datapoints.push(game);
            if board.in_check() {
                // checkmate opponnent wins
                return 2 - 2 * board.ctm;
            } else {
                return 1;
            }
        }
        #[allow(unused_variables)]
        let (mov, score, mut visit_points) = engine.datagen_search(board.clone());
        board.make_move(mov);
        if board.is_drawn() {
            _datapoints.push(game);
            return 1;
        }
        // checkmate check
        // this is more efficient than it is in clarity lol
        // generate the moves
        let mut list: MoveList = MoveList::new();
        board.get_moves(&mut list);

        // checkmate or stalemate
        if list.len() == 0 {
            _datapoints.push(game);
            if board.in_check() {
                // checkmate opponnent wins
                return 2 - 2 * board.ctm;
            } else {
                return 1;
            }
        }
        board.undo_move();
        if cfg!(feature = "policy") {
            let state: &Position = board.states.last().expect("bruh");
            let best_move = montyformat::chess::Move::from(mov.to_mf(state));
            let mut thing = vec![(montyformat::chess::Move::from(0), 0)];
            for point in &mut visit_points {
                thing.push((
                    montyformat::chess::Move::from(point.0.to_mf(state)),
                    point.1 as u32,
                ));
            }
            let data = SearchData::new(
                best_move,
                score as f32 * (-1 + i32::from(board.ctm) * 2) as f32,
                Some(thing),
            );
            game.push(data);
        } else if cfg!(feature = "value") {
            #[cfg(feature = "value")]
            datapoints.push(Datapoint(format!(
                "{} | {} | ",
                board.get_fen(),
                score * (-1 + i32::from(board.ctm) * 2)
            )));
        }
        board.make_move(mov);
    }
    let score = board.evaluate_non_stm();
    _datapoints.push(game);
    if score < -100 {
        return 0;
    } else if score > 100 {
        return 2;
    } else {
        return 1;
    }
}

// #[cfg(feature = "datagen")]
// #[cfg(feature = "policy")]
unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
}

// #[cfg(feature = "datagen")]
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
    position_count.fetch_add(datapoints.len() as u64, Ordering::Relaxed);

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
    #[allow(unused_mut)]
    for mut point in datapoints {
        #[cfg(feature = "value")]
        {
            point += Datapoint((result as f64 / 2.0).to_string());
            point += Datapoint("\n".to_string());
            writer
                .write_all(point.0.as_bytes())
                .expect("failed to write to file");
        }
        // #[cfg(feature = "policy")]
        unsafe {
            writer
                .write_all(any_as_u8_slice(&point))
                .expect("failed to write to file");
        }
    }
}
