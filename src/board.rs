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
pub enum Piece {
    Pawn, Knight, Bishop, Rook, Queen, King, None
}
pub const BLACK: u8 = 0;
pub const WHITE: u8 = 8;

pub struct BoardState {
    fn null_state()
    colors:   [u64; 2],
    pieces:   [u64; 6],
    mailbox:  [u8; 64],
    zobrist:   u64,
    king_sqs: [u8; 2],
    ep_index:  u8,
    hm_clock:  u8,
    castling:  u8,
}

pub struct Board {

}


impl Board {
    fn from_fen(&self, fen: String) -> Board {
        let mut colors:   [u64; 2];
        let mut pieces:   [u64; 6];
        let mut mailbox:  [u8; 64];
        let mut zobrist:   u64;
        let mut king_sqs: [u8; 2];
        let mut ep_index:  u8;
        let mut hm_clock:  u8;
        let mut castling:  u8;
    }
}