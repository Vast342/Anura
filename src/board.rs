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

pub static SQUARE_NAMES: [&str; 64] = [
    "a1","b1","c1","d1","e1","f1","g1","h1",
    "a2","b2","c2","d2","e2","f2","g2","h2",
    "a3","b3","c3","d3","e3","f3","g3","h3",
    "a4","b4","c4","d4","e4","f4","g4","h4",
    "a5","b5","c5","d5","e5","f5","g5","h5",
    "a6","b6","c6","d6","e6","f6","g6","h6",
    "a7","b7","c7","d7","e7","f7","g7","h7",
    "a8","b8","c8","d8","e8","f8","g8","h8",
];

pub const ROOK_RIGHT_MASKS: [u8; 64] = [
    0b1101,255,255,255,255,255,255,0b1110,
    255,   255,255,255,255,255,255,   255,
    255,   255,255,255,255,255,255,   255,
    255,   255,255,255,255,255,255,   255,
    255,   255,255,255,255,255,255,   255,
    255,   255,255,255,255,255,255,   255,
    255,   255,255,255,255,255,255,   255,
    0b0111,255,255,255,255,255,255,0b1011
]; 
pub const KING_RIGHT_MASKS: [u8; 2] = [
    0b0011, 0b1100
];

use crate::{
    eval::PIECE_WEIGHTS, movegen::{others::{get_king_attacks, get_knight_attacks}, pawns::{get_pawn_attacks_lookup, get_pawn_attacks_setwise, get_pawn_pushes_setwise}, slideys::{get_bishop_attacks, get_rook_attacks}}, types::{
        bitboard::Bitboard, moves::{Flag, Move}, piece::{
            Colors, Piece, Types
        }, square::Square, MoveList
    }
};

//use std::vec;

#[derive(Debug, Copy, Clone)]
pub struct Position {
    colors:   [Bitboard; 2],
    pieces:   [Bitboard; 6],
    mailbox:  [Piece; 64],
    eval: i32,
    king_sqs: [Square; 2],
    ep_index:  Square,
    hm_clock:  u8,
    castling:  u8,
}

impl Position {
    #[must_use] pub const fn empty() -> Self {
        let col: [Bitboard; 2] = [Bitboard(0); 2];
        let pcs: [Bitboard; 6] = [Bitboard(0); 6];
        let mail: [Piece; 64] = [Piece(Types::None as u8); 64];
        let ksqs: [Square; 2] = [Square(64); 2];
        let epsq: Square = Square(64);
        let hmc: u8 = 0;
        let ca: u8 = 0;
        let ev: i32 = 0;

        Self {colors: col, pieces: pcs, mailbox: mail, king_sqs: ksqs, ep_index: epsq, hm_clock: hmc, castling: ca, eval: ev}
    }
    pub fn add_piece(&mut self, sq: Square, piece: Piece) {
        let bitboard_square: Bitboard = Bitboard::from_square(sq);
        self.colors[piece.color() as usize] ^= bitboard_square;
        self.pieces[piece.piece() as usize] ^= bitboard_square;
        self.mailbox[sq.as_usize()] = piece;
        self.eval += PIECE_WEIGHTS[piece.piece() as usize] * (-1 + i32::from(piece.color()) * 2);
    }
    pub fn remove_piece(&mut self, sq: Square, piece: Piece) {
        let bitboard_square: Bitboard = Bitboard::from_square(sq);
        self.colors[piece.color() as usize] ^= bitboard_square;
        self.pieces[piece.piece() as usize] ^= bitboard_square;
        self.mailbox[sq.as_usize()] = Piece(Types::None as u8);
        self.eval -= PIECE_WEIGHTS[piece.piece() as usize] * (-1 + i32::from(piece.color()) * 2);
    }
    #[must_use] pub const fn piece_on_square(&self, sq: Square) -> Piece {
        self.mailbox[sq.as_usize()]
    }
    #[must_use] pub fn occupied(&self) -> Bitboard {
        self.colors[0] & self.colors[1]
    }
}

#[derive(Debug, Clone, Default)]
pub struct Board {
    states: Vec<Position>,
    ctm: u8,
    ply: i16,
    // phase: i8
}

