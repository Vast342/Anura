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
use crate::{
    board::{Board, Position},
    search::Engine,
    types::{bitboard::Bitboard, moves::Move, piece::Piece, square::Square, MoveList},
};
#[cfg(feature = "datagen")]
use rand::Rng;
#[cfg(feature = "datagen")]
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
#[cfg(feature = "datagen")]
pub const NODE_LIMIT: u128 = 1000;

// policy net datapoint, my own (probably bad and massive) format
#[cfg(feature = "datagen")]
#[cfg(feature = "policy")]
#[repr(C)]
// size = 160 (0xA0), align = 0x8
struct Datapoint {
    occupied: Bitboard,
    // 4 bits per piece, in order of the occ's bits, lsb to msb
    pieces: [u8; 16],
    // ctm, realistically it should be one bit but bruh
    ctm: u8,
    // number of visits on the root node is calculated from the sum of this array's visits
    // it's the 92 most visited moves out of however many the position has
    moves: [(Move, u16); 92],
}
#[cfg(feature = "datagen")]
#[cfg(feature = "policy")]
impl Datapoint {
    pub fn new(occ: Bitboard, pieces_: [u8; 16], ctm_: u8, moves_: [(Move, u16); 92]) -> Self {
        Self {
            occupied: occ,
            pieces: pieces_,
            ctm: ctm_,
            moves: moves_,
        }
    }
}

// value net datapoint, just text rn
#[cfg(feature = "datagen")]
#[cfg(feature = "value")]
struct Datapoint(pub String);

#[cfg(feature = "datagen")]
#[cfg(feature = "value")]
impl AddAssign for Datapoint {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += &rhs.0;
    }
}

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

#[cfg(feature = "datagen")]
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

#[cfg(feature = "datagen")]
// 0 if black won, 1 if draw, 2 if white won, 3 if error
fn run_game(datapoints: &mut Vec<Datapoint>, mut board: Board) -> u8 {
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
    let mut engine: Engine = Engine::new();
    // the rest of the moves
    for _ in 0..1000 {
        if board.is_drawn() {
            return 1;
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
                return 1;
            }
        }
        let (mov, score, mut visit_points) = engine.datagen_search(board.clone());
        board.make_move(mov);
        if board.is_drawn() {
            return 1;
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
                return 1;
            }
        }
        board.undo_move();
        if cfg!(feature = "policy") {
            let state: &Position = board.states.last().expect("bruh");
            let mut occ = state.occupied();
            let mut pieces = [0u8; 16];
            let mut index = 0;
            while !occ.is_empty() {
                let index1 = Square(occ.pop_lsb());
                let piece1 = state.piece_on_square(index1);
                let piece2 = if !occ.is_empty() {
                    let index2 = Square(occ.pop_lsb());
                    state.piece_on_square(index2)
                } else {
                    Piece(0)
                };
                pieces[index] = piece1.0 << 4 | piece2.0;
                index += 1;
            }
            occ = state.occupied();
            /*println!("{}", board.get_fen());
            dbg!(occ);
            for dn in 0..16 {
                println!("{}, {}", Piece(pieces[dn] >> 4), Piece(pieces[dn] & 0b1111));
            }
            panic!(":3");*/
            visit_points.sort_by(|a, b| b.1.cmp(&a.1));
            let mut thingies = [(Move::NULL_MOVE, 0); 92];
            let len = visit_points.len().min(thingies.len());
            thingies[..len].copy_from_slice(&visit_points[..len]);
            #[cfg(feature = "policy")]
            datapoints.push(Datapoint::new(occ, pieces, board.ctm, thingies));
        } else if cfg!(feature = "value") {
            #[cfg(feature = "value")]
            datapoints.push(Datapoint(format!(
                "{} | {} | ",
                board.get_fen(),
                score * (1 - i32::from(board.ctm) * 2)
            )));
        }
        board.make_move(mov);
    }
    let score = board.evaluate();
    if score < 0 {
        return 0;
    } else if score > 0 {
        return 2;
    } else {
        return 1;
    }
}

#[cfg(feature = "datagen")]
unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
}

#[cfg(feature = "datagen")]
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
    for mut point in datapoints {
        #[cfg(feature = "value")]
        {
            point += Datapoint((result as f64 / 2.0).to_string());
            point += Datapoint("\n".to_string());
            writer
                .write_all(point.0.as_bytes())
                .expect("failed to write to file");
        }
        #[cfg(feature = "policy")]
        unsafe {
            writer
                .write_all(any_as_u8_slice(&point))
                .expect("failed to write to file");
        }
    }
}
