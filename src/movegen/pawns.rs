use crate::types::bitboard::Bitboard;
use crate::types::square::Square;
use super::movegen_lookups::PAWN_ATTACKS;

// input: pawn bitboard, unoccupied bitboard, and ctm
// output: tuple of single and double pawn pushes
pub fn get_pawn_pushes_setwise(pawns: Bitboard, empties: Bitboard, ctm: u8) -> (Bitboard, Bitboard) {
    let single: Bitboard;
    if ctm == 0 {
        single = (pawns >> 8) & empties;
    } else {
        single = (pawns << 8) & empties;
    }
    let mut double = single & Bitboard::from_rank(if ctm == 1 { 2 } else { 5 });
    if ctm == 0 {
        double = (double >> 8) & empties;
    } else {
        double = (double << 8) & empties;
    }
    (single, double)
}

// input: pawn bitboard, opponent bitboard, and ctm
// output: tuple of left and right pawn pushes
pub fn get_pawn_attacks_setwise() -> (Bitboard, Bitboard) {
    let left_attacks = Bitboard(0);
    let right_attacks= Bitboard(0);
    (left_attacks, right_attacks)
}

// single square pawn capture lookups
pub fn get_pawn_attacks_lookup(sq: Square, ctm: u8) -> Bitboard {
    Bitboard(PAWN_ATTACKS[ctm as usize][sq.as_usize()])
}