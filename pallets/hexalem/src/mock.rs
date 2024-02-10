use crate as pallet_hexalem;
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU64, Get},
};
use pallet_hexalem::{
	GetTileInfo, ResourceAmount, ResourceProductions, ResourceType, ResourceUnit, TileCost,
	TilePattern, TileType, NUMBER_OF_RESOURCE_TYPES, NUMBER_OF_TILE_TYPES,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type Block = frame_system::mocking::MockBlock<TestRuntime>;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct HexalemTile(pub u8);

impl GetTileInfo for HexalemTile {
	fn get_level(&self) -> u8 {
		(self.0 >> 6) & 0x3
	}

	fn set_level(&mut self, level: u8) {
		self.0 = (self.0 & 0x3F) | (level << 6);
	}

	fn get_type(&self) -> TileType {
		TileType::from((self.0 >> 3) & 0x7)
	}

	fn get_pattern(&self) -> TilePattern {
		TilePattern::from(self.0 & 0x7)
	}

	fn set_pattern(&mut self, pattern: TilePattern) {
		self.0 = (self.0 & 0xF8) | (pattern as u8);
	}

	fn get_home() -> Self {
		Self(8) // Home level 0
	}
}

impl HexalemTile {
	pub fn new(tile_type: TileType, level: u8, pattern: TilePattern) -> Self {
		let encoded = ((tile_type as u8) << 3) | ((level & 0x3) << 6) | (pattern as u8 & 0x7);
		Self(encoded)
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct ParameterGet<const N: u32>;

impl<const N: u32> Get<u32> for ParameterGet<N> {
	fn get() -> u32 {
		N
	}
}

pub type HexalemMaxPlayers = ParameterGet<100>;
pub type HexalemMaxHexGridSize = ParameterGet<49>;
pub type HexalemMaxTileSelection = ParameterGet<16>;

parameter_types! {
	pub const HexalemMinPlayers: u8 = 1;
	pub const HexalemMaxRounds: u8 = 25;

	pub const HexalemBlocksToPlayLimit: u8 = 10;

	pub const HexalemTileResourceProductions: [ResourceProductions; NUMBER_OF_TILE_TYPES] = [
		// Empty
		ResourceProductions{
			produces: [0, 0, 0, 0, 0, 0, 0],
			human_requirements: [0, 0, 0, 0, 0, 0, 0],
		},
		// Home
		ResourceProductions{
			produces: [0, 1, 0, 0, 0, 0, 0],
			human_requirements: [0, 0, 0, 0, 0, 0, 0],
		},
		// Grass
		ResourceProductions{
			produces: [0, 0, 0, 2, 0, 0, 0],
			human_requirements: [0, 0, 0, 0, 0, 0, 0],
		},
		// Water
		ResourceProductions{
			produces: [0, 0, 2, 0, 0, 0, 0],
			human_requirements: [0, 0, 0, 0, 0, 0, 0],
		},
		// Mountain
		ResourceProductions{
			produces: [0, 0, 0, 0, 0, 4, 0],
			human_requirements: [0, 0, 0, 0, 0, 4, 0],
		},
		// Tree
		ResourceProductions{
			produces: [0, 0, 0, 1, 3, 0, 0],
			human_requirements: [0, 0, 0, 0, 2, 0, 0],
		},
		// Desert
		ResourceProductions{
			produces: [0, 0, 0, 0, 0, 0, 0],
			human_requirements: [0, 0, 0, 0, 0, 0, 0],
		},
		// Cave
		ResourceProductions{
			produces: [0, 0, 0, 0, 0, 2, 1],
			human_requirements: [0, 0, 0, 0, 0, 2, 3],
		},
	];

	pub const HexalemTileCosts: [TileCost<HexalemTile>; 15] = [
		// tile_to_buy: HexalemTile(16), // Grass, level 0
		// tile_to_buy: HexalemTile(24), // Water, level 0
		// tile_to_buy: HexalemTile(32), // Mountain, level 0
		// tile_to_buy: HexalemTile(40), // Tree, level 0
		// tile_to_buy: HexalemTile(48), // Desert, level 0
		// tile_to_buy: HexalemTile(56), // Cave, level 0

		TileCost {
			tile_to_buy: HexalemTile(16), // Grass, level 0
			cost: ResourceAmount { resource_type: ResourceType::Mana, amount: 1, }
		},
		TileCost {
			tile_to_buy: HexalemTile(16), // Grass, level 0
			cost: ResourceAmount { resource_type: ResourceType::Mana, amount: 1, }
		},
		TileCost {
			tile_to_buy: HexalemTile(16), // Grass, level 0
			cost: ResourceAmount { resource_type: ResourceType::Mana, amount: 1, }
		},
		TileCost {
			tile_to_buy: HexalemTile(24), // Water, level 0
			cost: ResourceAmount { resource_type: ResourceType::Mana, amount: 1, }
		},
		TileCost {
			tile_to_buy: HexalemTile(24), // Water, level 0
			cost: ResourceAmount { resource_type: ResourceType::Mana, amount: 1, }
		},
		TileCost {
			tile_to_buy: HexalemTile(24), // Water, level 0
			cost: ResourceAmount { resource_type: ResourceType::Mana, amount: 1, }
		},

		TileCost {
			tile_to_buy: HexalemTile(32), // Mountain, level 0
			cost: ResourceAmount { resource_type: ResourceType::Mana, amount: 1, }
		},
		TileCost {
			tile_to_buy: HexalemTile(32), // Mountain, level 0
			cost: ResourceAmount { resource_type: ResourceType::Mana, amount: 1, }
		},
		TileCost {
			tile_to_buy: HexalemTile(32), // Mountain, level 0
			cost: ResourceAmount { resource_type: ResourceType::Mana, amount: 1, }
		},
		TileCost {
			tile_to_buy: HexalemTile(40), // Tree, level 0
			cost: ResourceAmount { resource_type: ResourceType::Mana, amount: 1, }
		},
		TileCost {
			tile_to_buy: HexalemTile(40), // Tree, level 0
			cost: ResourceAmount { resource_type: ResourceType::Mana, amount: 1, }
		},
		TileCost {
			tile_to_buy: HexalemTile(40), // Tree, level 0
			cost: ResourceAmount { resource_type: ResourceType::Mana, amount: 1, }
		},
		TileCost {
			tile_to_buy: HexalemTile(48), // Desert, level 0
			cost: ResourceAmount { resource_type: ResourceType::Mana, amount: 1, }
		},
		TileCost {
			tile_to_buy: HexalemTile(56), // Cave, level 0
			cost: ResourceAmount { resource_type: ResourceType::Mana, amount: 1, }
		},
		TileCost {
			tile_to_buy: HexalemTile(56), // Cave, level 0
			cost: ResourceAmount { resource_type: ResourceType::Mana, amount: 1, }
		},
	];

	pub const HexalemFoodPerHuman: u8 = 1u8;
	pub const HexalemWaterPerHuman: u8 = 2u8;
	pub const HexalemHomePerHumans: u8 = 3u8;
	pub const HexalemFoodPerTree: u8 = 1u8;

	pub const HexalemDefaultPlayerResources: [ResourceUnit; NUMBER_OF_RESOURCE_TYPES] = [1, 1, 0, 0, 0, 0, 0];

	pub const HexalemTargetGoalGold: u8 = 10u8;
	pub const HexalemTargetGoalHuman: u8 = 7u8;
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum TestRuntime
	{
		System: frame_system,
		HexalemModule: pallet_hexalem,
		MatchmakerModule: pallet_matchmaker,
		EloModule: pallet_elo,
	}
);

// matchmaker parameters
parameter_types! {
	pub const AmountPlayers: u8 = 2;
	pub const AmountBrackets: u8 = 2;
}

/// pallet used for matchmaking in pallet-rps.
impl pallet_matchmaker::Config for TestRuntime {
	type RuntimeEvent = RuntimeEvent;
	type AmountPlayers = AmountPlayers;
	type AmountBrackets = AmountBrackets;
}

impl pallet_elo::Config for TestRuntime {
	type RuntimeEvent = RuntimeEvent;
	type MaxPlayers = HexalemMaxPlayers;
}

impl frame_system::Config for TestRuntime {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_hexalem::Config for TestRuntime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type MaxPlayers = HexalemMaxPlayers;
	type MinPlayers = HexalemMinPlayers;
	type MaxRounds = HexalemMaxRounds;
	type BlocksToPlayLimit = HexalemBlocksToPlayLimit;
	type MaxHexGridSize = HexalemMaxHexGridSize;
	type MaxTileSelection = HexalemMaxTileSelection;
	type Tile = HexalemTile;
	type TileCosts = HexalemTileCosts;
	type TileResourceProductions = HexalemTileResourceProductions;
	type WaterPerHuman = HexalemWaterPerHuman;
	type FoodPerHuman = HexalemFoodPerHuman;
	type FoodPerTree = HexalemFoodPerTree;
	type HomePerHumans = HexalemHomePerHumans;
	type DefaultPlayerResources = HexalemDefaultPlayerResources;
	type TargetGoalGold = HexalemTargetGoalGold;
	type TargetGoalHuman = HexalemTargetGoalHuman;
	type Matchmaker = MatchmakerModule;
	type Elo = EloModule;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<TestRuntime>::default()
		.build_storage()
		.unwrap()
		.into()
}
