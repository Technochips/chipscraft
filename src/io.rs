use async_trait::async_trait;
use codepage_437::FromCp437;
use codepage_437::CP437_WINGDINGS;
use codepage_437::IntoCp437;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;

use crate::packet::Packet;

pub const STRING_LEN: usize = 64;
pub const ARRAY_LEN: usize = 1024;

#[async_trait]
pub trait AsyncReadClassicExt: AsyncRead
{
	async fn read_string(&mut self) -> Result<String, std::io::Error>;
	async fn read_array(&mut self) -> Result<Vec<u8>, std::io::Error>;
	async fn read_packet(&mut self) -> Result<Packet, std::io::Error>;
}
#[async_trait]
impl<R: AsyncRead + Unpin + Send> AsyncReadClassicExt for R
{
	async fn read_string(&mut self) -> Result<String, std::io::Error>
	{
		let mut buffer = vec![0u8; 64];
		self.read_exact(&mut buffer).await?;
		let mut length = 0;
		for i in (0..64).rev()
		{
			if buffer[i] != 0x20 && buffer[i] != 0x00
			{
				length = i+1;
				break;
			}
		}
		buffer.truncate(length);
		Ok(String::from_cp437(buffer, &CP437_WINGDINGS))
	}
	async fn read_array(&mut self) -> Result<Vec<u8>, std::io::Error>
	{
		let mut buffer = vec![0u8; 1024];
		self.read_exact(&mut buffer).await?;
		Ok(buffer)
	}
	async fn read_packet(&mut self) -> Result<Packet, std::io::Error>
	{
		match self.read_u8().await?
		{
			0x00 => Ok(Packet::Identification
				{
					protocol: self.read_u8().await?,
					name: self.read_string().await?,
					data: self.read_string().await?,
					usertype: self.read_u8().await?
				}),
			0x01 => Ok(Packet::Ping),
			0x02 => Ok(Packet::LevelStart),
			0x03 => Ok(Packet::LevelData
				{
					length: self.read_i16().await?,
					data: self.read_array().await?,
					percentage: self.read_u8().await?
				}),
			0x04 => Ok(Packet::LevelSize
				{
					x: self.read_i16().await?,
					y: self.read_i16().await?,
					z: self.read_i16().await?
				}),
			0x05 => Ok(Packet::PlaceBlock
				{
					x: self.read_i16().await?,
					y: self.read_i16().await?,
					z: self.read_i16().await?,
					mode: self.read_u8().await?,
					block: self.read_u8().await?
				}),
			0x06 => Ok(Packet::SetBlock
				{
					x: self.read_i16().await?,
					y: self.read_i16().await?,
					z: self.read_i16().await?,
					block: self.read_u8().await?
				}),
			0x07 => Ok(Packet::Spawn
				{
					id: self.read_i8().await?,
					name: self.read_string().await?,
					x: self.read_i16().await?,
					y: self.read_i16().await?,
					z: self.read_i16().await?,
					yaw: self.read_u8().await?,
					pitch: self.read_u8().await?
				}),
			0x08 => Ok(Packet::SetPosAndLook
				{
					id: self.read_i8().await?,
					x: self.read_i16().await?,
					y: self.read_i16().await?,
					z: self.read_i16().await?,
					yaw: self.read_u8().await?,
					pitch: self.read_u8().await?
				}),
			0x09 => Ok(Packet::UpdatePosAndLook
				{
					id: self.read_i8().await?,
					x: self.read_i8().await?,
					y: self.read_i8().await?,
					z: self.read_i8().await?,
					yaw: self.read_u8().await?,
					pitch: self.read_u8().await?
				}),
			0x0a => Ok(Packet::UpdatePos
				{
					id: self.read_i8().await?,
					x: self.read_i8().await?,
					y: self.read_i8().await?,
					z: self.read_i8().await?
				}),
			0x0b => Ok(Packet::UpdateLook
				{
					id: self.read_i8().await?,
					yaw: self.read_u8().await?,
					pitch: self.read_u8().await?
				}),
			0x0c => Ok(Packet::Despawn
				{
					id: self.read_i8().await?
				}),
			0x0d => Ok(Packet::Message
				{
					id: self.read_i8().await?,
					message: self.read_string().await?
				}),
			0x0e => Ok(Packet::Disconnect
				{
					reason: self.read_string().await?
				}),
			0x0f => Ok(Packet::UpdateUserType
				{
					usertype: self.read_u8().await?
				}),
			_ => Ok(Packet::Unknown)
		}
	}
}
#[async_trait]
pub trait AsyncWriteClassicExt: AsyncWrite
{
	async fn write_string(&mut self, v: String) -> Result<(), std::io::Error>;
	async fn write_array(&mut self, v: Vec<u8>) -> Result<(), std::io::Error>;
	async fn write_packet(&mut self, v: Packet) -> Result<(), std::io::Error>;
}
#[async_trait]
impl<W: AsyncWrite + Unpin + Send> AsyncWriteClassicExt for W
{
	async fn write_string(&mut self, v: String) -> Result<(), std::io::Error>
	{
		if let Ok(mut v) = v.into_cp437(&CP437_WINGDINGS)
		{
			let mut l = v.len();
			if l > STRING_LEN
			{
				v.truncate(STRING_LEN);
				for i in STRING_LEN-3..STRING_LEN
				{
					v[i] = 0x2e;
				}
				l = STRING_LEN;
			}
			else if l < STRING_LEN
			{
				v.extend(std::iter::repeat(b' ').take(STRING_LEN-l));
			}
			for i in (0..l).rev()
			{
				if v[i] != b' '
				{
					if v[i] == b'&'
					{
						v[i] = b'%'
					}
					break;
				}
			}
			return self.write_all(&v).await;
		}
		Err(std::io::Error::new(std::io::ErrorKind::Other, "could not convert string to cp437"))
	}
	async fn write_array(&mut self, mut v: Vec<u8>) -> Result<(), std::io::Error>
	{
		let l = v.len();
		if l > ARRAY_LEN
		{
			return Err(std::io::Error::new(std::io::ErrorKind::Other, "data too long!"));
		}
		else if l < ARRAY_LEN
		{
			v.extend(std::iter::repeat(0x00).take(ARRAY_LEN-l));
		}
		assert!(v.len() == ARRAY_LEN);
		return self.write_all(&v).await;
	}
	async fn write_packet(&mut self, v: Packet) -> Result<(), std::io::Error> {
		match v {
			Packet::Identification {
				protocol,
				name,
				data,
				usertype,
			} => {
				self.write_u8(0x00).await?;
				self.write_u8(protocol).await?;
				self.write_string(name).await?;
				self.write_string(data).await?;
				self.write_u8(usertype).await?;
				Ok(())
			}
			Packet::Ping => {
				self.write_u8(0x01).await?;
				Ok(())
			}
			Packet::LevelStart => {
				self.write_u8(0x02).await?;
				Ok(())
			}
			Packet::LevelData { length, data, percentage } => {
				self.write_u8(0x03).await?;
				self.write_i16(length).await?;
				self.write_array(data).await?;
				self.write_u8(percentage).await?;
				Ok(())
			}
			Packet::LevelSize { x, y, z } => {
				self.write_u8(0x04).await?;
				self.write_i16(x).await?;
				self.write_i16(y).await?;
				self.write_i16(z).await?;
				Ok(())
			}
			Packet::PlaceBlock { x, y, z, mode, block } => {
				self.write_u8(0x05).await?;
				self.write_i16(x).await?;
				self.write_i16(y).await?;
				self.write_i16(z).await?;
				self.write_u8(mode).await?;
				self.write_u8(block).await?;
				Ok(())
			}
			Packet::SetBlock { x, y, z, block } => {
				self.write_u8(0x06).await?;
				self.write_i16(x).await?;
				self.write_i16(y).await?;
				self.write_i16(z).await?;
				self.write_u8(block).await?;
				Ok(())
			}
			Packet::Spawn { id, name, x, y, z, yaw, pitch, } => {
				self.write_u8(0x07).await?;
				self.write_i8(id).await?;
				self.write_string(name).await?;
				self.write_i16(x).await?;
				self.write_i16(y).await?;
				self.write_i16(z).await?;
				self.write_u8(yaw).await?;
				self.write_u8(pitch).await?;
				Ok(())
			}
			Packet::SetPosAndLook { id, x, y, z, yaw, pitch, } => {
				self.write_u8(0x08).await?;
				self.write_i8(id).await?;
				self.write_i16(x).await?;
				self.write_i16(y).await?;
				self.write_i16(z).await?;
				self.write_u8(yaw).await?;
				self.write_u8(pitch).await?;
				Ok(())
			}
			Packet::UpdatePosAndLook { id, x, y, z, yaw, pitch, } => {
				self.write_u8(0x09).await?;
				self.write_i8(id).await?;
				self.write_i8(x).await?;
				self.write_i8(y).await?;
				self.write_i8(z).await?;
				self.write_u8(yaw).await?;
				self.write_u8(pitch).await?;
				Ok(())
			}
			Packet::UpdatePos { id, x, y, z } => {
				self.write_u8(0x0a).await?;
				self.write_i8(id).await?;
				self.write_i8(x).await?;
				self.write_i8(y).await?;
				self.write_i8(z).await?;
				Ok(())
			}
			Packet::UpdateLook { id, yaw, pitch } => {
				self.write_u8(0x0b).await?;
				self.write_i8(id).await?;
				self.write_u8(yaw).await?;
				self.write_u8(pitch).await?;
				Ok(())
			}
			Packet::Despawn { id } => {
				self.write_u8(0x0c).await?;
				self.write_i8(id).await?;
				Ok(())
			}
			Packet::Message { id, message } => {
				self.write_u8(0x0d).await?;
				self.write_i8(id).await?;
				self.write_string(message).await?;
				Ok(())
			}
			Packet::Disconnect { reason } => {
				self.write_u8(0x0e).await?;
				self.write_string(reason).await?;
				Ok(())
			}
			Packet::UpdateUserType { usertype } => {
				self.write_u8(0x0f).await?;
				self.write_u8(usertype).await?;
				Ok(())
			}
			_ => Err(std::io::Error::new(std::io::ErrorKind::Other, "tried to send unknown packet")),
		}
	}
}