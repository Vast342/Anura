/*
    This file is from Ciekce's rust MCTS engine voidstar, and adapted to work with my structs
    Original code is licensed under GPL 3.0, same as Anura, and is used with permission.
*/

use crate::movegen::slideys::{get_bishop_attacks, get_rook_attacks};
use crate::types::bitboard::Bitboard;
use crate::types::square::Square;

pub static mut BETWEEN_RAYS: [[Bitboard; 64]; 64] = [[Bitboard::EMPTY; 64]; 64];

pub fn init_between() {
    for src_idx in 0..64 {
        let src = Square(src_idx as u8);
        let src_mask = Bitboard::from_square(src);

        let rook_attacks = get_rook_attacks(src, Bitboard::EMPTY);
        let bishop_attacks = get_bishop_attacks(src, Bitboard::EMPTY);
        unsafe {
            for dst_idx in 0..64 {
                if src_idx == dst_idx {
                    BETWEEN_RAYS[src_idx][dst_idx] = Bitboard::EMPTY;
                } else {
                    let dst = Square(dst_idx as u8);
                    let dst_mask = Bitboard::from_square(dst);

                    if rook_attacks & dst_mask != Bitboard::EMPTY {
                        BETWEEN_RAYS[src_idx][dst_idx] = get_rook_attacks(src, dst_mask) & get_rook_attacks(dst, src_mask);
                    } else if bishop_attacks & dst_mask != Bitboard::EMPTY {
                        BETWEEN_RAYS[src_idx][dst_idx] = get_bishop_attacks(src, dst_mask) & get_rook_attacks(dst, src_mask);
                    } else {
                        BETWEEN_RAYS[src_idx][dst_idx] = Bitboard::EMPTY;
                    }
                }
            }
        }
    }
}
#[must_use] pub fn ray_between(a: Square, b: Square) -> Bitboard {
    unsafe { BETWEEN_RAYS[a.as_usize()][b.as_usize()] }
}

/*
const INTERSECTING_RAYS: [[Bitboard; 64]; 64] = array_init!(|src_idx, 64| {
    let src = Square::from_raw(src_idx as u8);
    let src_mask = src.bit();

    let rook_attacks = attacks::rook_attacks(src, Bitboard::EMPTY);
    let bishop_attacks = attacks::bishop_attacks(src, Bitboard::EMPTY);

    array_init!(|dst_idx, 64| {
        if src_idx == dst_idx {
            Bitboard::EMPTY
        } else {
            let dst = Square::from_raw(dst_idx as u8);
            let dst_mask = dst.bit();

            if rook_attacks.get(dst) {
                src_mask
                    .or(attacks::rook_attacks(src, Bitboard::EMPTY))
                    .and(dst_mask.or(attacks::rook_attacks(dst, Bitboard::EMPTY)))
            } else if bishop_attacks.get(dst) {
                src_mask
                    .or(attacks::bishop_attacks(src, Bitboard::EMPTY))
                    .and(dst_mask.or(attacks::bishop_attacks(dst, Bitboard::EMPTY)))
            } else {
                Bitboard::EMPTY
            }
        }
    })
});

pub const fn ray_intersecting(a: Square, b: Square) -> Bitboard {
    INTERSECTING_RAYS[a.as_usize()][b.as_usize()]
}
*/