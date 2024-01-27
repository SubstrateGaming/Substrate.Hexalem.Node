use super::*;

pub type ResourceUnit = u8;

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone, Copy, PartialEq, Debug)]
pub enum ResourceType {
	Mana = 0,
	Human = 1,
	Water = 2,
	Food = 3,
	Wood = 4,
	Stone = 5,
	Gold = 6,
}

pub const NUMBER_OF_RESOURCE_TYPES: usize = 7;

pub const NUMBER_OF_LEVELS: usize = 4;

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Copy, Clone, PartialEq, Debug)]
pub struct ResourceAmount {
	pub resource_type: ResourceType,
	pub amount: ResourceUnit,
}

#[derive(Encode, TypeInfo)]
pub struct ResourceProductions {
	pub produces: [ResourceUnit; NUMBER_OF_RESOURCE_TYPES],
	pub human_requirements: [ResourceUnit; NUMBER_OF_RESOURCE_TYPES],
}

#[derive(Encode, Decode, TypeInfo, PartialEq, Clone, Debug)]
pub struct Move {
	pub place_index: u8,
	pub buy_index: u8,
}
