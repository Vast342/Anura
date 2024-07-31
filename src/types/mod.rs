pub mod bitboard;
pub mod piece;
pub mod moves;
pub mod square;

use arrayvec::ArrayVec;
use moves::Move;
pub type MoveList = ArrayVec<Move, 218>;