

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
    eval::PIECE_WEIGHTS, movegen::{others::{get_king_attacks, get_knight_attacks}, pawns::{get_pawn_attacks_lookup, get_pawn_pushes_setwise}, slideys::{get_bishop_attacks, get_rook_attacks}}, types::{
        bitboard::Bitboard, moves::{Flag, Move}, piece::{
            Colors, Piece, Types
        }, square::Square, MoveList
    }
};

//use std::vec;

#[derive(Debug, Copy, Clone)]
pub struct BoardState {
    colors:   [Bitboard; 2],
    pieces:   [Bitboard; 6],
    mailbox:  [Piece; 64],
    eval: i32,
    king_sqs: [Square; 2],
    ep_index:  Square,
    hm_clock:  u8,
    castling:  u8,
}

impl BoardState {
    pub fn empty() -> Self {
        let c: [Bitboard; 2] = [Bitboard(0); 2];
        let p: [Bitboard; 6] = [Bitboard(0); 6];
        let m: [Piece; 64] = [Piece(Types::None as u8); 64];
        let k: [Square; 2] = [Square(64); 2];
        let e: Square = Square(64);
        let h: u8 = 0;
        let ca: u8 = 0;
        let ev: i32 = 0;

        Self {colors: c, pieces: p, mailbox: m, king_sqs: k, ep_index: e, hm_clock: h, castling: ca, eval: ev}
    }
    pub fn add_piece(&mut self, sq: Square, piece: Piece) {
        let bitboard_square: Bitboard = Bitboard::from_square(sq);
        self.colors[piece.color() as usize] ^= bitboard_square;
        self.pieces[piece.piece() as usize] ^= bitboard_square;
        self.mailbox[sq.as_usize()] = piece;
        self.eval += PIECE_WEIGHTS[piece.piece() as usize] * (-1 + (piece.color() as i32) * 2);
    }
    pub fn remove_piece(&mut self, sq: Square, piece: Piece) {
        let bitboard_square: Bitboard = Bitboard::from_square(sq);
        self.colors[piece.color() as usize] ^= bitboard_square;
        self.pieces[piece.piece() as usize] ^= bitboard_square;
        self.mailbox[sq.as_usize()] = Piece(Types::None as u8);
        self.eval -= PIECE_WEIGHTS[piece.piece() as usize] * (-1 + (piece.color() as i32) * 2);
    }
    pub fn piece_on_square(&self, sq: Square) -> Piece {
        self.mailbox[sq.as_usize()]
    }
    pub fn occupied(&self) -> Bitboard {
        self.colors[0] & self.colors[1]
    }
}

#[derive(Debug, Clone, Default)]
pub struct Board {
    states: Vec<BoardState>,
    ctm: u8,
    ply: i16,
    // phase: i8
}

impl Board {
    pub fn new() -> Self {
        Self {states: vec![BoardState::empty(); 256], ctm: 0, ply: 0}
    }
    pub fn print_state(&self) {
        for i in (0..8).rev() {
            for j in 0..8 {
                print!("{} ", (self.states.last().expect("no state???").mailbox[i*8+j]));
            }
            println!();
        }
    }
    pub fn load_fen(&mut self, fen: &str) {
        let mut state: BoardState = BoardState::empty();
        let mut fen_split = fen.split_ascii_whitespace();
        // first token: position
        let mut token = fen_split.next().expect("no position?");
        let mut ranks = token.rsplit("/");
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
        self.ctm = if token == "w" { 1 } else { 0 };

        // third token: castling rights
        token = fen_split.next().expect("no castling rights?");
        for c in token.chars() { 
            match c {
                'K' => state.castling |= 1,
                'Q' => state.castling |= 2,
                'k' => state.castling |= 4,
                'q' => state.castling |= 8,
                '-' => (),
                ' ' => (),
                _ => assert!(false, "invalid castling rights (Anura doesn't support frc), you gave {}", c),
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
        if token_option != None {
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
        if (state.castling & KING_RIGHT_MASKS[1 - self.ctm as usize]) != 0 {
            if !self.in_check() {
                if (state.castling & 1) != 0 && (occ & Bitboard(0x60) == Bitboard(0)) && !self.square_attacked(Square(5)) {
                    list.push(Move::new_unchecked(4, 6, Flag::WKCastle as u8))
                }
                if (state.castling & 2) != 0 && (occ & Bitboard(0xE) == Bitboard(0)) && !self.square_attacked(Square(3)) {
                    list.push(Move::new_unchecked(4, 2, Flag::WQCastle as u8))
                }
                if (state.castling & 4) != 0 && (occ & Bitboard(0x6000000000000000) == Bitboard(0)) && !self.square_attacked(Square(61)) {
                    list.push(Move::new_unchecked(60, 62, Flag::BKCastle as u8))
                }
                if (state.castling & 8) != 0 && (occ & Bitboard(0xE00000000000000) == Bitboard(0)) && !self.square_attacked(Square(59)) {
                    list.push(Move::new_unchecked(60, 58, Flag::BQCastle as u8))
                } 
            }
        }
        while us != Bitboard(0) {
            let index = Square(us.pop_lsb());
            let piece = state.piece_on_square(index);
            let current_attack: Bitboard = match piece.piece() {
                // pawns (we do them setwise later)
                0 => Bitboard(0),
                // knights
                1 => {
                    get_knight_attacks(index)
                },
                // bishops
                2 => {
                    get_bishop_attacks(index, occ)
                },
                // rooks
                3 => {
                    get_rook_attacks(index, occ)
                },
                // queens
                4 => {
                    get_bishop_attacks(index, occ) | get_rook_attacks(index, occ)
                },
                // kings
                5 => {
                    get_king_attacks(index)
                },
                _ => panic!("invalid piece, value of {}", piece.piece()),
            }
            // make sure you can't capture your own pieces

            // convert it into moves
            while current_attack != Bitboard(0) {
                
            }
            
        }
        let mut pawn_pushes = get_pawn_pushes_setwise(state.pieces[Types::Pawn as usize], empties, self.ctm);
        while pawn_pushes.0 != Bitboard(0) {
            let index = pawn_pushes.0.pop_lsb();
            list.push(Move::new_unchecked(if self.ctm == 0 {} else {}, index, Flag::Normal as u8));
        }
        while pawn_pushes.1 != Bitboard(0) {
            let index = pawn_pushes.1.pop_lsb();
            list.push(Move::new_unchecked(if self.ctm == 0 {} else {}, index, Flag::Normal as u8));
        }
    }
    pub fn in_check(&self) -> bool {
        self.square_attacked(self.states.last().expect("no state").king_sqs[self.ctm as usize])
    }
    pub fn square_attacked(&self, sq: Square) -> bool {
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
    pub fn evaluate(&self) -> i32 {
        self.states.last().expect("no state").eval * (-1 + (self.ctm as i32) * 2)
    }
}