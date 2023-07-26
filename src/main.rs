mod block;
mod client;
mod io;
mod noise;
mod packet;
mod server;
mod level;

use crate::client::Client;
use crate::level::GenerationType;
use crate::level::Level;
use crate::server::Server;
use std::sync::Arc;
use std::time::SystemTime;
use std::time;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::signal;
use serde_derive::Serialize;
use serde_derive::Deserialize;
use std::fs::File;
use std::io::Write;

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

#[derive(Serialize, Deserialize)]
struct Config
{
	#[serde(default = "default_name")]
	name: String,
	#[serde(default = "default_motd")]
	motd: String,
	#[serde(default = "default_max_clients")]
	max_clients: i8,
	#[serde(default = "default_address")]
	address: String,
	#[serde(default = "default_level_name")]
	level_name: String,
	#[serde(default = "default_level_size_x")]
	level_size_x: i16,
	#[serde(default = "default_level_size_y")]
	level_size_y: i16,
	#[serde(default = "default_level_size_z")]
	level_size_z: i16,
	#[serde(default = "default_level_type")]
	level_type: GenerationType,
	#[serde(default = "default_level_seed")]
	level_seed: u64
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
			level_seed: default_level_seed()
		}
    }
}

const CONFIG_FILE: &str = "properties.json";

#[tokio::main]
async fn main()
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

	let listener: TcpListener = TcpListener::bind(config.address.clone()).await.unwrap();
	let mut level = Level::new();
	if level.load(format!("{}.dat", config.level_name)).is_err()
	{
		level.generate(config.level_size_x, config.level_size_y, config.level_size_z, config.level_type, config.level_seed).unwrap();
	}
	let server = Arc::new(Mutex::new(Server::new(config.max_clients, level, config.name.clone(), config.motd.clone())));
	let s = server.clone();
	println!("ready");
	tokio::select! 
	{
		_ = async move
		{
			while let Ok((stream, ip)) = listener.accept().await
			{
				let server = Arc::clone(&s);
				tokio::spawn(Client::handle_client(stream, ip, server));
			}
		} => {}
		_ = signal::ctrl_c() => {}
	}
	println!("shutting down...");
	server.lock().await.stop();
}