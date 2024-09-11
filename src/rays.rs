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
                BETWEEN_RAYS[src_idx][dst_idx] = if src_idx == dst_idx {
                    Bitboard::EMPTY
                } else {
                    let dst = Square(dst_idx as u8);
                    let dst_mask = Bitboard::from_square(dst);

                    if (rook_attacks & dst_mask).is_not_empty() {
                        get_rook_attacks(src, dst_mask) & get_rook_attacks(dst, src_mask)
                    } else if (bishop_attacks & dst_mask).is_not_empty() {
                        get_bishop_attacks(src, dst_mask) & get_bishop_attacks(dst, src_mask)
                    } else {
                        Bitboard::EMPTY
                    }
                }
            }
        }
    }
    //unsafe { dbg!(BETWEEN_RAYS[60][39]) };
}

#[must_use]
pub fn ray_between(a: Square, b: Square) -> Bitboard {
    unsafe { BETWEEN_RAYS[a.as_usize()][b.as_usize()] }
}

static mut INTERSECTING_RAYS: [[Bitboard; 64]; 64] = [[Bitboard::EMPTY; 64]; 64];

pub fn init_intersection() {
    for src_idx in 0..64 {
        let src = Square(src_idx as u8);
        let src_mask = Bitboard::from_square(src);

        let rook_attacks = get_rook_attacks(src, Bitboard::EMPTY);
        let bishop_attacks = get_bishop_attacks(src, Bitboard::EMPTY);
        unsafe {
            for dst_idx in 0..64 {
                INTERSECTING_RAYS[src_idx][dst_idx] = if src_idx == dst_idx {
                    Bitboard::EMPTY
                } else {
                    let dst = Square(dst_idx as u8);
                    let dst_mask = Bitboard::from_square(dst);

                    if (rook_attacks & dst_mask).is_not_empty() {
                        src_mask
                            | get_rook_attacks(src, Bitboard::EMPTY)
                                & (dst_mask | get_rook_attacks(dst, Bitboard::EMPTY))
                    } else if (bishop_attacks & dst_mask).is_not_empty() {
                        src_mask
                            | get_bishop_attacks(src, Bitboard::EMPTY)
                                & (dst_mask | get_bishop_attacks(dst, Bitboard::EMPTY))
                    } else {
                        Bitboard::EMPTY
                    }
                }
            }
        }
    }
}

#[must_use]
pub fn ray_intersecting(a: Square, b: Square) -> Bitboard {
    unsafe { INTERSECTING_RAYS[a.as_usize()][b.as_usize()] }
}
