use crate::io;
use crate::io::AsyncReadClassicExt;
use crate::io::AsyncWriteClassicExt;
use crate::level::SaveType;
use crate::packet::Packet;
use crate::server::Server;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;

#[derive(PartialEq, Clone, Copy)]
pub enum ClientMode
{
	Normal,
	Operator
}
pub struct Client
{
	pub username: String,
	pub x: i16,
	pub y: i16,
	pub z: i16,
	pub pitch: u8,
	pub yaw: u8,
	pub mode: ClientMode,
	pub packet_sender: UnboundedSender<Packet>,
}
impl Client
{
	pub async fn sender_client(mut recv: UnboundedReceiver<Packet>, mut write: OwnedWriteHalf)
	{
		while let Some(packet) = recv.recv().await
		{
			if write.write_packet(packet).await.is_err()
			{
				return;
			}
		}
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
	pub async fn init_client(mut stream: TcpStream, ip: SocketAddr, server: &Arc<Mutex<Server>>) -> Option<(i8, String, OwnedReadHalf)>
	{
		let username;
		if let Ok(Packet::Identification { protocol, name, data: key, usertype: _ }) = stream.read_packet().await
		{
			if protocol != 0x07
			{
				return None;
			}
			if server.lock().await.verify_players
			{
				if key != format!("{:?}", md5::compute(format!("{}{}", server.lock().await.salt.clone(), name)))
				{
					println!("{} tried to connect but couldn't verify from {}", name, ip);
					let _ = stream.write_packet(Packet::Disconnect { reason: "Could not verify".to_string() }).await;
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
		if id < 0
		{
			println!("server too full. {} tried to connect from {}", username, ip);
			// we do not really care if this packets fails to get sent.
			let _ = stream.write_packet(Packet::Disconnect { reason: "Too many players".to_string() }).await;
			return None;
		}
		if server.lock().await.username_index(&username) > -1
		{
			println!("{} tried to connect a second time from {}!", username, ip);
			let _ = stream.write_packet(Packet::Disconnect { reason: "Player already logged in".to_string() }).await;
			return None;
		}
		let server_name = server.lock().await.name.clone();
		let server_motd = server.lock().await.motd.clone();
		println!("{}:{} is connecting from {}...", id, username, ip);
		if stream.write_packet(Packet::Identification { protocol: 7, name: server_name, data: server_motd, usertype: 0 }).await.is_err() { return None; }
		if stream.write_packet(Packet::LevelStart).await.is_err() { return None; }
		// send/recv channel to receive packets asynchronously
		let (send, recv) = mpsc::unbounded_channel();
		// spawn the player. at this point the server starts sending things to the player
		if server.lock().await.spawn(id, username.clone(), send).is_err() { return None; };
		// we send the level to the player
		if Client::send_level(&mut stream, &server).await.is_none() { return None; };
		let (read, write) = stream.into_split();
		// finally we start actually sending the shit the server sends to the player
		tokio::spawn(Client::sender_client(recv, write));
		Some((id, username, read))
	}
	pub async fn handle_client(stream: TcpStream, ip: SocketAddr, server: Arc<Mutex<Server>>)
	{
		let op = Client::init_client(stream, ip, &server).await;
		if op.is_none() { return; }
		let (id, username, mut read) = op.unwrap();

		while let Ok(packet) = read.read_packet().await
		{
			if !match packet
			{
				Packet::PlaceBlock { x, y, z, block, mode } =>
				{
					server.lock().await.set_block(id, x, y, z, if mode == 0 { 0 } else { block });
					true
				}
				Packet::SetPosAndLook { id: negativeone, x, y, z, yaw, pitch } =>
				{
					if negativeone == -1
					{
						server.lock().await.move_player(id, id, x, y, z, yaw, pitch);
						true
					}
					else
					{
						false
					}
				}
				Packet::Message { id: negativeone, message } =>
				{
					if negativeone == -1
					{
						server.lock().await.broadcast_message(id, format!("<{}> {}", username, message));
						true
					}
					else
					{
						false
					}
				}
				_ => false
			}
			{
				println!("warning: {} sent an invalid packet", username);
				break;
			}
		}
		server.lock().await.disconnected(id);
	}
}