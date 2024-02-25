mod board;
mod game;

pub use board::*;
pub use game::*;

use frame_support::pallet_prelude::*;

pub type GameId = [u8; 32];

pub type TargetGoalHash = [u8; 16];

pub type Players<Account, N> = BoundedVec<Account, N>;

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Eq, PartialEq, Debug)]
pub enum Rewards {
	Winner,
	Loser,
	Draw,
	// Other types of rewards
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Eq, PartialEq, Debug)]
pub enum MatchmakingState {
	None,
	Matchmaking,
	Joined(GameId),
	Finished(Rewards),
}

impl MatchmakingState {
	pub fn get_game_id(self) -> Option<GameId> {
		match self {
			MatchmakingState::Joined(game_id) => Some(game_id),
			_ => None,
		}
	}
}
