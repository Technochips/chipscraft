use crate::block;
use crate::client::Client;
use crate::client::ClientMode;
use crate::level::Level;
use crate::packet::Packet;
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::error::SendError;

pub struct Server
{
	pub name: String,
	pub motd: String,
	pub max_clients: i8,
	pub clients: HashMap<i8, Client>,
	pub level: Level
}
impl Server
{
	pub fn new(max_clients: i8, level: Level, name: String, motd: String) -> Self
	{
		Self
		{
			name,
			motd,
			max_clients,
			clients: HashMap::new(),
			level
		}
	}
	pub fn first_free_space(&self) -> i8
	{
		let mut id = -1;
		for i in 0..self.max_clients
		{
			if self.clients.get(&i).is_none()
			{
				id = i;
				break;
			}
		}
		id
	}
	pub fn username_index(&self, username: &String) -> i8
	{
		for i in 0..self.max_clients
		{
			if let Some(client) = self.clients.get(&i)
			{
				if client.username.eq(username)
				{
					return i;
				}
			}
		}
		-1
	}
	pub fn disconnected(&mut self, id: i8)
	{
		let username = if let Some(client) = self.clients.get(&id)
		{
			client.username.clone()
		}
		else
		{
			return;
		};
		self.clients.remove(&id);
		self.broadcast_packet(id, Packet::Despawn { id: id });
		self.broadcast_system_message(id, format!("{} left", username));
	}
	pub fn send_packet(&mut self, cid: i8, packet: Packet) -> Result<(), SendError<Packet>>
	{
		if cid < 0
		{
			return Ok(());
		}
		if let Some(client) = self.clients.get(&cid)
		{
			let mut disconnect = false;
			let r = client.packet_sender.send(match packet
			{
				Packet::Spawn { id, name, x, y, z, yaw, pitch } => Packet::Spawn { id: if id == cid {-1} else {id}, name, x, y, z, yaw, pitch},
				Packet::SetPosAndLook { id, x, y, z, yaw, pitch } => Packet::SetPosAndLook { id: if id == cid {-1} else {id}, x, y, z, yaw, pitch },
				Packet::UpdatePosAndLook { id, x, y, z, yaw, pitch } => Packet::UpdatePosAndLook { id: if id == cid {-1} else {id}, x, y, z, yaw, pitch },
				Packet::UpdatePos { id, x, y, z } => Packet::UpdatePos { id: if id == cid {-1} else {id}, x, y, z },
				Packet::UpdateLook { id , yaw, pitch } => Packet::UpdateLook { id: if id == cid {-1} else {id}, yaw, pitch },
				Packet::Disconnect { reason } =>
				{
					disconnect = true;
					Packet::Disconnect { reason }
				}
				_ => packet
			});
			if disconnect || r.is_err()
			{
				self.disconnected(cid);
			}
			return r;
		}
		Err(SendError(packet))
	}
	pub fn set_block(&mut self, id: i8, x: i16, y: i16, z: i16, block: u8)
	{
		let mut place_block = false;
		if x >= 0 && y >= 0 && z >= 0 && x < self.level.size_x && y < self.level.size_y && z < self.level.size_z
		{
			if (block as usize) < block::BLOCKS.len()
			{
				let mode = if id < 0 { ClientMode::Operator } else if let Some(client) = self.clients.get(&id) { client.mode } else { ClientMode::Normal };
				let placed_block = &block::BLOCKS[block as usize];
				let replaced_block = &block::BLOCKS[self.level.get_block(x, y, z) as usize];
				if (!placed_block.place_op_only && !replaced_block.destroy_op_only) || mode == ClientMode::Operator
				{
					place_block = true;
				}
			}
		}
		let mut should_discard_original_placed_block = true;
		if place_block
		{
			for (xx,yy,zz,bblock) in self.level.place_block(x, y, z, block)
			{
				let id = if xx == x && yy == y && zz == z
				{
					should_discard_original_placed_block = false;
					if bblock == block
					{
						id
					}
					else
					{
						-1
					}
				}
				else
				{
					-1
				};
				self.broadcast_packet(id, Packet::SetBlock { x:xx, y:yy, z:zz, block:bblock });
			}
		}
		if should_discard_original_placed_block
		{
			let _ = self.send_packet(id, Packet::SetBlock { x, y, z, block: self.level.get_block(x, y, z) });
		}
	}
	pub fn move_player(&mut self, to_move: i8, mover: i8, x: i16, y: i16, z: i16, yaw: u8, pitch: u8)
	{
		let (position_changed, rotation_changed, x_diff, y_diff, z_diff);
		if let Some(client) = self.clients.get_mut(&to_move)
		{
			position_changed = client.x != x || client.y != y || client.z != z;
			if position_changed
			{
				// we do not want 
				x_diff = if let (Ok(x), Ok(client_x)) = (i8::try_from(x), i8::try_from(client.x)) {x.checked_sub(client_x)} else { None };
				y_diff = if let (Ok(y), Ok(client_y)) = (i8::try_from(y), i8::try_from(client.y)) {y.checked_sub(client_y)} else { None };
				z_diff = if let (Ok(z), Ok(client_z)) = (i8::try_from(z), i8::try_from(client.z)) {z.checked_sub(client_z)} else { None };
				client.x = x;
				client.y = y;
				client.z = z;
			}
			else
			{
				x_diff = Some(0);
				y_diff = Some(0);
				z_diff = Some(0);
			}
			rotation_changed = client.yaw != yaw || client.pitch != pitch;
			if rotation_changed
			{
				client.yaw = yaw;
				client.pitch = pitch;
			}
		}
		else
		{
			return;
		}
		if position_changed
		{
			if x_diff.is_none() || y_diff.is_none() || z_diff.is_none()
			{
				self.broadcast_packet(mover, Packet::SetPosAndLook { id: to_move, x, y, z, yaw, pitch });
			}
			else
			{
				if rotation_changed
				{
					self.broadcast_packet(mover, Packet::UpdatePosAndLook { id: to_move, x: x_diff.unwrap(), y: y_diff.unwrap(), z: z_diff.unwrap(), yaw, pitch });
				}
				else
				{
					self.broadcast_packet(mover, Packet::UpdatePos { id: to_move, x: x_diff.unwrap(), y: y_diff.unwrap(), z: z_diff.unwrap() });
				}
			}
		}
		else if rotation_changed
		{
			self.broadcast_packet(mover, Packet::UpdateLook { id: to_move, yaw, pitch });
		}
	}
	pub fn broadcast_packet(&mut self, oid: i8, packet: Packet)
	{
		for cid in 0..self.max_clients
		{
			if self.clients.contains_key(&cid)
			{
				let packet = packet.clone();
				if let Some(packet) = match packet
				{
					Packet::SetBlock { x:_, y:_, z:_, block:_ } => if oid == cid { None } else { Some(packet) },
					Packet::SetPosAndLook { id:_, x:_, y:_, z:_, yaw:_, pitch:_ } => if oid == cid { None } else { Some(packet) },
					Packet::UpdatePosAndLook { id:_, x:_, y:_, z:_, yaw:_, pitch:_ } => if oid == cid { None } else { Some(packet) },
					Packet::UpdatePos { id:_, x:_, y:_, z:_ } => if oid == cid { None } else { Some(packet) },
					Packet::UpdateLook { id:_, yaw:_, pitch:_ } => if oid == cid { None } else { Some(packet) },
					Packet::Disconnect { reason:_ } => if oid == cid { None } else { Some(packet) },
					Packet::Despawn { id:_ } => if oid == cid { None } else { Some(packet ) },
					_ => Some(packet)
				}
				{
					let _ = self.send_packet(cid, packet);
				}
			}
		}
	}
	fn broadcast_log_message(&mut self, id: i8, logid: i8, message: String)
	{
		println!("{}:{}", logid, message);
		self.broadcast_packet(id, Packet::Message { id: id, message: message.to_string() });
	}
	pub fn broadcast_message(&mut self, id: i8, message: String)
	{
		self.broadcast_log_message(id, id, message);
	}
	pub fn broadcast_system_message(&mut self, id: i8, message: String)
	{
		self.broadcast_log_message(-1, id, message);
	}
	pub fn spawn(&mut self, id: i8, username: String, packet_sender: UnboundedSender<Packet>) -> Result<(), SendError<Packet>>
	{
		let x = self.level.spawn_x;
		let y = self.level.spawn_y;
		let z = self.level.spawn_z;
		let yaw = self.level.spawn_yaw;
		let pitch = self.level.spawn_pitch;
		for i in 0..self.max_clients
		{
			if let Some(client) = self.clients.get(&i)
			{
				packet_sender.send(Packet::Spawn { id: i, name: client.username.clone(), x: client.x, y: client.y, z: client.z, yaw: client.yaw, pitch: client.pitch })?;
			}
		}
		self.clients.insert(id, Client { username: username.clone(), packet_sender, x, y, z, yaw, pitch, mode: ClientMode::Normal } );
		self.broadcast_system_message(id, format!("{} joined", username.clone()));
		self.broadcast_packet(id, Packet::Spawn { id: id, name: username, x, y, z, yaw, pitch});
		Ok(())
	}
	pub fn stop(&mut self)
	{
		self.broadcast_packet(-1, Packet::Disconnect { reason: "Stopping server".to_string() });
		if self.level.save("level.dat".to_string()).is_err()
		{
			println!("could not save.");
		}
	}
}