/*
    This file is from Ciekce's rust MCTS engine voidstar, and adapted to work with my structs
    Original code is licensed under GPL 3.0, same as Anura, and is used with permission.
*/

use crate::movegen::slideys::{get_bishop_attacks, get_rook_attacks};
use crate::types::bitboard::Bitboard;
use crate::types::square::Square;

use std::sync::OnceLock;

static BETWEEN_RAYS: OnceLock<[[Bitboard; 64]; 64]> = OnceLock::new();
static INTERSECTING_RAYS: OnceLock<[[Bitboard; 64]; 64]> = OnceLock::new();

#[allow(clippy::needless_range_loop)]
fn init_between_table() -> [[Bitboard; 64]; 64] {
    let mut table = [[Bitboard::EMPTY; 64]; 64];

    for src_idx in 0..64 {
        let src = Square(src_idx as u8);
        let src_mask = Bitboard::from_square(src);

        let rook_attacks = get_rook_attacks(src, Bitboard::EMPTY);
        let bishop_attacks = get_bishop_attacks(src, Bitboard::EMPTY);

        for dst_idx in 0..64 {
            table[src_idx][dst_idx] = if src_idx == dst_idx {
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
    table
}

#[allow(clippy::needless_range_loop)]
fn init_intersection_table() -> [[Bitboard; 64]; 64] {
    let mut table = [[Bitboard::EMPTY; 64]; 64];

    for src_idx in 0..64 {
        let src = Square(src_idx as u8);
        let src_mask = Bitboard::from_square(src);

        let rook_attacks = get_rook_attacks(src, Bitboard::EMPTY);
        let bishop_attacks = get_bishop_attacks(src, Bitboard::EMPTY);

        for dst_idx in 0..64 {
            table[src_idx][dst_idx] = if src_idx == dst_idx {
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
    table
}

pub fn init_ray_tables() {
    BETWEEN_RAYS.get_or_init(init_between_table);
    INTERSECTING_RAYS.get_or_init(init_intersection_table);
}

#[must_use]
pub fn ray_between(a: Square, b: Square) -> Bitboard {
    let table = BETWEEN_RAYS.get_or_init(init_between_table);
    table[a.as_usize()][b.as_usize()]
}

#[must_use]
pub fn ray_intersecting(a: Square, b: Square) -> Bitboard {
    let table = INTERSECTING_RAYS.get_or_init(init_intersection_table);
    table[a.as_usize()][b.as_usize()]
}