use frame_support::traits::Get;
use super::*;

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Clone, Debug)]
pub enum Rewards {
	Winner,
	Loser,
	Draw,
	// Other types of rewards
}

pub type RewardsDistribution<N> = BoundedVec<Rewards, N>;

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Clone, Debug)]
pub enum GameState<MaxPlayers: Get<u32>> {
	Playing,
	Finished(RewardsDistribution<MaxPlayers>), // Ready to reward players
}

pub trait GameProperties<Account, MaxPlayers: Get<u32>> {
	// Player made a move
	// It is used for determining whether to generate a new selection
	fn get_played(&self) -> bool;
	fn set_played(&mut self, played: bool);

	fn get_round(&self) -> u8;
	fn set_round(&mut self, round: u8);

	fn get_player_turn(&self) -> u8;
	fn set_player_turn(&mut self, turn: u8);

	fn borrow_players(&self) -> &Players<Account, MaxPlayers>;

	fn get_selection_size(&self) -> u8;
	fn set_selection_size(&mut self, selection_size: u8);
}

// Index used for referencing the TileCost
pub type TileCostIndex = u8;

// Tiles to select
pub type TileSelection<N> = BoundedVec<TileCostIndex, N>;

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Game<Account, BlockNumber, MaxPlayers: Get<u32>, MaxTiles> {
	pub state: GameState<MaxPlayers>,
	pub player_turn_and_played: u8,
	pub last_played_block: BlockNumber,
	pub players: Players<Account, MaxPlayers>, // Player AccountIds
	pub selection: TileSelection<MaxTiles>,
	pub selection_size: u8,
	pub round: u8,
	pub max_rounds: u8,
}

impl<Account, BlockNumber, MaxPlayers: Get<u32>, MaxTiles> GameProperties<Account, MaxPlayers>
	for Game<Account, BlockNumber, MaxPlayers, MaxTiles>
{
	fn get_played(&self) -> bool {
		((self.player_turn_and_played & 0x80) >> 7) == 1
	}

	fn set_played(&mut self, played: bool) {
		self.player_turn_and_played = (self.player_turn_and_played & 0x7F) | (played as u8) << 7
	}

	fn get_round(&self) -> u8 {
		self.round
	}

	fn set_round(&mut self, round: u8) {
		self.round = round;
	}

	fn get_player_turn(&self) -> u8 {
		self.player_turn_and_played & 0x7F
	}

	fn set_player_turn(&mut self, turn: u8) {
		self.player_turn_and_played = (self.player_turn_and_played & 0x80) | turn;
	}

	fn borrow_players(&self) -> &Players<Account, MaxPlayers> {
		&self.players
	}

	fn get_selection_size(&self) -> u8 {
		self.selection_size
	}

	fn set_selection_size(&mut self, selection_size: u8) {
		self.selection_size = selection_size;
	}
}
