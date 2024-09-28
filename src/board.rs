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
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
];

pub const ROOK_RIGHT_MASKS: [u8; 64] = [
    0b1101, 255, 255, 255, 255, 255, 255, 0b1110, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    0b0111, 255, 255, 255, 255, 255, 255, 0b1011,
];
pub const KING_RIGHT_MASKS: [u8; 2] = [0b0011, 0b1100];

use crate::{
    hash::{zobrist_ctm, zobrist_psq},
    movegen::{
        lookups::DIRECTIONAL_OFFSETS,
        others::{get_king_attacks, get_knight_attacks},
        pawns::{get_pawn_attacks_lookup, get_pawn_attacks_setwise, get_pawn_pushes_setwise},
        slideys::{get_bishop_attacks, get_rook_attacks},
    },
    nets::{policy::get_score, value::ValueNetworkState},
    rays::{ray_between, ray_intersecting},
    types::{
        bitboard::Bitboard,
        moves::{Flag, Move},
        piece::{Colors, Piece, Types},
        square::Square,
        MoveList,
    },
};

#[derive(Debug, Copy, Clone)]
pub struct Position {
    colors: [Bitboard; 2],
    pieces: [Bitboard; 6],
    mailbox: [Piece; 64],
    king_sqs: [Square; 2],
    hash: u64,
    pub ep_index: Square,
    pub hm_clock: u8,
    pub castling: u8,
    checkers: Bitboard,
    diago_pin_mask: Bitboard,
    ortho_pin_mask: Bitboard,
}

impl Position {
    #[must_use]
    pub const fn empty() -> Self {
        let col: [Bitboard; 2] = [Bitboard::EMPTY; 2];
        let pcs: [Bitboard; 6] = [Bitboard::EMPTY; 6];
        let mail: [Piece; 64] = [Piece(Types::None as u8); 64];
        let ksqs: [Square; 2] = [Square(64); 2];
        let epsq: Square = Square(64);
        let hmc: u8 = 0;
        let ca: u8 = 0;

        Self {
            colors: col,
            pieces: pcs,
            mailbox: mail,
            king_sqs: ksqs,
            ep_index: epsq,
            hm_clock: hmc,
            castling: ca,
            checkers: Bitboard::EMPTY,
            diago_pin_mask: Bitboard::EMPTY,
            ortho_pin_mask: Bitboard::EMPTY,
            hash: 0,
        }
    }
    pub fn add_piece(&mut self, sq: Square, piece: Piece) {
        let bitboard_square: Bitboard = Bitboard::from_square(sq);
        self.colors[piece.color() as usize] ^= bitboard_square;
        self.pieces[piece.piece() as usize] ^= bitboard_square;
        self.mailbox[sq.as_usize()] = piece;
        self.hash ^= zobrist_psq(piece, sq);
    }
    pub fn remove_piece(&mut self, sq: Square, piece: Piece) {
        let bitboard_square: Bitboard = Bitboard::from_square(sq);
        self.colors[piece.color() as usize] ^= bitboard_square;
        self.pieces[piece.piece() as usize] ^= bitboard_square;
        self.mailbox[sq.as_usize()] = Piece(Types::None as u8);
        self.hash ^= zobrist_psq(piece, sq);
    }
    pub fn move_piece(&mut self, from: Square, piece: Piece, to: Square, victim: Piece) {
        if victim.piece() != Types::None as u8 {
            self.remove_piece(to, victim);
        }
        self.remove_piece(from, piece);
        self.add_piece(to, piece);
    }
    #[must_use]
    pub const fn piece_on_square(&self, sq: Square) -> Piece {
        self.mailbox[sq.as_usize()]
    }
    #[must_use]
    pub fn occupied(&self) -> Bitboard {
        self.colors[0] | self.colors[1]
    }
    #[must_use]
    pub fn colored_piece(&self, piece: u8, color: u8) -> Bitboard {
        self.colors[color as usize] & self.pieces[piece as usize]
    }
    pub fn switch_color(&mut self) {
        self.hash ^= zobrist_ctm();
    }
}

#[derive(Debug, Clone, Default)]
pub struct Board {
    pub states: Vec<Position>,
    pub ctm: u8,
    ply: i16,
    // phase: i8
}

