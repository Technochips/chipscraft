mod block;
mod client;
mod config;
mod io;
mod level;
mod noise;
mod packet;
mod server;

use crate::client::Client;
use crate::level::Level;
use crate::server::Server;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::Mutex;

#[tokio::main]
async fn main()
{
	let config = config::load_config();

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