use crate::types::bitboard::Bitboard;
use crate::types::square::Square;

use super::movegen_lookups::SLIDEY_PIECE_RAYS;

// slidey pieces

// Rooks
// Classical Approach
pub fn get_rook_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    let mut total_attacks: Bitboard = Bitboard(0);
    for dir in 0..4 {
        let mut current_attack: Bitboard = Bitboard(SLIDEY_PIECE_RAYS[dir][sq.as_usize()]);
        if (dir & 1) == 0 {
            current_attack ^= Bitboard(SLIDEY_PIECE_RAYS[dir][(current_attack & occupied).lsb() as usize])
        } else {
            current_attack ^= Bitboard(SLIDEY_PIECE_RAYS[dir][63 - (current_attack & occupied).msb() as usize])
        }
        total_attacks |= current_attack;
    }
    total_attacks
}
// Magics

// Pext?

// Bishops
// Classical Approach
pub fn get_bishop_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    let mut total_attacks: Bitboard = Bitboard(0);
    for dir in 4..8 {
        let mut current_attack: Bitboard = Bitboard(SLIDEY_PIECE_RAYS[dir][sq.as_usize()]);
        if (dir & 1) == 0 {
            current_attack ^= Bitboard(SLIDEY_PIECE_RAYS[dir][(current_attack & occupied).lsb() as usize])
        } else {
            current_attack ^= Bitboard(SLIDEY_PIECE_RAYS[dir][63 - (current_attack & occupied).msb() as usize])
        }
        total_attacks |= current_attack;
    }
    total_attacks
}
// Magics

// Pext?
