use crate::{
    movegen::lookups::{A_FILE, DIAGONALS, KING_ATTACKS, KNIGHT_ATTACKS},
    types::{
        moves::{Flag, Move},
        square::Square,
    },
};

pub fn move_index(ctm: u8, mov: Move, king: Square) -> usize {
    let hm = if king.0 % 8 > 3 { 7 } else { 0 };

    let idx = if mov.is_promotion() {
        let ffile = (mov.from() ^ hm) % 8;
        let tfile = (mov.to() ^ hm) % 8;
        let promo_id = 2 * ffile + tfile;
        OFFSETS[64] + 22 * (mov.flag() as usize - Flag::KnightPromo as usize) + promo_id as usize
    } else {
        let flipper = if ctm == 0 { 56 } else { 0 };
        let from = (mov.from() ^ flipper ^ hm) as usize;
        let to = (mov.to() ^ flipper ^ hm) as usize;

        let below = ALL_DESTINATIONS[from] & ((1 << to) - 1);

        OFFSETS[from] + below.count_ones() as usize
    };

    idx
}

const OFFSETS: [usize; 65] = {
    let mut offsets = [0; 65];
    let mut curr = 0;
    let mut square = 0;
    while square < 64 {
        offsets[square] = curr;
        curr += ALL_DESTINATIONS[square].count_ones() as usize;
        square += 1;
    }
    offsets[64] = curr;
    offsets
};

const ALL_DESTINATIONS: [u64; 64] = {
    let mut thing = [0; 64];
    let mut square = 0;
    while square < 64 {
        let rank = square / 8;
        let file = square % 8;
        let rooks = (0xFF << (rank * 8)) ^ (A_FILE << file);
        let bishops = DIAGONALS[file + rank].swap_bytes() ^ DIAGONALS[7 + file - rank];
        thing[square] = rooks | bishops | KNIGHT_ATTACKS[square] | KING_ATTACKS[square];
        square += 1;
    }
    thing
};
