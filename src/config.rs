use crate::level::GenerationType;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::fs::File;
use std::io::Write;
use std::time;
use std::time::SystemTime;

fn default_name() -> String { "Minecraft server".to_string() }
fn default_motd() -> String { "Have fun.".to_string() }
fn default_max_clients() -> i8 { 20 }
fn default_address() -> String { "0.0.0.0:25565".to_string() }
fn default_level_name() -> String { "level".to_string() }
fn default_level_size_x() -> i16 { 128 }
fn default_level_size_y() -> i16 { 64 }
fn default_level_size_z() -> i16 { 128 }
fn default_level_type() -> GenerationType { GenerationType::Flat }
fn default_level_seed() -> u64 { SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs() }
fn default_heartbeat() -> bool { false }
fn default_heartbeat_address() -> String { "".to_string() }
fn default_verify_players() -> bool { false }
fn default_public() -> bool { false }

#[derive(Serialize, Deserialize)]
pub struct Config
{
	#[serde(default = "default_name")]
	pub name: String,
	#[serde(default = "default_motd")]
	pub motd: String,
	#[serde(default = "default_max_clients")]
	pub max_clients: i8,
	#[serde(default = "default_address")]
	pub address: String,
	#[serde(default = "default_level_name")]
	pub level_name: String,
	#[serde(default = "default_level_size_x")]
	pub level_size_x: i16,
	#[serde(default = "default_level_size_y")]
	pub level_size_y: i16,
	#[serde(default = "default_level_size_z")]
	pub level_size_z: i16,
	#[serde(default = "default_level_type")]
	pub level_type: GenerationType,
	#[serde(default = "default_level_seed")]
	pub level_seed: u64,
	#[serde(default = "default_heartbeat")]
	pub heartbeat: bool,
	#[serde(default = "default_heartbeat_address")]
	pub heartbeat_address: String,
	#[serde(default = "default_verify_players")]
	pub verify_players: bool,
	#[serde(default = "default_public")]
	pub public: bool
}

impl Default for Config
{
    fn default() -> Self
	{
        Self
		{
			name: default_name(),
			motd: default_motd(),
			max_clients: default_max_clients(),
			address: default_address(),
			level_name: default_level_name(),
			level_size_x: default_level_size_x(),
			level_size_y: default_level_size_y(),
			level_size_z: default_level_size_z(),
			level_type: default_level_type(),
			level_seed: default_level_seed(),
			heartbeat: default_heartbeat(),
			heartbeat_address: default_heartbeat_address(),
			verify_players: default_verify_players(),
			public: default_public()
		}
    }
}

const CONFIG_FILE: &str = "properties.json";

pub fn load_config() -> Config
{
	let config;

	if let Ok(file) = File::open(CONFIG_FILE)
	{
		config = serde_json::from_reader(file).unwrap();
	}
	else
	{
		println!("could not open config file. loading default settings.");
		config = Config::default();
	}

	if let (Ok(json), Ok(mut file)) = (serde_json::to_string_pretty(&config), File::create("properties.json"))
	{
		if file.write(json.as_bytes()).is_err()
		{
			println!("could not write config file.");
		}
	}
	else
	{
		println!("could not parse config file.");
	}

	config
}