use crate::block::BLOCKS;
use crate::client::ClientMode;
use crate::server::Server;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Command
{
	pub name: &'static str, // name of the command e.g. 'tp'
	pub desc: &'static str, // what the command does e.g. 'Teleports the player'
	pub usage: &'static str, // how to use the command e.g. '/tp [x] [y] [z] OR /tp [player]'
	pub ops_only: bool, // if true, will be completely hidden from non-operators
	pub unmuted_only: bool,
	pub unrestricted_only: bool,
	pub run: fn(server: &mut Server, id: i8, args: Vec<&str>, mode: ClientMode) -> Result<(), String>
}
pub struct CommandList
{
	commands: HashMap<String, Command>
}
impl CommandList
{
	pub fn new() -> CommandList
	{
		let mut commands = CommandList { commands: HashMap::new() };
		commands.register(Command
		{
			name: "help",
			desc: "Shows help.",
			usage: "[command]",
			ops_only: false,
			unmuted_only: false,
			unrestricted_only: false,
			run: |server, id, args, mode|
			{
				if let Some(command) = args.get(0)
				{
					if let Some(command) = server.commands.get(&command.to_string())
					{
						server.send_message(-1, id, &format!("- /{0} -\n  {1}\n  Usage: /{0} {2}", command.name, command.desc, command.usage));
					}
					else
					{
						server.send_message(-1, id, "This command does not exist.");
					}
				}
				else
				{
					let mut str = "List of available commands:".to_string();
					for (name, desc) in server.commands.iter(mode == ClientMode::Operator).map(|command| (command.name.clone(), command.desc.clone())).collect::<Vec<_>>()
					{
						str.push_str(&format!("\n  /{} - {}", name, desc));
					}
					server.send_message(-1, id, &str);
				}
				Ok(())
			}
		});
		commands.register(Command
		{
			name: "rules",
			desc: "Shows rules.",
			usage: "",
			ops_only: false,
			unmuted_only: false,
			unrestricted_only: false,
			run: |server, id, _, _|
			{
				let rules = server.config.rules.clone();
				server.send_message(-1, id, &rules);
				Ok(())
			}
		});
		commands.register(Command
		{
			name: "kick",
			desc: "Kicks a user from the server.",
			usage: "<username>",
			ops_only: true,
			unmuted_only: false,
			unrestricted_only: false,
			run: |server, _, args, _|
			{
				let username = args.join(" ");
				if username.is_empty()
				{
					return Err("No username was provided.".to_string());
				}
				if let Some(id) = server.get_index_from_username(&username)
				{
					server.kick(id, "You have been kicked.".to_string());
					server.broadcast_system_message(-1, &format!("{} has been kicked.", username));
					return Ok(())
				}
				Err(format!("{} is not in the server.", username))
			}
		});
		commands.register(Command
		{
			name: "ban",
			desc: "Bans a user from the server.",
			usage: "<username>",
			ops_only: true,
			unmuted_only: false,
			unrestricted_only: false,
			run: |server, id, args, _|
			{
				let username = args.join(" ");
				if username.is_empty()
				{
					return Err("No username was provided.".to_string());
				}
				let msg = &format!("{} has been banned.", username);
				if let Some(bid) = server.get_index_from_username(&username)
				{
					if id == bid
					{
						return Err("You wouldn't want to ban yourself.".to_string());
					}
					server.kick(bid, "You have been banned.".to_string());
					server.broadcast_system_message(-1, msg);
				}
				else
				{
					server.send_message(-1, id, msg);
				}
				server.config.user_data.banned.add_username(username);
				Ok(())
			}
		});
		commands.register(Command
		{
			name: "banip",
			desc: "IP-bans a user from the server.",
			usage: "<username>",
			ops_only: true,
			unmuted_only: false,
			unrestricted_only: false,
			run: |server, id, args, _|
			{
				let username = args.join(" ");
				if username.is_empty()
				{
					return Err("No username was provided.".to_string());
				}
				if let Some(bid) = server.get_index_from_username(&username)
				{
					if id == bid
					{
						return Err("You wouldn't want to ban yourself.".to_string());
					}
					if let Some(client) = server.clients.get(&bid)
					{
						server.config.user_data.banned.add_ip(client.ip.ip());
						server.kick(bid, "You have been banned.".to_string());
						server.broadcast_system_message(-1, &format!("{} has been banned.", username));
						return Ok(())
					}
				}
				Err(format!("Could not find {}.", username))
			}
		});
		commands.register(Command
		{
			name: "unban",
			desc: "Unbans a user from the server.",
			usage: "<username>",
			ops_only: true,
			unmuted_only: false,
			unrestricted_only: false,
			run: |server, id, args, _|
			{
				let username = args.join(" ");
				if username.is_empty()
				{
					return Err("No username was provided.".to_string());
				}
				if server.config.user_data.banned.remove_username(&username)
				{	
					server.send_message(-1, id, &format!("{} has been unbanned.", username));
				}
				else
				{
					server.send_message(-1, id, &format!("{} is not in the ban list.", username));
				}
				Ok(())
			}
		});
		commands.register(Command
		{
			name: "mute",
			desc: "Mutes a user from the server.",
			usage: "<username>",
			ops_only: true,
			unmuted_only: true,
			unrestricted_only: false,
			run: |server, id, args, _|
			{
				let username = args.join(" ");
				if username.is_empty()
				{
					return Err("No username was provided.".to_string());
				}
				if let Some(id) = server.get_index_from_username(&username)
				{
					server.send_message(-1, id, "You have been muted.");
				}
				server.send_message(-1, id, &format!("{} has been muted.", username));
				server.config.user_data.muted.add_username(username);
				Ok(())
			}
		});
		commands.register(Command
		{
			name: "unmute",
			desc: "Unmutes a user from the server.",
			usage: "<username>",
			ops_only: true,
			unmuted_only: true,
			unrestricted_only: false,
			run: |server, id, args, _|
			{
				let username = args.join(" ");
				if username.is_empty()
				{
					return Err("No username was provided.".to_string());
				}
				if server.config.user_data.muted.remove_username(&username)
				{
					if let Some(id) = server.get_index_from_username(&username)
					{
						server.send_message(-1, id, "You have been unmuted.");
					}
					server.send_message(-1, id, &format!("{} has been unmutted.", username));
				}
				else
				{
					server.send_message(-1, id, &format!("{} is not in the mute list.", username));
				}
				Ok(())
			}
		});
		commands.register(Command
		{
			name: "restrict",
			desc: "Restricts a user from the server.",
			usage: "<username>",
			ops_only: true,
			unmuted_only: false,
			unrestricted_only: true,
			run: |server, id, args, _|
			{
				let username = args.join(" ");
				if username.is_empty()
				{
					return Err("No username was provided.".to_string());
				}
				if let Some(id) = server.get_index_from_username(&username)
				{
					server.send_message(-1, id, "You have been restricted.");
				}
				server.send_message(-1, id, &format!("{} has been restricted.", username));
				server.config.user_data.restricted.add_username(username);
				Ok(())
			}
		});
		commands.register(Command
		{
			name: "unrestrict",
			desc: "Unrestricts a user from the server.",
			usage: "<username>",
			ops_only: true,
			unmuted_only: false,
			unrestricted_only: true,
			run: |server, id, args, _|
			{
				let username = args.join(" ");
				if username.is_empty()
				{
					return Err("No username was provided.".to_string());
				}
				if server.config.user_data.restricted.remove_username(&username)
				{
					if let Some(id) = server.get_index_from_username(&username)
					{
						server.send_message(-1, id, "You have been unrestricted.");
					}
					server.send_message(-1, id, &format!("{} has been unrestricted.", username));
				}
				else
				{
					server.send_message(-1, id, &format!("{} is not in the restriction list.", username));
				}
				Ok(())
			}
		});
		commands.register(Command
		{
			name: "tp",
			desc: "Teleports yourself to a target. It can either be a position, or a user.",
			usage: "<x> <y> <z> || <username>",
			ops_only: false,
			unmuted_only: false,
			unrestricted_only: false,
			run: |server, id, args, _|
			{
				if let (Some(x),Some(y),Some(z)) = (args.get(0), args.get(1), args.get(2))
				{
					if let (Ok(x),Ok(y),Ok(z)) = (x.parse::<i16>(),y.parse::<i16>(),z.parse::<i16>())
					{
						if x <= i16::MAX / 32 && x >= i16::MIN / 32 && y <= i16::MAX / 32 - 1 && y >= i16::MIN / 32 && z < i16::MAX / 32 && z >= i16::MIN / 32
						{
							let client = server.clients.get(&id).unwrap();
							server.move_player(id, -1, x*32+16, y*32+61, z*32+16, client.yaw, client.pitch);
							server.send_message(-1, id, &format!("Teleported to coordinate {} {} {}.", x, y, z));
							return Ok(());
						}
					}
				}
				if let Some(target) = args.get(0)
				{
					if let Some(t_client) = server.get_client_from_username(target)
					{
						server.move_player(id, -1, t_client.x, t_client.y, t_client.z, t_client.yaw, t_client.pitch);
						server.send_message(-1, id, &format!("Teleported to player {}.", target));
						return Ok(());
					}
				}
				Err("Invalid target.".to_string())
			}
		});
		commands.register(Command
		{
			name: "tpo",
			desc: "Teleport another player to a target. It can either be a position, or a user.",
			usage: "<username> ( <x> <y> <z> || <username> )",
			ops_only: true,
			unmuted_only: false,
			unrestricted_only: false,
			run: |server, fid, args, _|
			{
				let fusername = server.clients.get(&fid).unwrap().username.clone();
				if let Some(username) = args.get(0)
				{
					if let Some(id) = server.get_index_from_username(username)
					{
						if let (Some(x),Some(y),Some(z)) = (args.get(1), args.get(2), args.get(3))
						{
							if let (Ok(x),Ok(y),Ok(z)) = (x.parse::<i16>(),y.parse::<i16>(),z.parse::<i16>())
							{
								if x <= i16::MAX / 32 && x >= i16::MIN / 32 && y <= i16::MAX / 32 - 1 && y >= i16::MIN / 32 && z < i16::MAX / 32 && z >= i16::MIN / 32
								{
									let client = server.clients.get(&id).unwrap();
									server.move_player(id, -1, x*32+16, y*32+61, z*32+16, client.yaw, client.pitch);
									server.send_message(-1, fid, &format!("Teleported {} to coordinate {} {} {}.", username, x, y, z));
									server.send_message(-1, id, &format!("Teleported by {} to coordinate {} {} {}.", fusername, x, y, z));
									return Ok(());
								}
							}
						}
						if let Some(target) = args.get(1)
						{
							if let Some(client) = server.get_client_from_username(target)
							{
								server.move_player(id, -1, client.x, client.y, client.z, client.yaw, client.pitch);
								server.send_message(-1, fid, &format!("Teleported {} to player {}", username, target));
								server.send_message(-1, id, &format!("Teleported by {} to player {}.", fusername, target));
								return Ok(());
							}
						}
						return Err("Invalid target.".to_string());
					}
				}
				Err("Could not find user to teleport.".to_string())
			}
		});
		commands.register(Command
		{
			name: "save",
			desc: "Saves the world. This also creates a backup.",
			usage: "",
			ops_only: true,
			unmuted_only: false,
			unrestricted_only: false,
			run: |server, fid, _, _|
			{
				server.broadcast_message(-1, "Saving world...");
				if let Err(e) = server.level.save()
				{
					server.send_message(-1, fid, &e);
				}
				server.broadcast_message(-1, "Done.");
				Ok(())
			}
		});
		commands.register(Command
		{
			name: "msg",
			desc: "Sends a message to a player.",
			usage: "<player> <message>",
			ops_only: false,
			unmuted_only: true,
			unrestricted_only: false,
			run: |server, fid, args, _|
			{
				let mut args = args.iter();
				let sender = if fid >= 0 {
					if let Some(sender) = server.clients.get(&fid)
					{
						(sender.username).clone()
					}
					else
					{
						return Err("Major lack of a username.".to_string());
					}
				}
				else
				{
					"Console".to_string()
				};
				if let Some(username) = args.next()
				{
					if let Some(id) = server.get_index_from_username(username)
					{
						let msg = args.map(|x| *x).collect::<Vec<&str>>().join(" ");
						if msg.is_empty()
						{
							return Err("Please type in a message to send.".to_string());
						}
						server.send_message(fid, fid, &format!("&7to <{}> {}", username, msg));
						server.send_message(fid, id, &format!("&7<{}> {}", sender, msg));
						return Ok(());
					}
				}
				Err("Unknown player.".to_string())
			}
		});
		commands.register(Command
		{
			name: "reload-config",
			desc: "Reload the configuration files.",
			usage: "",
			ops_only: true,
			unmuted_only: false,
			unrestricted_only: false,
			run: |server, id, _, _|
			{
				if let Err(e) = server.reload_config()
				{
					Err(format!("Could not reload configuration file: {}", e))
				}
				else
				{
					server.send_message(-1, id, "Configuration file was reloaded");
					Ok(())
				}
			}
		});
		commands.register(Command
		{
			name: "cuboid",
			desc: "Creates a cube",
			usage: "<x1> <y1> <z1> <x2> <y2> <z2> <block>",
			ops_only: true,
			unmuted_only: false,
			unrestricted_only: true,
			run: |server, id, args, _|
			{
				let mut args = args.iter();
				if let (Some(x1), Some(y1), Some(z1), Some(x2), Some(y2), Some(z2), Some(block)) = (args.next(), args.next(), args.next(), args.next(), args.next(), args.next(), args.next())
				{
					if let (Ok(x1), Ok(y1), Ok(z1), Ok(x2), Ok(y2), Ok(z2), Ok(block)) = (x1.parse::<i16>(), y1.parse::<i16>(), z1.parse::<i16>(), x2.parse::<i16>(), y2.parse::<i16>(), z2.parse::<i16>(), block.parse::<u8>())
					{
						if block >= BLOCKS.len() as u8
						{
							return Err("Invalid block ID.".to_string());
						}
						if x1 < 0 || x1 >= server.level.size_x || y1 < 0 || y1 >= server.level.size_y || z1 < 0 || z1 >= server.level.size_z
						|| x2 < 0 || x2 >= server.level.size_x || y2 < 0 || y2 >= server.level.size_y || z2 < 0 || z2 >= server.level.size_z
						{
							return Err("Block out of bound.".to_string());
						}
						for x in i16::min(x1,x2)..=i16::max(x1,x2)
						{
							for y in i16::min(y1,y2)..=i16::max(y1,y2)
							{
								for z in i16::min(z1,z2)..=i16::max(z1,z2)
								{
									server.set_block(id, x, y, z, block, false);
								}
							}
						}
						return Ok(());
					}
				}
				Err("Invalid arguments.".to_string())
			}
		});
		commands
	}
	pub fn register(&mut self, command: Command)
	{
		self.commands.insert(command.name.to_string(), command);
	}
	pub fn get(&self, name: &String) -> Option<&Command>
	{
		self.commands.get(name)
	}
	pub fn iter(&self, ops: bool) -> impl Iterator<Item = &Command>
	{
		return self.commands.values().filter(move |v| !v.ops_only || ops);
	}
}