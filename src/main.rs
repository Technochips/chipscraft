mod block;
mod chat;
mod client;
mod command;
mod config;
mod io;
mod level;
mod noise;
mod packet;
mod server;
mod userdata;

use crate::client::Client;
use crate::config::Config;
use crate::level::Level;
use crate::server::Server;
use crate::userdata::UserData;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::Mutex;

#[tokio::main]
async fn main()
{
	let config = Config::load();
	let user_data = UserData::load();

	let address: SocketAddr = config.address.parse().expect("could not parse address");

	let listener: TcpListener = TcpListener::bind(address).await.unwrap();
	let mut level = Level::new(config.level_name);
	if level.load().is_err()
	{
		level.generate(config.level_size_x, config.level_size_y, config.level_size_z, config.level_type, config.level_seed).unwrap();
	}
	let server = Arc::new(Mutex::new(Server::new(config.max_clients, level, config.name, config.motd, user_data, config.public, config.verify_players, config.heartbeat, config.heartbeat_address, address.port(), config.rules)));
	Server::start_ticks(&server).await;
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