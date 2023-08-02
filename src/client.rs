use crate::io;
use crate::io::AsyncReadClassicExt;
use crate::io::AsyncWriteClassicExt;
use crate::level::SaveType;
use crate::packet::Packet;
use crate::server::Server;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

#[derive(PartialEq, Clone, Copy)]
pub enum ClientMode
{
	Normal,
	Operator
}
impl ClientMode
{
	pub fn get_id(&self) -> u8
	{
		match self
		{
			ClientMode::Normal => 0x00,
			ClientMode::Operator => 0x64,
		}
	}
}
pub struct Client
{
	pub ip: SocketAddr,
	pub username: String,
	pub x: i16,
	pub y: i16,
	pub z: i16,
	pub pitch: u8,
	pub yaw: u8,
	pub mode: ClientMode,
	pub packet_sender: UnboundedSender<Packet>,
}

const TIMEOUT: Duration = Duration::from_secs(10);

impl Client
{
	pub async fn sender_client(mut recv: UnboundedReceiver<Packet>, write: Arc<Mutex<OwnedWriteHalf>>) -> String
	{
		while let Some(packet) = recv.recv().await
		{
			if let Packet::Disconnect { reason } = packet
			{
				return reason;
			}
			if let Ok(result) = tokio::time::timeout(TIMEOUT, write.lock().await.write_packet(packet)).await
			{
				if result.is_err()
				{
					return "Connection closed.".to_string();
				}
			}
			else
			{
				return "Timed out".to_string();
			}
		}
		"No more packets to send... server most likely crashed.".to_string()
	}
	pub async fn receiver_client(mut read: OwnedReadHalf, server: Arc<Mutex<Server>>, id: i8, username: String, ip: SocketAddr) -> String
	{
		while let Ok(result) = tokio::time::timeout(TIMEOUT, read.read_packet()).await
		{
			if let Ok(packet) = result
			{
				if !match packet
				{
					Packet::PlaceBlock { x, y, z, block, mode } =>
					{
						let server = server.clone();
						tokio::spawn(async move
							{
								server.lock().await.set_block(id,x,y,z, if mode == 0 { 0 } else { block })
							});
						true
					}
					Packet::SetPosAndLook { id: _, x, y, z, yaw, pitch } =>
					{
						let server = server.clone();
						tokio::spawn(async move
						{
							server.lock().await.move_player(id, id, x, y, z, yaw, pitch);
						});
						true
					}
					Packet::Message { id: _, message } =>
					{
						let server = server.clone();

						// check if first char is /
						if message.len() > 0 && &message[0..1] == "/"
						{
							tokio::spawn(async move
							{
								let mut split = message.split(' ');
								let command = split.next().unwrap()[1..].to_lowercase();
								server.lock().await.command(id, command, split.collect());
							});
						}
						else if server.lock().await.config.user_data.muted.contains(&username, &ip.ip())
						{
							println!("{}:<{}> (muted) {}", id, username, message);
							tokio::spawn(async move
							{
								server.lock().await.send_message(-1, id, "You have been muted, you cannot send any messages.");
							});
						}
						else
						{
							let username = username.clone();
							tokio::spawn(async move
							{
								server.lock().await.broadcast_message(id, &format!("<{}> {}", username, message));
							});
						}
						true
					}
					_ => false
				}
				{
					return "Invalid packet received.".to_string();
				}
			}
			else
			{
				return "Connection closed".to_string();
			}
		}
		"Timed out".to_string()
	}
	pub async fn send_level(stream: &mut TcpStream, server: &Arc<Mutex<Server>>) -> Option<()>
	{
		let size_x = server.lock().await.level.size_x;
		let size_y = server.lock().await.level.size_y;
		let size_z = server.lock().await.level.size_z;
		let gzip = server.lock().await.level.get_gzip(SaveType::Network).unwrap();
		let total_chunk = (gzip.len() + io::ARRAY_LEN - 1) / io::ARRAY_LEN;
		for (i, chunk) in gzip.chunks(io::ARRAY_LEN).enumerate()
		{
			if stream.write_packet(Packet::LevelData { length: chunk.len() as i16, data: chunk.to_vec(), percentage: ((i+1)*100/total_chunk) as u8 }).await.is_err() { return None; };
		}
		if stream.write_packet(Packet::LevelSize { x: size_x, y: size_y, z: size_z }).await.is_err() { return None; };
		Some(())
	}
	pub async fn init_client(mut stream: TcpStream, ip: SocketAddr, server: &Arc<Mutex<Server>>) -> Option<(JoinHandle<String>, JoinHandle<String>, i8, Arc<Mutex<OwnedWriteHalf>>)>
	{
		let username;
		if let Ok(Packet::Identification { protocol, name, data: key, user_mode: _ }) = stream.read_packet().await
		{
			if protocol != 0x07
			{
				return None;
			}
			if server.lock().await.config.user_data.banned.contains(&name, &ip.ip())
			{
				println!("{} from {} tried to connect but was banned", name, ip);
				let _ = stream.write_packet(Packet::Disconnect { reason: "You are banned.".to_string() }).await;
				return None;
			}
			if server.lock().await.config.verify_players
			{
				if key != format!("{:?}", md5::compute(format!("{}{}", server.lock().await.salt.clone(), name)))
				{
					println!("{} tried to connect but couldn't verify from {}", name, ip);
					let _ = stream.write_packet(Packet::Disconnect { reason: "Could not verify. Try refreshing the server list.".to_string() }).await;
					return None;
				}
			}
			username = name;
		}
		else
		{
			return None;
		};
		let id = server.lock().await.first_free_space();
		if id.is_none()
		{
			println!("server too full. {} tried to connect from {}", username, ip);
			// we do not really care if this packets fails to get sent.
			let _ = stream.write_packet(Packet::Disconnect { reason: "Too many players.".to_string() }).await;
			return None;
		}
		let id = id.unwrap();
		if server.lock().await.get_index_from_username(&username).is_some()
		{
			println!("{} tried to connect a second time from {}!", username, ip);
			let _ = stream.write_packet(Packet::Disconnect { reason: "Player already logged in.".to_string() }).await;
			return None;
		}
		let server_name = server.lock().await.config.name.clone();
		let server_motd = server.lock().await.config.motd.clone();
		let user_mode = if server.lock().await.config.user_data.ops.contains(&username, &ip.ip()) { ClientMode::Operator } else { ClientMode::Normal };
		println!("{}:{} is connecting from {}...", id, username, ip);
		if stream.write_packet(Packet::Identification { protocol: 7, name: server_name, data: server_motd, user_mode: user_mode.get_id() }).await.is_err() { return None; }
		if stream.write_packet(Packet::LevelStart).await.is_err() { return None; }


		// send/recv channel to receive packets asynchronously
		let (send, recv) = mpsc::unbounded_channel();


		// spawn the player. at this point the server starts sending things to the player
		if server.lock().await.spawn(id, ip, username.clone(), user_mode, send).is_err() { return None; };


		// we send the level to the player
		if Client::send_level(&mut stream, &server).await.is_none() { return None; };
		let (read, write) = stream.into_split();

		let write = Arc::new(Mutex::new(write));

		// finally we start actually sending the shit the server sends to the player
		let sender = tokio::spawn(Client::sender_client(recv, write.clone()));
		let receiver = tokio::spawn(Client::receiver_client(read, server.clone(), id, username, ip));

		Some((sender, receiver, id, write))
	}
	pub async fn handle_client(stream: TcpStream, ip: SocketAddr, server: Arc<Mutex<Server>>)
	{
		let op = Client::init_client(stream, ip, &server).await;
		if op.is_none() { return; }
		let (sender, receiver, id, write) = op.unwrap();

		let reason = match tokio::select!
		{
			r = sender => { r }
			r = receiver => { r }
		}
		{
			Ok(reason) => reason,
			Err(err) => format!("Panic: {}", err)
		};

		println!("player disconnected: \"{}\"", reason);
		let _ = write.lock().await.write_packet(Packet::Disconnect { reason }).await;

		server.lock().await.disconnected(id);
	}
}