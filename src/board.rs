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

use crate::types::bitboard::Bitboard;
use crate::types::piece::Piece;
//use std::vec;

#[derive(Debug, Copy, Clone)]
pub struct BoardState {
    colors:   [Bitboard; 2],
    pieces:   [Bitboard; 6],
    mailbox:  [Piece; 64],
    zobrist:   u64,
    king_sqs: [u8; 2],
    ep_index:  u8,
    hm_clock:  u8,
    castling:  u8,
}

impl BoardState {
    pub fn empty() -> Self {
        let c: [Bitboard; 2] = [Bitboard(0); 2];
        let p: [Bitboard; 6] = [Bitboard(0); 6];
        let m: [Piece; 64] = [Piece(0); 64];
        let z: u64 = 0;
        let k: [u8; 2] = [0; 2];
        let e: u8 = 0;
        let h: u8 = 0;
        let ca: u8 = 0;

        Self {colors: c, pieces: p, mailbox: m, zobrist: z, king_sqs: k, ep_index: e, hm_clock: h, castling: ca}
    }
}

#[derive(Debug, Clone, Default)]
pub struct Board {
    states: Vec<BoardState>,
}


impl Board {
    pub fn new() -> Self {
        Self {states: vec![BoardState::empty(); 256]}
    }
    pub fn load_fen(mut self, _fen: String) {
        let /*mut*/ state: BoardState = BoardState::empty();
        
        self.states.push(state);
    }
}