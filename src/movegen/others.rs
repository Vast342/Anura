use crate::types::bitboard::Bitboard;
use crate::types::square::Square;
use super::movegen_lookups::{KING_ATTACKS, KNIGHT_ATTACKS};

// Knights
pub fn get_knight_attacks(sq: Square) -> Bitboard {
    Bitboard(KNIGHT_ATTACKS[sq.as_usize()])
}

// Kings
pub fn get_king_attacks(sq: Square) -> Bitboard {
    Bitboard(KING_ATTACKS[sq.as_usize()])
}