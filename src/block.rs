pub struct BlockState
{
	pub place_op_only: bool,
	pub destroy_op_only: bool,
	pub fall: bool,
	pub slab: Option<u8>,
	pub fluid: bool, // can have blocks placed in
}

pub const BLOCKS: [BlockState; 50] =
[
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: true }, // Air
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Stone
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Grass
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Dirt
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Cobblestone
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Planks
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Sapling
	BlockState { place_op_only: true, destroy_op_only: true, fall: false, slab: None, fluid: false }, // Bedrock
	BlockState { place_op_only: true, destroy_op_only: false, fall: false, slab: None, fluid: true }, // Flowing Water
	BlockState { place_op_only: true, destroy_op_only: false, fall: false, slab: None, fluid: true }, // Stationary Water
	BlockState { place_op_only: true, destroy_op_only: false, fall: false, slab: None, fluid: true }, // Flowing Lava
	BlockState { place_op_only: true, destroy_op_only: false, fall: false, slab: None, fluid: true }, // Stationary Lava
	BlockState { place_op_only: false, destroy_op_only: false, fall: true, slab: None, fluid: false }, // Sand
	BlockState { place_op_only: false, destroy_op_only: false, fall: true, slab: None, fluid: false }, // Gravel
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Gold Ore
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Iron Ore
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Coal Ore
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Wood
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Leaves
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Sponge
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Glass
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Red Cloth
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Orange Cloth
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Yellow Cloth
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Chartreuse Cloth
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Green Cloth
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Spring Green Cloth
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Cyan Cloth
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Capri Cloth
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Ultramarine Cloth
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Violet Cloth
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Purple Cloth
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Magenta Cloth
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Rose Cloth
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Dark Gray Cloth
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Light Gray Cloth
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // White Cloth
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Flower
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Rose
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Brown Mushroom
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Red Mushroom
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Gold Block
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Iron Block
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Double Slab
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: Some(43), fluid: false }, // Slab
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Bricks
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // TNT
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Bookshelf
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Mossy Cobblestone
	BlockState { place_op_only: false, destroy_op_only: false, fall: false, slab: None, fluid: false }, // Obsidian
];