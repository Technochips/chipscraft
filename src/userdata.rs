use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::fs::File;
use std::io::Write;
use std::net::IpAddr;

#[derive(Default, Serialize, Deserialize)]
pub struct UserList
{
	usernames: Vec<String>,
	ips: Vec<IpAddr>,
	#[serde(skip)]
	file: String
}
impl UserList
{
	pub fn load(file: &str) -> UserList
	{
		let mut list = if let Ok(file) = File::open(file)
		{
			if let Ok(list) = serde_json::from_reader(file)
			{
				list
			}
			else
			{
				UserList::default()
			}
		}
		else
		{
			UserList::default()
		};
		list.file = file.to_string();
		list.save();
		list
	}
	pub fn save(&self)
	{
		if let (Ok(json), Ok(mut file)) = (serde_json::to_string_pretty(self), File::create(&self.file))
		{
			if file.write(json.as_bytes()).is_err()
			{
				println!("could not write to {}.", self.file);
			}
		}
		else
		{
			println!("could not create {}.", self.file);
		}
	}
	pub fn contains_username(&self, username: &String) -> bool
	{
		self.usernames.contains(username)
	}
	pub fn contains_ip(&self, ip: &IpAddr) -> bool
	{
		self.ips.contains(ip)
	}
	// do double check
	pub fn contains(&self, username: &String, ip: &IpAddr) -> bool
	{
		self.contains_ip(ip) || self.contains_username(username)
	}
	pub fn add_username(&mut self, username: String)
	{
		if !self.contains_username(&username)
		{
			self.usernames.push(username);
			self.save();
		}
	}
	pub fn add_ip(&mut self, ip: IpAddr)
	{
		if !self.contains_ip(&ip)
		{
			self.ips.push(ip);
			self.save();
		}
	}
	pub fn remove_username(&mut self, username: &String) -> bool
	{
		if self.contains_username(username)
		{
			self.usernames.retain(|i| *i != *username);
			self.save();
			true
		}
		else
		{
			false
		}
	}
	pub fn remove_ip(&mut self, ip: &IpAddr) -> bool
	{
		if self.contains_ip(ip)
		{
			self.ips.retain(|i| *i != *ip);
			self.save();
			true
		}
		else
		{
			false
		}
	}
}
pub struct UserData
{
	pub ops: UserList,
	pub banned: UserList,
	pub muted: UserList,
	pub restricted: UserList
}
impl UserData
{
	const OPS: &str = "ops.json";
	const BANNED: &str = "banned.json";
	const MUTED: &str = "muted.json";
	const RESTRICTED: &str = "restricted.json";
	pub fn load() -> UserData
	{
		UserData
		{
			ops: UserList::load(UserData::OPS),
			banned: UserList::load(UserData::BANNED),
			muted: UserList::load(UserData::MUTED),
			restricted: UserList::load(UserData::RESTRICTED)
		}
	}
}