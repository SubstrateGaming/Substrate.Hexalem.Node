use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

pub type GameId = u32;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Position {
	#[default]
	One,
	Two,
	Three,
}

impl From<Position> for usize {
	fn from(value: Position) -> Self {
		match value {
			Position::One => 0,
			Position::Two => 1,
			Position::Three => 2,
		}
	}
}

impl From<usize> for Position {
	fn from(value: usize) -> Self {
		match value {
			0 => Position::One,
			1 => Position::Two,
			2 => Position::Three,
			_ => Position::One,
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct Coordinates {
	row: Position,
	col: Position,
}

impl From<Coordinates> for (usize, usize) {
	fn from(value: Coordinates) -> Self {
		(value.row.into(), value.col.into())
	}
}

impl From<(usize, usize)> for Coordinates {
	fn from(value: (usize, usize)) -> Self {
		Self { row: Position::from(value.0), col: Position::from(value.1) }
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum CellState {
	Circle,
	Cross,
	#[default]
	Empty,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum CurrentTurn {
	#[default]
	PlayerOne,
	PlayerTwo,
}

impl CurrentTurn {
	fn next(&self) -> Self {
		match self {
			CurrentTurn::PlayerOne => CurrentTurn::PlayerTwo,
			CurrentTurn::PlayerTwo => CurrentTurn::PlayerOne,
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum BoardState<AccountId> {
	MissingPlayer(AccountId),
	Playing(AccountId, AccountId, CurrentTurn),
	Finished(AccountId, AccountId),
}

impl<AccountId> BoardState<AccountId>
where
	AccountId: Clone,
{
	fn next_turn(&mut self) {
		if let BoardState::Playing(_, _, turn) = self {
			*turn = turn.next();
		};
	}
}

impl<AccountId> BoardState<AccountId>
where
	AccountId: Clone,
{
	fn to_playing(&self, other: AccountId) -> Option<Self> {
		if let BoardState::MissingPlayer(player) = self {
			Some(BoardState::Playing(player.clone(), other, CurrentTurn::PlayerOne))
		} else {
			None
		}
	}

	fn to_finished(&self) -> Option<Self> {
		if let BoardState::Playing(p1, p2, _) = self {
			Some(BoardState::Finished(p1.clone(), p2.clone()))
		} else {
			None
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum PlayResult<AccountId> {
	Winner(AccountId),
	Draw,
	InvalidTurn,
	InvalidCell,
	GamePending,
	GameAlreadyFinished,
	Continue,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct Board<AccountId> {
	pub(crate) cells: [[CellState; 3]; 3],
	pub(crate) state: BoardState<AccountId>,
	pub(crate) moves_played: u8,
}

impl<AccountId> Board<AccountId>
where
	AccountId: Clone,
{
	pub(crate) fn new(creator: AccountId) -> Self {
		Self {
			cells: [[CellState::default(); 3]; 3],
			state: BoardState::MissingPlayer(creator),
			moves_played: 0,
		}
	}

	pub(crate) fn get_state(&self) -> BoardState<AccountId> {
		self.state.clone()
	}

	pub(crate) fn start_game(&mut self, rival: AccountId)
	where
		AccountId: Clone,
	{
		self.state = self.state.to_playing(rival).unwrap();
	}

	pub(crate) fn play_turn(
		&mut self,
		player: &AccountId,
		coordinates: Coordinates,
	) -> PlayResult<AccountId>
	where
		AccountId: PartialEq + Clone,
	{
		match self.state {
			BoardState::MissingPlayer(_) => PlayResult::GamePending,
			BoardState::Playing(ref p1, ref p2, turn) => match turn {
				CurrentTurn::PlayerOne =>
					if player == p1 {
						self.execute_move(player, CellState::Circle, coordinates)
					} else {
						PlayResult::InvalidTurn
					},
				CurrentTurn::PlayerTwo =>
					if player == p2 {
						self.execute_move(player, CellState::Cross, coordinates)
					} else {
						PlayResult::InvalidTurn
					},
			},
			BoardState::Finished(_, _) => PlayResult::GameAlreadyFinished,
		}
	}

	fn execute_move(
		&mut self,
		player: &AccountId,
		cell_type: CellState,
		coordinates: Coordinates,
	) -> PlayResult<AccountId>
	where
		AccountId: PartialEq + Clone,
	{
		let (row, col) = coordinates.into();

		if self.cells[col][row] == CellState::Empty {
			self.cells[col][row] = cell_type;
			self.moves_played += 1;
		} else {
			return PlayResult::InvalidCell
		}

		if self.has_player_won(cell_type) {
			self.finish_game();
			PlayResult::Winner(player.clone())
		} else if self.moves_played == 9 {
			self.finish_game();
			PlayResult::Draw
		} else {
			self.state.next_turn();
			PlayResult::Continue
		}
	}

	fn has_player_won(&self, cell_type: CellState) -> bool {
		let mut results = self.check_row(0, cell_type) ||
			self.check_row(1, cell_type) ||
			self.check_row(2, cell_type);

		results = results ||
			self.check_col(0, cell_type) ||
			self.check_col(1, cell_type) ||
			self.check_col(2, cell_type);

		results = results || self.check_diagonals(cell_type);

		results
	}

	fn check_row(&self, row: usize, cell_type: CellState) -> bool {
		self.cells[0][row] == cell_type &&
			self.cells[1][row] == cell_type &&
			self.cells[2][row] == cell_type
	}

	fn check_col(&self, col: usize, cell_type: CellState) -> bool {
		self.cells[col][0] == cell_type &&
			self.cells[col][1] == cell_type &&
			self.cells[col][2] == cell_type
	}

	fn check_diagonals(&self, cell_type: CellState) -> bool {
		(self.cells[0][0] == cell_type &&
			self.cells[1][1] == cell_type &&
			self.cells[2][2] == cell_type) ||
			(self.cells[2][0] == cell_type &&
				self.cells[1][1] == cell_type &&
				self.cells[0][2] == cell_type)
	}

	fn finish_game(&mut self) {
		self.state = self.state.to_finished().unwrap();
	}
}
