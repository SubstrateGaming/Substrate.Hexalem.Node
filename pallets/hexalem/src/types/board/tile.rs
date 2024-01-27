use crate::types::*;

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TileType {
	Empty = 0,
	Home = 1,
	Grass = 2,
	Water = 3,
	Mountain = 4,
	Tree = 5,
	Desert = 6,
	Cave = 7,
}

impl From<u8> for TileType {
	fn from(value: u8) -> Self {
		match value {
			1 => TileType::Home,
			2 => TileType::Grass,
			3 => TileType::Water,
			4 => TileType::Mountain,
			5 => TileType::Tree,
			6 => TileType::Desert,
			7 => TileType::Cave,
			_ => TileType::Empty,
		}
	}
}

pub const NUMBER_OF_TILE_TYPES: usize = 8;

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TilePattern {
	Normal = 0,
	Delta = 1,
	Line = 2,
	Ypsilon = 3,
}

pub const NUMBER_OF_PATTERNS: usize = 8;

impl From<u8> for TilePattern {
	fn from(value: u8) -> Self {
		match value {
			1 => TilePattern::Delta,
			2 => TilePattern::Line,
			3 => TilePattern::Ypsilon,
			_ => TilePattern::Normal,
		}
	}
}