impl Board {
    #[must_use] pub fn new() -> Self {
        Self {states: vec![Position::empty(); 256], ctm: 0, ply: 0}
    }
    pub fn print_state(&self) {
        for i in (0..8).rev() {
            for j in 0..8 {
                print!("{} ", (self.states.last().expect("no state???").mailbox[i*8+j]));
            }
            println!();
        }
    }
    #[allow(clippy::cast_possible_truncation)] pub fn load_fen(&mut self, fen: &str) {
        let mut state: Position = Position::empty();
        let mut fen_split = fen.split_ascii_whitespace();
        // first token: position
        let mut token = fen_split.next().expect("no position?");
        let mut ranks = token.rsplit('/');
        let mut i: Square = Square(0);
        for rank in ranks.by_ref() {
            for c in rank.chars() {
                match c {
                    'p' => {
                        state.add_piece(i, Piece::new_unchecked(Types::Pawn as u8, Colors::Black as u8));
                        i += Square(1);
                    }
                    'P' => {
                        state.add_piece(i, Piece::new_unchecked(Types::Pawn as u8, Colors::White as u8));
                        i += Square(1);
                    }
                    'n' => {
                        state.add_piece(i, Piece::new_unchecked(Types::Knight as u8, Colors::Black as u8));
                        i += Square(1);
                    }
                    'N' => {
                        state.add_piece(i, Piece::new_unchecked(Types::Knight as u8, Colors::White as u8));
                        i += Square(1);
                    }
                    'b' => {
                        state.add_piece(i, Piece::new_unchecked(Types::Bishop as u8, Colors::Black as u8));
                        i += Square(1);
                    }
                    'B' => {
                        state.add_piece(i, Piece::new_unchecked(Types::Bishop as u8, Colors::White as u8));
                        i += Square(1);
                    }
                    'r' => {
                        state.add_piece(i, Piece::new_unchecked(Types::Rook as u8, Colors::Black as u8));
                        i += Square(1);
                    }
                    'R' => {
                        state.add_piece(i, Piece::new_unchecked(Types::Rook as u8, Colors::White as u8));
                        i += Square(1);
                    }
                    'q' => {
                        state.add_piece(i, Piece::new_unchecked(Types::Queen as u8, Colors::Black as u8));
                        i += Square(1);
                    }
                    'Q' => {
                        state.add_piece(i, Piece::new_unchecked(Types::Queen as u8, Colors::White as u8));
                        i += Square(1);
                    }
                    'k' => {
                        state.add_piece(i, Piece::new_unchecked(Types::King as u8, Colors::Black as u8));
                        state.king_sqs[0] = i;
                        i += Square(1);
                    }
                    'K' => {
                        state.add_piece(i, Piece::new_unchecked(Types::King as u8, Colors::White as u8));
                        state.king_sqs[1] = i;
                        i += Square(1);
                    }
                    _ => i += Square(c.to_digit(10).expect("invalid character in fen") as u8),
                }
            }
        }

        // second token: color to move
        token = fen_split.next().expect("no ctm?"); 
        self.ctm = u8::from(token == "w");

        // third token: castling rights
        token = fen_split.next().expect("no castling rights?");
        for c in token.chars() { 
            match c {
                'K' => state.castling |= 1,
                'Q' => state.castling |= 2,
                'k' => state.castling |= 4,
                'q' => state.castling |= 8,
                '-' | ' ' => (),
                _ => panic!("invalid castling rights (Anura doesn't support frc), you gave {c}"),
            }
        } 

        // fourth token: en passant
        token = fen_split.next().expect("no en passant index?"); 
        if token == "-" { state.ep_index = Square(64); } else {
            for i in 0..63 {
                if token == SQUARE_NAMES[i as usize] { 
                    state.ep_index = Square(i); 
                    break;
                };
            }
        }
        
        // here on out is optional:
        // fifth token: half move clock
        let mut token_option = fen_split.next(); 
        if token_option.is_some() {
            state.hm_clock = 0;
            // sixth token: ply count
            token_option = fen_split.next(); 
            self.ply = token_option.expect("why would you have a 5th token but not a 6th").parse().unwrap();
        }

        self.states.push(state);
    }
    pub fn get_moves(&self, list: &mut MoveList) {
        let state = self.states.last().expect("no state");
        let occ: Bitboard = state.occupied();
        let empties: Bitboard = !occ;
        let mut us: Bitboard = state.colors[self.ctm as usize];
        if (state.castling & KING_RIGHT_MASKS[1 - self.ctm as usize]) != 0 && !self.in_check() {
            if (state.castling & 1) != 0 && (occ & Bitboard(0x60) == Bitboard(0)) && !self.square_attacked(Square(5)) {
                list.push(Move::new_unchecked(4, 6, Flag::WKCastle as u8));
            }
            if (state.castling & 2) != 0 && (occ & Bitboard(0xE) == Bitboard(0)) && !self.square_attacked(Square(3)) {
                list.push(Move::new_unchecked(4, 2, Flag::WQCastle as u8));
            }
            if (state.castling & 4) != 0 && (occ & Bitboard(0x6000_0000_0000_0000) == Bitboard(0)) && !self.square_attacked(Square(61)) {
                list.push(Move::new_unchecked(60, 62, Flag::BKCastle as u8));
            }
            if (state.castling & 8) != 0 && (occ & Bitboard(0x0E00_0000_0000_0000) == Bitboard(0)) && !self.square_attacked(Square(59)) {
                list.push(Move::new_unchecked(60, 58, Flag::BQCastle as u8));
            } 
        }
        while us != Bitboard(0) {
            let index = us.pop_lsb();
            let piece = state.piece_on_square(Square(index));
            let mut current_attack: Bitboard = match piece.piece() {
                // pawns (we do them setwise later)
                0 => Bitboard(0),
                // knights
                1 => {
                    get_knight_attacks(Square(index))
                },
                // bishops
                2 => {
                    get_bishop_attacks(Square(index), occ)
                },
                // rooks
                3 => {
                    get_rook_attacks(Square(index), occ)
                },
                // queens
                4 => {
                    get_bishop_attacks(Square(index), occ) | get_rook_attacks(Square(index), occ)
                },
                // kings
                5 => {
                    get_king_attacks(Square(index))
                },
                _ => panic!("invalid piece, value of {}", piece.piece()),
            };
            // make sure you can't capture your own pieces
            current_attack ^= current_attack & state.colors[1 - self.ctm as usize];
            // convert it into moves
            while current_attack != Bitboard(0) {
                let end = current_attack.pop_lsb();
                list.push(Move::new_unchecked(index, end, Flag::Normal as u8));
            }
            
        }
        // setwise pawns
        let pawns = state.pieces[Types::Pawn as usize];
        let (mut single_pushes, mut double_pushes) = get_pawn_pushes_setwise(pawns, empties, self.ctm);
        // identify promotions
        let mut pawn_push_promotions = single_pushes & Bitboard::from_rank(7 * self.ctm);
        single_pushes ^= pawn_push_promotions;
        while single_pushes != Bitboard(0) {
            let index = single_pushes.pop_lsb();
            list.push(Move::new_unchecked(if self.ctm == 0 {index + 8} else {index - 8}, index, Flag::Normal as u8));
        }
        while double_pushes != Bitboard(0) {
            let index = double_pushes.pop_lsb();
            list.push(Move::new_unchecked(if self.ctm == 0 {index + 16} else {index - 16}, index, Flag::DoublePush as u8));
        }
        while pawn_push_promotions != Bitboard(0) {
            let index = pawn_push_promotions.pop_lsb();
            for i in 7..11 {
                list.push(Move::new_unchecked(if self.ctm == 0 {index + 8} else {index - 8}, index, i));
            }
        }

        let capturable: Bitboard = state.colors[1 - self.ctm as usize];
        let (mut left_captures, mut right_captures) = get_pawn_attacks_setwise(pawns, capturable, self.ctm);
        //let mut left_capture_promotions  = 
        //let mut right_capture_promotions =

    }
    #[must_use] pub fn in_check(&self) -> bool {
        self.square_attacked(self.states.last().expect("no state").king_sqs[self.ctm as usize])
    }
    #[must_use] pub fn square_attacked(&self, sq: Square) -> bool {
        let opp = 1 - self.ctm as usize;

        let state = self.states.last().expect("no state");
        let occ = state.occupied();
        let opp_queens: Bitboard = state.pieces[Types::Queen as usize] & state.colors[opp];

        let mut mask: Bitboard = get_rook_attacks(sq, occ) & (opp_queens | (state.pieces[Types::Rook as usize] & state.colors[opp]));
        if mask != Bitboard(0) {
            return true
        }

        mask = get_bishop_attacks(sq, occ) & (opp_queens | (state.pieces[Types::Bishop as usize] & state.colors[opp]));
        if mask != Bitboard(0) {
            return true
        }

        mask = get_knight_attacks(sq) & (state.pieces[Types::Knight as usize] & state.colors[opp]);
        if mask != Bitboard(0) {
            return true
        }

        mask = get_pawn_attacks_lookup(sq, self.ctm) & (state.pieces[Types::Pawn as usize] & state.colors[opp]);
        if mask != Bitboard(0) {
            return true
        }

        mask = get_king_attacks(sq) & (state.pieces[Types::King as usize] & state.colors[self.ctm as usize]);
        if mask != Bitboard(0) {
            return true
        }

        false
    }
    #[must_use] pub fn evaluate(&self) -> i32 {
        self.states.last().expect("no state").eval * (-1 + i32::from(self.ctm) * 2)
    }
}