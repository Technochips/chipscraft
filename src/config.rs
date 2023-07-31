use crate::level::GenerationType;
use crate::userdata::UserData;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::fs::File;
use std::io::Write;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::IpAddr;
use std::time;
use std::time::SystemTime;

fn default_name() -> String { "Minecraft server".to_string() }
fn default_motd() -> String { "Have fun.".to_string() }
fn default_rules() -> String { "Be nice.".to_string() }
fn default_max_clients() -> i8 { 20 }
fn default_address() -> SocketAddr { SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 25565) }
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
	#[serde(default = "default_rules")]
	pub rules: String,
	#[serde(default = "default_max_clients")]
	pub max_clients: i8,
	#[serde(default = "default_address")]
	pub address: SocketAddr,
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
	pub public: bool,

	#[serde(skip, default = "UserData::load")]
	pub user_data: UserData,
}

impl Default for Config
{
    fn default() -> Self
	{
        Self
		{
			name: default_name(),
			motd: default_motd(),
			rules: default_rules(),
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
			public: default_public(),
			user_data: UserData::load()
		}
    }
}
impl Config
{
	const FILE: &str = "properties.json";

	pub fn load() -> Result<Self, String>
	{
		let config;
		let mut firsttime = false;

		if let Ok(file) = File::open(Config::FILE)
		{
			config = serde_json::from_reader(file).unwrap();
		}
		else
		{
			firsttime = true;
			config = Config::default();
		}

		if let (Ok(json), Ok(mut file)) = (serde_json::to_string_pretty(&config), File::create(Config::FILE))
		{
			if file.write(json.as_bytes()).is_err()
			{
				return Err("could not write config file.".to_string());
			}
		}
		else
		{
			return Err("could not create config file.".to_string());
		}

		if firsttime
		{
			return Err("config file was not found, so one was created. please configure it, and then you can run the program again. add your username to the operator list as well.".to_string());
		}

		Ok(config)
	}
}