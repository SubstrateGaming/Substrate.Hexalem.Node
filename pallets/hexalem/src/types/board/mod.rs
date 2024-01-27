use super::*;

use sp_std::vec;

pub use resource::*;
pub use tile::*;

mod resource;
mod tile;

// Custom trait for Tile definition
pub trait GetTileInfo {
	fn get_level(&self) -> u8;
	fn set_level(&mut self, level: u8);

	fn get_type(&self) -> TileType;

	fn get_pattern(&self) -> TilePattern;
	fn set_pattern(&mut self, value: TilePattern);

	fn same(&self, other: &Self) -> bool {
		self.get_type() == other.get_type()
	}

	fn get_home() -> Self;
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Copy, Clone, PartialEq)]
pub struct TileCost<Tile> {
	pub tile_to_buy: Tile,
	pub cost: ResourceAmount,
}

// The board hex grid
pub type HexGrid<Tile, N> = BoundedVec<Tile, N>;

// The board of the player, with all stats and resources
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct HexBoard<Tile, MaxGridSize> {
	pub resources: [ResourceUnit; NUMBER_OF_RESOURCE_TYPES],
	pub hex_grid: HexGrid<Tile, MaxGridSize>, // Board with all tiles
	pub game_id: GameId,                      // Game key
}

impl<Tile, MaxGridSize> HexBoard<Tile, MaxGridSize>
where
	Tile: Default + Clone + GetTileInfo,
	MaxGridSize: Get<u32>,
{
	pub fn try_new<DefaultPlayerResources>(
		size: usize,
		game_id: GameId,
	) -> Option<HexBoard<Tile, MaxGridSize>>
	where
		DefaultPlayerResources: Get<[ResourceUnit; 7]>,
	{
		if size > MaxGridSize::get() as usize {
			return None;
		}

		let maybe_new_hex_grid: Result<HexGrid<Tile, MaxGridSize>, _> =
			vec![Default::default(); size].try_into();

		if let Ok(mut new_hex_grid) = maybe_new_hex_grid {
			new_hex_grid[size / 2] = Tile::get_home();

			Some(HexBoard::<Tile, MaxGridSize> {
				resources: DefaultPlayerResources::get(),
				hex_grid: new_hex_grid,
				game_id,
			})
		} else {
			None
		}
	}

	pub fn get_stats(&self) -> BoardStats {
		let mut stats = BoardStats::default();

		for tile in self.hex_grid.clone() {
			let tile_type = tile.get_type();
			stats.set_tiles(tile_type, stats.get_tiles(tile_type).saturating_add(1));

			stats.set_levels(
				tile_type,
				tile.get_level(),
				stats.get_levels(tile_type, tile.get_level() as usize).saturating_add(1),
			);

			stats.set_patterns(
				tile_type,
				tile.get_pattern(),
				stats.get_patterns(tile_type, tile.get_pattern()).saturating_add(1),
			);
		}

		stats
	}
}

pub struct BoardStats {
	tiles: [u8; NUMBER_OF_TILE_TYPES],
	levels: [u8; NUMBER_OF_TILE_TYPES * NUMBER_OF_LEVELS],
	patterns: [u8; NUMBER_OF_TILE_TYPES * NUMBER_OF_PATTERNS],
}

impl Default for BoardStats {
	fn default() -> Self {
		Self {
			tiles: [0; NUMBER_OF_TILE_TYPES],
			levels: [0; NUMBER_OF_TILE_TYPES * NUMBER_OF_LEVELS],
			patterns: [0; NUMBER_OF_TILE_TYPES * NUMBER_OF_PATTERNS],
		}
	}
}

impl BoardStats {
	pub fn get_tiles(&self, tile_type: TileType) -> u8 {
		self.tiles[tile_type as usize]
	}

	pub fn get_tiles_by_tile_index(&self, tile_type_index: usize) -> u8 {
		self.tiles[tile_type_index]
	}

	pub fn set_tiles(&mut self, tile_type: TileType, value: u8) {
		self.tiles[tile_type as usize] = value;
	}

	pub fn get_levels(&self, tile_type: TileType, level: usize) -> u8 {
		self.levels[(tile_type as usize).saturating_mul(NUMBER_OF_LEVELS).saturating_add(level)]
	}

	pub fn set_levels(&mut self, tile_type: TileType, level: u8, value: u8) {
		self.levels[(tile_type as usize)
			.saturating_mul(NUMBER_OF_LEVELS)
			.saturating_add(level as usize)] = value;
	}

	pub fn get_patterns(&self, tile_type: TileType, pattern: TilePattern) -> u8 {
		self.patterns[(tile_type as usize)
			.saturating_mul(NUMBER_OF_PATTERNS)
			.saturating_add(pattern as usize)]
	}

	pub fn set_patterns(&mut self, tile_type: TileType, pattern: TilePattern, value: u8) {
		self.patterns[(tile_type as usize)
			.saturating_mul(NUMBER_OF_PATTERNS)
			.saturating_add(pattern as usize)] = value;
	}
}