impl Board {
    #[must_use]
    pub fn new() -> Self {
        Self {
            states: vec![Position::empty(); 256],
            ctm: 0,
            ply: 0,
        }
    }
    pub fn load_state(&mut self, position: &Position, ctm: u8) {
        self.states.clear();
        self.states.push(*position);
        self.ctm = ctm;
    }
    pub fn print_state(&self) {
        let state = self.current_state();
        for i in (0..8).rev() {
            for j in 0..8 {
                print!("{} ", (state.mailbox[i * 8 + j]));
            }
            println!();
        }
        println!("hash: {}", state.hash);
        println!("ctm: {}", self.ctm);
        println!("is_drawn: {}", self.is_drawn())
    }
    #[allow(clippy::cast_possible_truncation)]
    pub fn load_fen(&mut self, fen: &str) {
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
                        state.add_piece(
                            i,
                            Piece::new_unchecked(Types::Pawn as u8, Colors::Black as u8),
                        );
                        i += Square(1);
                    }
                    'P' => {
                        state.add_piece(
                            i,
                            Piece::new_unchecked(Types::Pawn as u8, Colors::White as u8),
                        );
                        i += Square(1);
                    }
                    'n' => {
                        state.add_piece(
                            i,
                            Piece::new_unchecked(Types::Knight as u8, Colors::Black as u8),
                        );
                        i += Square(1);
                    }
                    'N' => {
                        state.add_piece(
                            i,
                            Piece::new_unchecked(Types::Knight as u8, Colors::White as u8),
                        );
                        i += Square(1);
                    }
                    'b' => {
                        state.add_piece(
                            i,
                            Piece::new_unchecked(Types::Bishop as u8, Colors::Black as u8),
                        );
                        i += Square(1);
                    }
                    'B' => {
                        state.add_piece(
                            i,
                            Piece::new_unchecked(Types::Bishop as u8, Colors::White as u8),
                        );
                        i += Square(1);
                    }
                    'r' => {
                        state.add_piece(
                            i,
                            Piece::new_unchecked(Types::Rook as u8, Colors::Black as u8),
                        );
                        i += Square(1);
                    }
                    'R' => {
                        state.add_piece(
                            i,
                            Piece::new_unchecked(Types::Rook as u8, Colors::White as u8),
                        );
                        i += Square(1);
                    }
                    'q' => {
                        state.add_piece(
                            i,
                            Piece::new_unchecked(Types::Queen as u8, Colors::Black as u8),
                        );
                        i += Square(1);
                    }
                    'Q' => {
                        state.add_piece(
                            i,
                            Piece::new_unchecked(Types::Queen as u8, Colors::White as u8),
                        );
                        i += Square(1);
                    }
                    'k' => {
                        state.add_piece(
                            i,
                            Piece::new_unchecked(Types::King as u8, Colors::Black as u8),
                        );
                        state.king_sqs[0] = i;
                        i += Square(1);
                    }
                    'K' => {
                        state.add_piece(
                            i,
                            Piece::new_unchecked(Types::King as u8, Colors::White as u8),
                        );
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
        if self.ctm == 1 {
            state.switch_color()
        };

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
        if token == "-" {
            state.ep_index = Square(64);
        } else {
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
            self.ply = token_option
                .expect("why would you have a 5th token but not a 6th")
                .parse()
                .unwrap();
        }

        self.states.push(state);
        self.update_pins_and_checkers();
    }
    pub fn get_moves(&self, list: &mut MoveList) {
        let state = self.current_state();
        let occ: Bitboard = state.occupied();
        let checkers: Bitboard = state.checkers;
        let num_checkers = checkers.popcount();
        let mut us: Bitboard = if num_checkers == 2 {
            state.colored_piece(5, self.ctm)
        } else {
            state.colors[self.ctm as usize]
        };
        let empties: Bitboard = !occ;

        if (state.castling & KING_RIGHT_MASKS[1 - self.ctm as usize]) != 0 && num_checkers == 0 {
            if self.ctm == 1 {
                if (state.castling & 1) != 0
                    && (occ & Bitboard(0x60)).is_empty()
                    && !self.square_attacked(Square(5))
                    && !self.square_attacked(Square(6))
                {
                    list.push(Move::new_unchecked(4, 6, Flag::WKCastle as u8));
                }
                if (state.castling & 2) != 0
                    && (occ & Bitboard(0xE)).is_empty()
                    && !self.square_attacked(Square(3))
                    && !self.square_attacked(Square(2))
                {
                    list.push(Move::new_unchecked(4, 2, Flag::WQCastle as u8));
                }
            } else {
                if (state.castling & 4) != 0
                    && (occ & Bitboard(0x6000_0000_0000_0000)).is_empty()
                    && !self.square_attacked(Square(61))
                    && !self.square_attacked(Square(62))
                {
                    list.push(Move::new_unchecked(60, 62, Flag::BKCastle as u8));
                }
                if (state.castling & 8) != 0
                    && (occ & Bitboard(0x0E00_0000_0000_0000)).is_empty()
                    && !self.square_attacked(Square(59))
                    && !self.square_attacked(Square(58))
                {
                    list.push(Move::new_unchecked(60, 58, Flag::BQCastle as u8));
                }
            }
        }
        while !us.is_empty() {
            let index = us.pop_lsb();
            let piece = state.piece_on_square(Square(index));
            let is_diag_pin =
                (state.diago_pin_mask & Bitboard::from_square(Square(index))).is_not_empty();
            let is_ortho_pin =
                (state.ortho_pin_mask & Bitboard::from_square(Square(index))).is_not_empty();
            let is_pinned = is_diag_pin || is_ortho_pin;
            let mut current_attack: Bitboard = match piece.piece() {
                // pawns (we do them setwise later)
                0 => Bitboard::EMPTY,
                // knights
                1 => {
                    if is_pinned {
                        continue;
                    }
                    get_knight_attacks(Square(index))
                }
                // bishops
                2 => {
                    if is_pinned {
                        if is_diag_pin {
                            get_bishop_attacks(Square(index), occ) & state.diago_pin_mask
                        } else {
                            continue;
                        }
                    } else {
                        get_bishop_attacks(Square(index), occ)
                    }
                }
                // rooks
                3 => {
                    if is_pinned {
                        if is_ortho_pin {
                            get_rook_attacks(Square(index), occ) & state.ortho_pin_mask
                        } else {
                            continue;
                        }
                    } else {
                        get_rook_attacks(Square(index), occ)
                    }
                }
                // queens
                4 => {
                    if is_pinned {
                        if is_diag_pin {
                            get_bishop_attacks(Square(index), occ) & state.diago_pin_mask
                        } else if is_ortho_pin {
                            get_rook_attacks(Square(index), occ) & state.ortho_pin_mask
                        } else {
                            panic!("invalid pin state")
                        }
                    } else {
                        get_bishop_attacks(Square(index), occ)
                            | get_rook_attacks(Square(index), occ)
                    }
                }
                // kings
                5 => {
                    let mut potential_moves = get_king_attacks(Square(index));
                    let mut checkers_clone = checkers;
                    while checkers_clone.is_not_empty() {
                        let checker = Square(checkers_clone.pop_lsb());
                        let checker_piece = state.piece_on_square(checker).piece();
                        if let 2..=4 = checker_piece {
                            potential_moves &= !(ray_intersecting(Square(index), checker)
                                & !Bitboard::from_square(checker))
                        }
                    }
                    potential_moves
                }
                _ => panic!("invalid piece, value of {}", piece.piece()),
            };
            // make sure you can't capture your own pieces
            current_attack ^= current_attack & state.colors[self.ctm as usize];
            // if not king
            if piece.piece() != 5 && num_checkers == 1 {
                let checker = Square(checkers.lsb());
                // make sure move blocks check properly
                current_attack &= ray_between(state.king_sqs[self.ctm as usize], checker)
                    | Bitboard::from_square(checker);
            }
            // not kings
            if piece.piece() != 5 {
                // convert it into moves
                while current_attack.is_not_empty() {
                    let to = current_attack.pop_lsb();
                    list.push(Move::new_unchecked(index, to, Flag::Normal as u8));
                }
            } else {
                let kingless_occ = occ ^ Bitboard::from_square(Square(index));
                // convert it into moves
                while current_attack.is_not_empty() {
                    let to = current_attack.pop_lsb();
                    if !self.square_attacked_occ(Square(to), kingless_occ) {
                        list.push(Move::new_unchecked(index, to, Flag::Normal as u8));
                    }
                }
            }
        }
        if num_checkers != 2 {
            // setwise pawns
            let pawns = state.pieces[Types::Pawn as usize] & state.colors[self.ctm as usize];
            let (mut single_pushes, mut double_pushes) = get_pawn_pushes_setwise(
                pawns,
                empties,
                self.ctm,
                state.ortho_pin_mask,
                state.diago_pin_mask,
            );
            if num_checkers == 1 {
                let checker = Square(checkers.lsb());
                // make sure move blocks check properly
                let mask = ray_between(state.king_sqs[self.ctm as usize], checker)
                    | Bitboard::from_square(checker);
                single_pushes &= mask;
                double_pushes &= mask;
            }
            // identify promotions
            let mut pawn_push_promotions = single_pushes & Bitboard::from_rank(7 * self.ctm);
            single_pushes ^= pawn_push_promotions;
            while single_pushes.is_not_empty() {
                let index = single_pushes.pop_lsb();
                list.push(Move::new_unchecked(
                    if self.ctm == 0 { index + 8 } else { index - 8 },
                    index,
                    Flag::Normal as u8,
                ));
            }
            while double_pushes.is_not_empty() {
                let index = double_pushes.pop_lsb();
                list.push(Move::new_unchecked(
                    if self.ctm == 0 {
                        index + 16
                    } else {
                        index - 16
                    },
                    index,
                    Flag::DoublePush as u8,
                ));
            }
            while pawn_push_promotions.is_not_empty() {
                let index = pawn_push_promotions.pop_lsb();
                for i in 7..11 {
                    list.push(Move::new_unchecked(
                        if self.ctm == 0 { index + 8 } else { index - 8 },
                        index,
                        i,
                    ));
                }
            }

            let capturable: Bitboard = state.colors[1 - self.ctm as usize];
            let (mut left_captures, mut right_captures) = get_pawn_attacks_setwise(
                pawns,
                capturable,
                self.ctm,
                state.ortho_pin_mask,
                state.diago_pin_mask,
            );
            if num_checkers == 1 {
                let checker = Square(checkers.lsb());
                // make sure move blocks check properly
                let mask = ray_between(state.king_sqs[self.ctm as usize], checker)
                    | Bitboard::from_square(checker);
                left_captures &= mask;
                right_captures &= mask;
            }
            let mut left_capture_promotions = left_captures & Bitboard::from_rank(7 * self.ctm);
            left_captures ^= left_capture_promotions;
            let mut right_capture_promotions = right_captures & Bitboard::from_rank(7 * self.ctm);
            right_captures ^= right_capture_promotions;

            while left_captures.is_not_empty() {
                let index = left_captures.pop_lsb();
                let start_square = if self.ctm == 0 { index + 9 } else { index - 7 };
                list.push(Move::new_unchecked(start_square, index, Flag::Normal as u8));
            }
            while right_captures.is_not_empty() {
                let index = right_captures.pop_lsb();
                let start_square = if self.ctm == 0 { index + 7 } else { index - 9 };
                list.push(Move::new_unchecked(start_square, index, Flag::Normal as u8));
            }
            while left_capture_promotions.is_not_empty() {
                let index = left_capture_promotions.pop_lsb();
                let start_square = if self.ctm == 0 { index + 9 } else { index - 7 };
                for i in 7..11 {
                    list.push(Move::new_unchecked(start_square, index, i));
                }
            }
            while right_capture_promotions.is_not_empty() {
                let index = right_capture_promotions.pop_lsb();
                let start_square = if self.ctm == 0 { index + 7 } else { index - 9 };
                for i in 7..11 {
                    list.push(Move::new_unchecked(start_square, index, i));
                }
            }
            if state.ep_index != Square::INVALID {
                let mut en_passanters = get_pawn_attacks_lookup(state.ep_index, 1 - self.ctm)
                    & pawns
                    & !state.ortho_pin_mask;
                while en_passanters.is_not_empty() {
                    let passanter = en_passanters.pop_lsb();
                    let post_ep_occ = occ
                        ^ Bitboard::from_square(Square(passanter))
                        ^ Bitboard::from_square(state.ep_index)
                        ^ Bitboard::from_square(Square(
                            (state.ep_index.0 as i8 + DIRECTIONAL_OFFSETS[self.ctm as usize]) as u8,
                        ));

                    let their_queens = state.colored_piece(4, 1 - self.ctm);
                    let their_rooks = their_queens | state.colored_piece(3, 1 - self.ctm);
                    let their_bishops = their_queens | state.colored_piece(2, 1 - self.ctm);

                    if (get_rook_attacks(state.king_sqs[self.ctm as usize], post_ep_occ)
                        & their_rooks)
                        .is_empty()
                        && (get_bishop_attacks(state.king_sqs[self.ctm as usize], post_ep_occ)
                            & their_bishops)
                            .is_empty()
                    {
                        list.push(Move::new_unchecked(
                            passanter,
                            state.ep_index.0,
                            Flag::EnPassant as u8,
                        ));
                    }
                }
            }
        }
    }
    pub fn get_move_count(&self) -> u64 {
        let mut sum = 0;
        let state = self.current_state();
        let occ: Bitboard = state.occupied();
        let checkers: Bitboard = state.checkers;
        let num_checkers = checkers.popcount();
        let mut us: Bitboard = if num_checkers == 2 {
            state.colored_piece(5, self.ctm)
        } else {
            state.colors[self.ctm as usize]
        };
        let empties: Bitboard = !occ;

        if (state.castling & KING_RIGHT_MASKS[1 - self.ctm as usize]) != 0 && num_checkers == 0 {
            if self.ctm == 1 {
                if (state.castling & 1) != 0
                    && (occ & Bitboard(0x60)).is_empty()
                    && !self.square_attacked(Square(5))
                    && !self.square_attacked(Square(6))
                {
                    sum += 1;
                }
                if (state.castling & 2) != 0
                    && (occ & Bitboard(0xE)).is_empty()
                    && !self.square_attacked(Square(3))
                    && !self.square_attacked(Square(2))
                {
                    sum += 1;
                }
            } else {
                if (state.castling & 4) != 0
                    && (occ & Bitboard(0x6000_0000_0000_0000)).is_empty()
                    && !self.square_attacked(Square(61))
                    && !self.square_attacked(Square(62))
                {
                    sum += 1;
                }
                if (state.castling & 8) != 0
                    && (occ & Bitboard(0x0E00_0000_0000_0000)).is_empty()
                    && !self.square_attacked(Square(59))
                    && !self.square_attacked(Square(58))
                {
                    sum += 1;
                }
            }
        }
        while !us.is_empty() {
            let index = us.pop_lsb();
            let piece = state.piece_on_square(Square(index));
            let is_diag_pin =
                (state.diago_pin_mask & Bitboard::from_square(Square(index))).is_not_empty();
            let is_ortho_pin =
                (state.ortho_pin_mask & Bitboard::from_square(Square(index))).is_not_empty();
            let is_pinned = is_diag_pin || is_ortho_pin;
            let mut current_attack: Bitboard = match piece.piece() {
                // pawns (we do them setwise later)
                0 => Bitboard::EMPTY,
                // knights
                1 => {
                    if is_pinned {
                        continue;
                    }
                    get_knight_attacks(Square(index))
                }
                // bishops
                2 => {
                    if is_pinned {
                        if is_diag_pin {
                            get_bishop_attacks(Square(index), occ) & state.diago_pin_mask
                        } else {
                            continue;
                        }
                    } else {
                        get_bishop_attacks(Square(index), occ)
                    }
                }
                // rooks
                3 => {
                    if is_pinned {
                        if is_ortho_pin {
                            get_rook_attacks(Square(index), occ) & state.ortho_pin_mask
                        } else {
                            continue;
                        }
                    } else {
                        get_rook_attacks(Square(index), occ)
                    }
                }
                // queens
                4 => {
                    if is_pinned {
                        if is_diag_pin {
                            get_bishop_attacks(Square(index), occ) & state.diago_pin_mask
                        } else if is_ortho_pin {
                            get_rook_attacks(Square(index), occ) & state.ortho_pin_mask
                        } else {
                            panic!("invalid pin state")
                        }
                    } else {
                        get_bishop_attacks(Square(index), occ)
                            | get_rook_attacks(Square(index), occ)
                    }
                }
                // kings
                5 => {
                    let mut potential_moves = get_king_attacks(Square(index));
                    let mut checkers_clone = checkers;
                    while checkers_clone.is_not_empty() {
                        let checker = Square(checkers_clone.pop_lsb());
                        let checker_piece = state.piece_on_square(checker).piece();
                        if let 2..=4 = checker_piece {
                            potential_moves &= !(ray_intersecting(Square(index), checker)
                                & !Bitboard::from_square(checker))
                        }
                    }
                    potential_moves
                }
                _ => panic!("invalid piece, value of {}", piece.piece()),
            };
            // make sure you can't capture your own pieces
            current_attack ^= current_attack & state.colors[self.ctm as usize];
            // if not king
            if piece.piece() != 5 && num_checkers == 1 {
                let checker = Square(checkers.lsb());
                // make sure move blocks check properly
                current_attack &= ray_between(state.king_sqs[self.ctm as usize], checker)
                    | Bitboard::from_square(checker);
            }
            // not kings
            if piece.piece() != 5 {
                sum += current_attack.popcount();
            } else {
                let kingless_occ = occ ^ Bitboard::from_square(Square(index));
                // convert it into moves
                while current_attack.is_not_empty() {
                    let to = current_attack.pop_lsb();
                    if !self.square_attacked_occ(Square(to), kingless_occ) {
                        sum += 1;
                    }
                }
            }
        }
        if num_checkers != 2 {
            // setwise pawns
            let pawns = state.pieces[Types::Pawn as usize] & state.colors[self.ctm as usize];
            let (mut single_pushes, mut double_pushes) = get_pawn_pushes_setwise(
                pawns,
                empties,
                self.ctm,
                state.ortho_pin_mask,
                state.diago_pin_mask,
            );
            if num_checkers == 1 {
                let checker = Square(checkers.lsb());
                // make sure move blocks check properly
                let mask = ray_between(state.king_sqs[self.ctm as usize], checker)
                    | Bitboard::from_square(checker);
                single_pushes &= mask;
                double_pushes &= mask;
            }
            // identify promotions
            let pawn_push_promotions = single_pushes & Bitboard::from_rank(7 * self.ctm);
            single_pushes ^= pawn_push_promotions;
            sum += single_pushes.popcount();
            sum += double_pushes.popcount();
            sum += pawn_push_promotions.popcount() * 4;

            let capturable: Bitboard = state.colors[1 - self.ctm as usize];
            let (mut left_captures, mut right_captures) = get_pawn_attacks_setwise(
                pawns,
                capturable,
                self.ctm,
                state.ortho_pin_mask,
                state.diago_pin_mask,
            );
            if num_checkers == 1 {
                let checker = Square(checkers.lsb());
                // make sure move blocks check properly
                let mask = ray_between(state.king_sqs[self.ctm as usize], checker)
                    | Bitboard::from_square(checker);
                left_captures &= mask;
                right_captures &= mask;
            }
            let left_capture_promotions = left_captures & Bitboard::from_rank(7 * self.ctm);
            left_captures ^= left_capture_promotions;
            let right_capture_promotions = right_captures & Bitboard::from_rank(7 * self.ctm);
            right_captures ^= right_capture_promotions;

            sum += left_captures.popcount();
            sum += right_captures.popcount();
            sum += left_capture_promotions.popcount() * 4;
            sum += right_capture_promotions.popcount() * 4;

            if state.ep_index != Square::INVALID {
                let mut en_passanters = get_pawn_attacks_lookup(state.ep_index, 1 - self.ctm)
                    & pawns
                    & !state.ortho_pin_mask;
                while en_passanters.is_not_empty() {
                    let passanter = en_passanters.pop_lsb();
                    let post_ep_occ = occ
                        ^ Bitboard::from_square(Square(passanter))
                        ^ Bitboard::from_square(state.ep_index)
                        ^ Bitboard::from_square(Square(
                            (state.ep_index.0 as i8 + DIRECTIONAL_OFFSETS[self.ctm as usize]) as u8,
                        ));

                    let their_queens = state.colored_piece(4, 1 - self.ctm);
                    let their_rooks = their_queens | state.colored_piece(3, 1 - self.ctm);
                    let their_bishops = their_queens | state.colored_piece(2, 1 - self.ctm);

                    if (get_rook_attacks(state.king_sqs[self.ctm as usize], post_ep_occ)
                        & their_rooks)
                        .is_empty()
                        && (get_bishop_attacks(state.king_sqs[self.ctm as usize], post_ep_occ)
                            & their_bishops)
                            .is_empty()
                    {
                        sum += 1;
                    }
                }
            }
        }
        sum as u64
    }
    pub fn make_move(&mut self, mov: Move) {
        self.states.push(*self.current_state());
        //                         not using self.current_state_mut() because of borrowing shenanigans
        let state = self.states.last_mut().expect("no position");

        let from = mov.from();
        let from_square = Square(from);
        let to = mov.to();
        let to_square = Square(to);
        let piece = state.piece_on_square(from_square);
        let victim = state.piece_on_square(to_square);
        let flag = mov.flag();
        let is_capture = victim.piece() != Types::None as u8;

        state.hm_clock += 1;
        if is_capture || piece.piece() == Types::Pawn as u8 {
            state.hm_clock = 0;
        }

        if piece.piece() == Types::King as u8 {
            state.king_sqs[self.ctm as usize] = to_square;
        }

        if state.castling & KING_RIGHT_MASKS[1 - self.ctm as usize] != 0 {
            match piece.piece() {
                3 => state.castling &= ROOK_RIGHT_MASKS[from as usize],
                5 => state.castling &= KING_RIGHT_MASKS[self.ctm as usize],
                _ => (),
            }
        }

        if victim.piece() == Types::Rook as u8 {
            state.castling &= ROOK_RIGHT_MASKS[to as usize];
        }

        state.ep_index = Square::INVALID;

        // pretty much inlined state.move_piece so that I can save an add+remove+add shenanigan for promotions
        if is_capture {
            state.remove_piece(to_square, victim);
        }
        state.remove_piece(from_square, piece);
        if flag < Flag::KnightPromo {
            state.add_piece(to_square, piece);
        }

        match flag {
            Flag::Normal => (),
            Flag::WKCastle => state.move_piece(
                Square(7),
                Piece::new_unchecked(Types::Rook as u8, Colors::White as u8),
                Square(5),
                Piece(Types::None as u8),
            ),
            Flag::WQCastle => state.move_piece(
                Square(0),
                Piece::new_unchecked(Types::Rook as u8, Colors::White as u8),
                Square(3),
                Piece(Types::None as u8),
            ),
            Flag::BKCastle => state.move_piece(
                Square(63),
                Piece::new_unchecked(Types::Rook as u8, Colors::Black as u8),
                Square(61),
                Piece(Types::None as u8),
            ),
            Flag::BQCastle => state.move_piece(
                Square(56),
                Piece::new_unchecked(Types::Rook as u8, Colors::Black as u8),
                Square(59),
                Piece(Types::None as u8),
            ),
            Flag::DoublePush => {
                state.ep_index = Square((to as i8 + DIRECTIONAL_OFFSETS[self.ctm as usize]) as u8)
            }
            Flag::EnPassant => state.remove_piece(
                Square((to as i8 + DIRECTIONAL_OFFSETS[self.ctm as usize]) as u8),
                Piece::new_unchecked(Types::Pawn as u8, 1 - self.ctm),
            ),
            Flag::KnightPromo => state.add_piece(
                to_square,
                Piece::new_unchecked(Types::Knight as u8, self.ctm),
            ),
            Flag::BishopPromo => state.add_piece(
                to_square,
                Piece::new_unchecked(Types::Bishop as u8, self.ctm),
            ),
            Flag::RookPromo => {
                state.add_piece(to_square, Piece::new_unchecked(Types::Rook as u8, self.ctm))
            }
            Flag::QueenPromo => state.add_piece(
                to_square,
                Piece::new_unchecked(Types::Queen as u8, self.ctm),
            ),
        }

        self.ply += 1;
        self.ctm = 1 - self.ctm;
        state.switch_color();
        self.update_pins_and_checkers();
    }
    pub fn undo_move(&mut self) {
        self.states.pop();
        self.ply -= 1;
        self.ctm = 1 - self.ctm;
    }
    #[must_use]
    pub fn in_check(&self) -> bool {
        !self
            .states
            .last()
            .expect("ahahahahah you messed up now")
            .checkers
            .is_empty()
    }
    #[must_use]
    pub fn square_attacked(&self, sq: Square) -> bool {
        let opp = 1 - self.ctm as usize;

        let state = self.current_state();
        let occ = state.occupied();
        let opp_queens: Bitboard = state.pieces[Types::Queen as usize] & state.colors[opp];

        let mut mask: Bitboard = get_rook_attacks(sq, occ)
            & (opp_queens | (state.pieces[Types::Rook as usize] & state.colors[opp]));
        if mask.is_not_empty() {
            return true;
        }

        mask = get_bishop_attacks(sq, occ)
            & (opp_queens | (state.pieces[Types::Bishop as usize] & state.colors[opp]));
        if mask.is_not_empty() {
            return true;
        }

        mask = get_knight_attacks(sq) & (state.pieces[Types::Knight as usize] & state.colors[opp]);
        if mask.is_not_empty() {
            return true;
        }

        mask = get_pawn_attacks_lookup(sq, self.ctm)
            & (state.pieces[Types::Pawn as usize] & state.colors[opp]);
        if mask.is_not_empty() {
            return true;
        }

        mask = get_king_attacks(sq) & (state.pieces[Types::King as usize] & state.colors[opp]);
        if mask.is_not_empty() {
            return true;
        }

        false
    }
    #[must_use]
    pub fn square_attacked_occ(&self, sq: Square, occ: Bitboard) -> bool {
        let opp = 1 - self.ctm as usize;

        let state = self.current_state();
        let opp_queens: Bitboard = state.pieces[Types::Queen as usize] & state.colors[opp];

        let mut mask: Bitboard = get_rook_attacks(sq, occ)
            & (opp_queens | (state.pieces[Types::Rook as usize] & state.colors[opp]));
        if mask.is_not_empty() {
            return true;
        }

        mask = get_bishop_attacks(sq, occ)
            & (opp_queens | (state.pieces[Types::Bishop as usize] & state.colors[opp]));
        if mask.is_not_empty() {
            return true;
        }

        mask = get_knight_attacks(sq) & (state.pieces[Types::Knight as usize] & state.colors[opp]);
        if mask.is_not_empty() {
            return true;
        }

        mask = get_pawn_attacks_lookup(sq, self.ctm)
            & (state.pieces[Types::Pawn as usize] & state.colors[opp]);
        if mask.is_not_empty() {
            return true;
        }

        mask = get_king_attacks(sq) & (state.pieces[Types::King as usize] & state.colors[opp]);
        if mask.is_not_empty() {
            return true;
        }

        false
    }
    pub fn current_state(&self) -> &Position {
        self.states.last().expect("No current state")
    }
    pub fn current_state_mut(&mut self) -> &mut Position {
        self.states.last_mut().expect("No current state")
    }
    #[must_use]
    pub fn evaluate(&self) -> i32 {
        let mut net = ValueNetworkState::new();
        net.evaluate(self.current_state(), self.ctm)
    }
    #[must_use]
    pub fn evaluate_non_stm(&self) -> i32 {
        let mut net = ValueNetworkState::new();
        net.evaluate(self.current_state(), 1)
    }
    pub fn get_policy(&self, mov: Move) -> f32 {
        get_score(self.current_state(), mov)
    }
    #[must_use]
    pub fn get_fen(&self) -> String {
        let mut fen: String = String::new();
        let state = self.current_state();
        for rank in (0..8).rev() {
            let mut num_empty_files = 0;
            for file in 0..8 {
                let piece = state.piece_on_square(Square(8 * rank + file));
                if piece != Piece(Types::None as u8) {
                    if num_empty_files != 0 {
                        fen += &num_empty_files.to_string();
                        num_empty_files = 0;
                    }
                    let is_white = piece.color() == 1;
                    let piece_type = piece.piece();
                    let mut piece_char: String = match piece_type {
                        0 => "p".to_owned(),
                        1 => "n".to_owned(),
                        2 => "b".to_owned(),
                        3 => "r".to_owned(),
                        4 => "q".to_owned(),
                        5 => "k".to_owned(),
                        _ => panic!("invalid piece type"),
                    };
                    if is_white {
                        piece_char = piece_char.to_uppercase()
                    }
                    fen += &piece_char;
                } else {
                    num_empty_files += 1;
                }
            }
            if num_empty_files != 0 {
                fen += &num_empty_files.to_string();
            }
            if rank != 0 {
                fen += "/";
            }
        }

        // color to move
        fen += " ";
        fen += match self.ctm {
            0 => "b",
            1 => "w",
            _ => panic!("invalid ctm"),
        };
        // castling rights
        fen += " ";
        let mut thing_added = false;
        if (state.castling & 1) != 0 {
            fen += "K";
            thing_added = true;
        }
        if (state.castling & 2) != 0 {
            fen += "Q";
            thing_added = true;
        }
        if (state.castling & 4) != 0 {
            fen += "k";
            thing_added = true;
        }
        if (state.castling & 8) != 0 {
            fen += "q";
            thing_added = true;
        }
        if !thing_added {
            fen += "-"
        }

        // en passant square
        fen += " ";
        if state.ep_index == Square::INVALID {
            fen += "-";
        } else {
            fen += SQUARE_NAMES[state.ep_index.0 as usize];
        }
        // nobody cares about 50mr or the other thing right???
        fen
    }
    fn get_attackers(&self, sq: Square) -> Bitboard {
        let state = self.current_state();
        let occupied = state.occupied();
        let opp = 1 - self.ctm;
        let opp_queens = state.colored_piece(4, opp);
        (get_pawn_attacks_lookup(sq, self.ctm) & state.colored_piece(0, opp))
            | (get_knight_attacks(sq) & state.colored_piece(1, opp))
            | (get_bishop_attacks(sq, occupied) & (state.colored_piece(2, opp) | opp_queens))
            | (get_rook_attacks(sq, occupied) & (state.colored_piece(3, opp) | opp_queens))
            | (get_king_attacks(sq) & state.colored_piece(5, opp))
    }
    fn update_pins_and_checkers(&mut self) {
        // info gathering
        let us_idx = self.ctm as usize;
        let opp = 1 - self.ctm;
        let opp_idx = opp as usize;
        let state = self.current_state();
        let king = state.king_sqs[us_idx];

        // while the other engines were playing chess, ANURA WAS PLAYING CHECKERS
        let checkers = self.get_attackers(king);
        let state = self.current_state_mut();
        state.checkers = checkers;

        // more info gathering
        // the ordering above was chosen because it fixes shenanigans with mutability vs immutability
        let us = state.colors[us_idx];
        let them = state.colors[opp_idx];
        let opp_queens = state.colored_piece(4, opp);
        let opp_diago = opp_queens | state.colored_piece(2, opp);
        let opp_ortho = opp_queens | state.colored_piece(3, opp);
        let mut potential_diago_pinners = opp_diago & get_bishop_attacks(king, them);
        let mut potential_ortho_pinners = opp_ortho & get_rook_attacks(king, them);
        state.diago_pin_mask = Bitboard::EMPTY;
        state.ortho_pin_mask = Bitboard::EMPTY;

        while potential_diago_pinners.is_not_empty() {
            let pinner = Square(potential_diago_pinners.pop_lsb());
            let potentially_pinned = ray_between(king, pinner) | Bitboard::from_square(pinner);
            if (potentially_pinned & us).contains_one() {
                state.diago_pin_mask |= potentially_pinned;
            }
        }

        while potential_ortho_pinners.is_not_empty() {
            let pinner = Square(potential_ortho_pinners.pop_lsb());
            let potentially_pinned = ray_between(king, pinner) | Bitboard::from_square(pinner);
            if (potentially_pinned & us).contains_one() {
                state.ortho_pin_mask |= potentially_pinned;
            }
        }
    }
    pub fn is_drawn(&self) -> bool {
        let state = self.current_state();

        if state.hm_clock >= 100 {
            return true;
        }

        for other_state in self
            .states
            .iter()
            .rev()
            .take(state.hm_clock as usize + 1)
            .skip(2)
            .step_by(2)
        {
            if other_state.hash == state.hash {
                return true;
            }
        }

        false
    }
}
