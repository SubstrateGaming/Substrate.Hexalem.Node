mod board;
mod game;

pub use board::*;
pub use game::*;

use frame_support::pallet_prelude::*;

pub type GameId = [u8; 32];

pub type TargetGoalHash = [u8; 16];

pub type Players<Account, N> = BoundedVec<Account, N>;
