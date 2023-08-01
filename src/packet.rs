#[derive(Debug,Clone)]
pub enum Packet
{
	Unknown 
	{
		id: u8
	},
	Identification
	{
		protocol: u8,
		name: String,
		data: String,
		user_mode: u8
	},
	Ping,
	LevelStart,
	LevelData
	{
		length: i16,
		data: Vec<u8>,
		percentage: u8
	},
	LevelSize
	{
		x: i16,
		y: i16,
		z: i16,
	},
	PlaceBlock
	{
		x: i16,
		y: i16,
		z: i16,
		mode: u8,
		block: u8
	},
	SetBlock
	{
		x: i16,
		y: i16,
		z: i16,
		block: u8
	},
	Spawn
	{
		id: i8,
		name: String,
		x: i16,
		y: i16,
		z: i16,
		yaw: u8,
		pitch: u8
	},
	SetPosAndLook
	{
		id: i8,
		x: i16,
		y: i16,
		z: i16,
		yaw: u8,
		pitch: u8
	},
	UpdatePosAndLook
	{
		id: i8,
		x: i8,
		y: i8,
		z: i8,
		yaw: u8,
		pitch: u8
	},
	UpdatePos
	{
		id: i8,
		x: i8,
		y: i8,
		z: i8
	},
	UpdateLook
	{
		id: i8,
		yaw: u8,
		pitch: u8
	},
	Despawn
	{
		id: i8
	},
	Message
	{
		id: i8,
		message: String
	},
	Disconnect
	{
		reason: String
	},
	UpdateUserMode
	{
		user_mode: u8
	}
}
