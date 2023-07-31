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
			run: |server, id, _, _|
			{
				let rules = server.rules.clone();
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
				server.user_data.banned.add_username(username);
				Ok(())
			}
		});
		commands.register(Command
		{
			name: "banip",
			desc: "IP-bans a user from the server.",
			usage: "<username>",
			ops_only: true,
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
						server.user_data.banned.add_ip(client.ip.ip());
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
			run: |server, id, args, _|
			{
				let username = args.join(" ");
				if username.is_empty()
				{
					return Err("No username was provided.".to_string());
				}
				if server.user_data.banned.remove_username(&username)
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
				server.user_data.muted.add_username(username);
				Ok(())
			}
		});
		commands.register(Command
		{
			name: "unmute",
			desc: "Unmutes a user from the server.",
			usage: "<username>",
			ops_only: true,
			run: |server, id, args, _|
			{
				let username = args.join(" ");
				if username.is_empty()
				{
					return Err("No username was provided.".to_string());
				}
				if server.user_data.muted.remove_username(&username)
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
				server.user_data.restricted.add_username(username);
				Ok(())
			}
		});
		commands.register(Command
		{
			name: "unrestrict",
			desc: "Unrestricts a user from the server.",
			usage: "<username>",
			ops_only: true,
			run: |server, id, args, _|
			{
				let username = args.join(" ");
				if username.is_empty()
				{
					return Err("No username was provided.".to_string());
				}
				if server.user_data.restricted.remove_username(&username)
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