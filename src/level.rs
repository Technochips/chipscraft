use byteorder::NetworkEndian;
use byteorder::ReadBytesExt;
use byteorder::WriteBytesExt;
use chrono::Local;
use crate::block;
use crate::noise::CombinedNoise;
use crate::noise::Noise;
use crate::noise::OctaveNoise;
use crate::noise::PerlinNoise;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::f64::consts::PI;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;

#[derive(Clone)]
pub struct Level
{
	pub name: String,
	pub size_x: i16,
	pub size_y: i16,
	pub size_z: i16,
	pub spawn_x: i16,
	pub spawn_y: i16,
	pub spawn_z: i16,
	pub spawn_yaw: u8,
	pub spawn_pitch: u8,
	pub changed: bool,
	data: Vec<u8>
}
#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum GenerationType
{
	Empty,
	Flat,
	Vanilla
}
pub enum SaveType
{
	Network,
	Disk
}
// A level implements everything that's permanently saved into a level.
// This means its size, and the blocks inside.
impl Level
{
	pub fn new(name: String) -> Self
	{
		Self {
			name,
			size_x: 0,
			size_y: 0,
			size_z: 0,
			data: vec![0; 0],
			spawn_x: 0,
			spawn_y: 0,
			spawn_z: 0,
			spawn_yaw: 0,
			spawn_pitch: 0,
			changed: false
		}
	}
	pub fn generate(&mut self, size_x: i16, size_y: i16, size_z: i16, gen_type: GenerationType, seed: u64) -> Result<(), String>
	{
		println!("generating...");
		if size_x < 0 || size_y < 0 || size_z < 0
		{
			return Err("impossible size".to_string());
		}
		self.size_x = size_x;
		self.size_y = size_y;
		self.size_z = size_z;
		self.data = vec![0; size_x as usize * size_y as usize * size_z as usize];
		
		match gen_type
		{
			GenerationType::Empty => {}
			GenerationType::Flat =>
			{
				let floor = self.size_y/2;
				for y in 0..=floor
				{
					let b = if y == 0
					{ 10 } // lava
					else if y < floor - 3
					{ 1 } // stone
					else if y < floor
					{ 3 } // dirt
					else
					{ 2 }; // grass
					for x in 0..self.size_x
					{
						for z in 0..self.size_z
						{
							self.set_block(x, y, z, b);
						}
					}
				}
			}
			// vanilla generation stuff mostly taken from ClassiCube
			GenerationType::Vanilla =>
			{
				let mut rng = StdRng::seed_from_u64(seed);
				let mut height_map = vec![0i16;self.size_x as usize*self.size_z as usize];
				println!("raising...");
				let noise1 = CombinedNoise::new(OctaveNoise::new(PerlinNoise::new(&mut rng),8),OctaveNoise::new(PerlinNoise::new(&mut rng),8));
				let noise2 = CombinedNoise::new(OctaveNoise::new(PerlinNoise::new(&mut rng),8),OctaveNoise::new(PerlinNoise::new(&mut rng),8));
				let noise3 = OctaveNoise::new(PerlinNoise::new(&mut rng), 6);
				for x in 0..size_x
				{
					for z in 0..size_z
					{
						let height = noise1.get(x as f64*1.3,z as f64*1.3)/6.0-4.0;
						let mut height = if noise3.get(x as f64,z as f64) / 8.0 > 0.0
						{
							height
						}
						else
						{
							height.max(noise2.get(x as f64*1.3,z as f64*1.3)/5.0+6.0)
						} / 2.0;
						if height < 0.0
						{
							height *= 0.8;
						}
						height_map[x as usize + z as usize * self.size_x as usize] = (height as i16 + size_y/2).clamp(0, size_y-1);
					}
				}
				println!("eroding...");
				let noise1 = CombinedNoise::new(OctaveNoise::new(PerlinNoise::new(&mut rng),8),OctaveNoise::new(PerlinNoise::new(&mut rng),8));
				let noise2 = CombinedNoise::new(OctaveNoise::new(PerlinNoise::new(&mut rng),8),OctaveNoise::new(PerlinNoise::new(&mut rng),8));
				for x in 0..size_x
				{
					for z in 0..size_z
					{
						let c = x as usize + z as usize * self.size_x as usize;
						let a = noise1.get(x as f64*2.0, z as f64*2.0) / 8.0;
						let b = if noise2.get(x as f64*2.0, z as f64*2.0) > 0.0 { 1 } else { 0 };
						if a > 2.0
						{
							height_map[c] = ((height_map[c] - b) / 2) * 2 + b;
						}
					}
				}
				println!("soiling...");
				let noise1 = PerlinNoise::new(&mut rng);
				for x in 0..size_x
				{
					for z in 0..size_z
					{
						let dirt_thickness = (noise1.get(x as f64,z as f64) / 24.0 - 4.0) as i16;
						let dirt_transition = height_map[x as usize + z as usize * self.size_x as usize];
						let stone_transition = dirt_transition + dirt_thickness;
						self.set_block(x, 0, z, 10); // lava
						for y in 1..=stone_transition
						{
							self.set_block(x, y, z, 1);
						}
						for y in stone_transition+1..=dirt_transition
						{
							self.set_block(x, y, z, 3);
						}
					}
				}
				println!("carving...");
				for _ in 0..size_x as usize * size_y as usize * size_z as usize / 8192
				{
					let mut cave_x = rng.gen_range(0..size_x) as f64;
					let mut cave_y = rng.gen_range(0..size_y) as f64;
					let mut cave_z = rng.gen_range(0..size_z) as f64;
					let cave_len = (rng.gen::<f64>() + rng.gen::<f64>() * 200.0) as i16;

					let mut theta = rng.gen::<f64>() * PI * 2.0;
					let mut deltatheta = 0.0;
					let mut phi = rng.gen::<f64>() * PI * 2.0;
					let mut deltaphi = 0.0;

					let cave_radius = rng.gen::<f64>() * rng.gen::<f64>();

					for len in 0..cave_len
					{
						cave_x += theta.sin() * phi.cos();
						cave_y += theta.cos() * phi.cos();
						cave_z += phi.sin();

						theta += deltatheta * 0.2;
						deltatheta = (deltatheta * 0.9) + rng.gen::<f64>() - rng.gen::<f64>();
						phi = phi/2.0 + deltaphi/4.0;
						deltaphi = (deltaphi * 0.75) + rng.gen::<f64>() - rng.gen::<f64>();

						if rng.gen::<f64>() >= 0.25
						{
							let center_x = cave_x + (rng.gen_range(0..4) as f64 - 2.0) * 0.2;
							let center_y = cave_y + (rng.gen_range(0..4) as f64 - 2.0) * 0.2;
							let center_z = cave_z + (rng.gen_range(0..4) as f64 - 2.0) * 0.2;

							let mut radius = (size_y as f64 - center_y) / size_y as f64;
							radius = 1.2 + (radius * 3.5 + 1.0) * cave_radius;
							radius *= (len as f64 * PI / cave_len as f64).sin();
							self.fill_oblate_spheroid(center_x, center_y, center_z, 0, radius);
						}
					}
				}
				for (block, abundance) in [(14, 0.5), (15, 0.7), (16, 0.9)]
				{
					for _ in 0..(size_x as f64 * size_y as f64 * size_z as f64 * abundance / 16384.0) as usize
					{
						let mut vein_x = rng.gen_range(0..size_x) as f64;
						let mut vein_y = rng.gen_range(0..size_y) as f64;
						let mut vein_z = rng.gen_range(0..size_z) as f64;
						let vein_len = (rng.gen::<f64>() * rng.gen::<f64>() * 75.0 * abundance) as i16;

						let mut theta = rng.gen::<f64>() * PI * 2.0;
						let mut deltatheta = 0.0;
						let mut phi = rng.gen::<f64>() * PI * 2.0;
						let mut deltaphi = 0.0;

						for len in 0..vein_len
						{
							vein_x += theta.sin() * phi.cos();
							vein_y += theta.cos() * phi.cos();
							vein_z += phi.sin();

							theta = deltatheta * 0.2;
							deltatheta = (deltatheta * 0.9) + rng.gen::<f64>() - rng.gen::<f64>();
							phi = phi / 2.0 + deltaphi / 4.0;
							deltaphi = (deltaphi * 0.9) + rng.gen::<f64>() - rng.gen::<f64>();

							let radius = abundance * (len as f64 * PI / vein_len as f64).sin() + 1.0;

							self.fill_oblate_spheroid(vein_x, vein_y, vein_z, block, radius);
						}
					}
				}
				println!("watering...");
				let y = self.size_y / 2 - 1;
				for x in 0..size_x
				{
					self.flood_fill(x, y, 0, 9);
					self.flood_fill(x, y, self.size_z-1, 9);
				}
				for z in 0..size_z
				{
					self.flood_fill(0, y, z, 9);
					self.flood_fill(self.size_x-1, y, z, 9);
				}
				for _ in 0..self.size_x as usize * self.size_z as usize / 800
				{
					self.flood_fill(rng.gen_range(0..self.size_x), y - rng.gen_range(1..3), rng.gen_range(0..self.size_z), 9);
				}
				println!("melting...");
				for _ in 0..self.size_x as usize * self.size_y as usize * self.size_z as usize / 20000
				{
					self.flood_fill(rng.gen_range(0..self.size_x), ((y - 3) as f64 * rng.gen::<f64>() * rng.gen::<f64>()) as i16, rng.gen_range(0..self.size_z), 11);
				}
				println!("growing...");
				let noise1 = OctaveNoise::new(PerlinNoise::new(&mut rng), 8);
				let noise2 = OctaveNoise::new(PerlinNoise::new(&mut rng), 8);
				for x in 0..size_x
				{
					for z in 0..size_z
					{
						let sand_chance = noise1.get(x as f64,z as f64) > 8.0;
						let gravel_chance = noise2.get(x as f64,z as f64) > 12.0;

						let y = height_map[x as usize + z as usize * self.size_x as usize];
						let block_above = self.get_block(x, y + 1, z);

						match block_above
						{
							9 =>
							{
								if gravel_chance
								{
									self.set_block(x, y, z, 13);
								}
							}
							0 =>
							{
								if y <= self.size_y/2 && sand_chance
								{
									self.set_block(x, y, z, 12)
								}
								else
								{
									self.set_block(x, y, z, 2);
								}
							}
							_ => {}
						}
					}
				}
				println!("planting...");
				for _ in 0..self.size_x as usize * self.size_z as usize / 3000
				{
					let b = if rng.gen() { 37 } else { 38 };
					let patch_x = rng.gen_range(0..size_x);
					let patch_z = rng.gen_range(0..size_z);
					for _ in 0..10
					{
						let mut x = patch_x;
						let mut z = patch_z;
						for _ in 0..5
						{
							x += rng.gen_range(0..6) - rng.gen_range(0..6);
							z += rng.gen_range(0..6) - rng.gen_range(0..6);
							if x >= 0 && x < size_x && z >= 0 && z < size_z
							{
								let y = height_map[x as usize + z as usize * size_x as usize] + 1;
								if self.get_block(x, y, z) == 0 && self.get_block(x, y-1, z) == 2
								{
									self.set_block(x, y, z, b);
								}
							}
						}
					}
				}
				for _ in 0..self.size_x as usize * self.size_y as usize * self.size_z as usize / 2000
				{
					let b = if rng.gen() { 39 } else { 40 };
					let patch_x = rng.gen_range(0..size_x);
					let patch_y = rng.gen_range(0..size_y);
					let patch_z = rng.gen_range(0..size_z);
					for _ in 0..20
					{
						let mut x = patch_x;
						let mut y = patch_y;
						let mut z = patch_z;
						for _ in 0..5
						{
							x += rng.gen_range(0..6) - rng.gen_range(0..6);
							y += rng.gen_range(0..2) - rng.gen_range(0..2);
							z += rng.gen_range(0..6) - rng.gen_range(0..6);
							if x >= 0 && x < size_x && z >= 0 && z < size_z && y >= 0 && y < height_map[x as usize + z as usize * size_x as usize]-1
							{
								if self.get_block(x, y, z) == 0 && self.get_block(x, y-1, z) == 1
								{
									self.set_block(x, y, z, b);
								}
							}
						}
					}
				}
				for _ in 0..self.size_x as usize * self.size_z as usize / 4000
				{
					let patch_x = rng.gen_range(0..size_x);
					let patch_z = rng.gen_range(0..size_z);
					for _ in 0..20
					{
						let mut x = patch_x;
						let mut z = patch_z;
						for _ in 0..20
						{
							x += rng.gen_range(0..6) - rng.gen_range(0..6);
							z += rng.gen_range(0..6) - rng.gen_range(0..6);
							if x >= 0 && x < size_x && z >= 0 && z < size_z && rng.gen::<f64>() <= 0.25
							{
								let y = height_map[x as usize + z as usize * size_x as usize] + 1;
								let tree_height = rng.gen_range(4..7);
								if self.is_space_for_tree(x,y,z,tree_height)
								{
									self.grow_tree(x,y,z,tree_height, &mut rng);
								}
							}
						}
					}
				}
			}
		}
		self.reset_spawn();
		Ok(())
	}
	pub fn load_from(&mut self, path: String) -> Result<(), String>
	{
		if let Ok(mut file) = File::open(path)
		{
			println!("loading level");
			let mut buf = Vec::<u8>::new();
			if let Ok(_) = file.read_to_end(&mut buf)
			{
				let mut gz: GzDecoder<&[u8]> = GzDecoder::new(&buf[..]);
				self.size_x = gz.read_i16::<NetworkEndian>().unwrap_or_default();
				self.size_y = gz.read_i16::<NetworkEndian>().unwrap_or_default();
				self.size_z = gz.read_i16::<NetworkEndian>().unwrap_or_default();
				if self.size_x <= 0 || self.size_y <= 0 || self.size_z <= 0
				{
					return Err(String::from("invalid size"));
				}
				self.data = Vec::new();
				let size = gz.read_to_end(&mut self.data).unwrap_or_default();
				if size != self.size_x as usize * self.size_y as usize * self.size_z as usize
				{
					return Err(String::from("size and length does not match"));
				}
				self.reset_spawn();
				return Ok(())
			}
		}
		Err("could not open level file for loading".to_string())
	}
	pub fn load(&mut self) -> Result<(), String>
	{
		self.load_from(format!("{}.dat", self.name))
	}
	pub fn save_to(&self, path: String) -> Result<(), String>
	{
		if let Ok(mut f) = File::create(&path)
		{
			println!("saving level {}", path);
			if f.write_all(&self.get_gzip(SaveType::Disk)?).is_ok()
			{
				return Ok(());
			}
		}
		Err(format!("could not save level {}", path))
	}
	pub fn save(&mut self) -> Result<(), String>
	{
		if self.changed
		{
			self.changed = false;
			if self.copy_backup().is_err()
			{
				println!("could not backup previous world.");
			}
			self.save_to(format!("{}.dat", self.name))
		}
		else
		{
			Ok(())
		}
	}
	pub fn copy_backup(&self) -> Result<u64, std::io::Error>
	{
		fs::create_dir_all("backup/")?;
		println!("copying backup");
		fs::copy(format!("{}.dat", self.name), format!("backup/{}-{}.dat", self.name, Local::now().format("%Y%m%d_%H%M%S").to_string()))
	}
	pub fn get_block(&self, x: i16, y: i16, z: i16) -> u8
	{
		assert!(x >= 0 && y >= 0 && z >= 0 && x < self.size_x && y < self.size_y && z < self.size_z);
		self.data[x as usize + z as usize * self.size_x as usize + y as usize * self.size_x as usize * self.size_z as usize]
	}
	pub fn set_block(&mut self, x: i16, y: i16, z: i16, b: u8)
	{
		assert!(x >= 0 && y >= 0 && z >= 0 && x < self.size_x && y < self.size_y && z < self.size_z);
		if !self.changed
		{
			self.changed = true;
		}
		self.data[x as usize + z as usize * self.size_x as usize + y as usize * self.size_x as usize * self.size_z as usize] = b;
	}
	pub fn place_block(&mut self, x: i16, mut y: i16, z: i16, mut b: u8) -> Vec<(i16,i16,i16,u8)>
	{
		let data = &block::BLOCKS[b as usize];
		if data.fluid && y < self.size_y-1
		{
			let b_above = self.get_block(x, y+1, z);
			if block::BLOCKS[b_above as usize].fall
			{
				// we broke something and something above is falling!
				let sand_tower_bottom = y+1;
				let mut sand_tower_top = sand_tower_bottom;
				let mut line = vec![b_above];
				loop
				{
					if sand_tower_top >= self.size_y - 1
					{
						break;
					}
					let b = self.get_block(x, sand_tower_top+1, z);
					if !block::BLOCKS[b as usize].fall
					{
						break;
					}
					sand_tower_top += 1;
					line.push(b);
				}
				// let's find a bottom
				let mut fallen_tower_bottom = y;
				loop
				{
					if fallen_tower_bottom <= 0
					{
						break;
					}
					let b = self.get_block(x, fallen_tower_bottom-1, z);
					if !block::BLOCKS[b as usize].fluid
					{
						break;
					}
					fallen_tower_bottom -= 1;
				}
				let mut blocks_changed = vec![];
				let fallen_tower_top = fallen_tower_bottom + (sand_tower_top-sand_tower_bottom);
				if sand_tower_bottom - fallen_tower_top > 1
				{
					self.set_block(x, y, z, 0);
					blocks_changed.push((x,y,z,0));
				}
				for (i, b) in line.iter().enumerate()
				{
					let y = fallen_tower_bottom + i as i16;
					if *b != self.get_block(x, y, z)
					{
						self.set_block(x, y, z, *b);
						blocks_changed.push((x,y,z,*b));
					}
				}
				for y in sand_tower_bottom.max(fallen_tower_top+1)..=sand_tower_top
				{
					self.set_block(x, y, z, 0);
					blocks_changed.push((x,y,z,0));
				}
				self.reset_spawn();
				return blocks_changed;
			}
		}
		else
		{
			if data.fall
			{
				while y > 0 && block::BLOCKS[self.get_block(x, y-1, z) as usize].fluid
				{
					y -= 1;
				}
			}
			if data.slab.is_some() && y > 0 && self.get_block(x, y-1, z) == b
			{
				b = data.slab.unwrap();
				y -= 1;
			}
		}
		
		self.set_block(x, y, z, b);
		self.reset_spawn();
		vec![(x,y,z,b)]
	}
	pub fn fill_oblate_spheroid(&mut self, x: f64, y: f64, z: f64, b: u8, r: f64)
	{
		let x_beg = (x - r).max(0.0).floor() as i16;
		let x_end = (x + r).min(self.size_x as f64).floor() as i16;
		let y_beg = (y - r).max(0.0).floor() as i16;
		let y_end = (y + r).min(self.size_y as f64).floor() as i16;
		let z_beg = (z - r).max(0.0).floor() as i16;
		let z_end = (z + r).min(self.size_z as f64).floor() as i16;

		let radius_sq = r*r;
		for yy in y_beg..y_end
		{
			let dy = yy - y as i16;
			for zz in z_beg..z_end
			{
				let dz = zz - z as i16;
				for xx in x_beg..x_end
				{
					let dx = xx - x as i16;
					if ((dx * dx + 2 * dy * dy + dz * dz) as f64) < radius_sq
					{
						if self.get_block(xx, yy, zz) == 1
						{
							self.set_block(xx, yy, zz, b);
						}
					}
				}
			}			
		}
	}
	pub fn flood_fill(&mut self, x: i16, y: i16, z: i16, b: u8)
	{
		let mut stack = vec![];
		stack.push((x,y,z));
		while let Some((x,y,z)) = stack.pop()
		{
			if self.get_block(x, y, z) != 0
			{
				continue;
			}
			self.set_block(x, y, z, b);

			if x > 0
			{
				stack.push((x-1,y,z));
			}
			if x < self.size_x-1
			{
				stack.push((x+1,y,z));
			}
			if z > 0
			{
				stack.push((x,y,z-1));
			}
			if z < self.size_z-1
			{
				stack.push((x,y,z+1));
			}
			if y > 0
			{
				stack.push((x,y-1,z));
			}
		}
	}
	pub fn is_space_for_tree(&self, x: i16, y: i16, z: i16, height: i16) -> bool
	{
		let base_height = height - 4;
		for y in y..y+base_height as i16
		{
			for z in z-1..=z+1
			{
				for x in x-1..=x+1
				{
					if x < 0 || x >= self.size_x || y < 0 || y >= self.size_y || z < 0 || z >= self.size_z
					{
						return false;
					}
					if self.get_block(x, y, z) != 0
					{
						return false;
					}
				}
			}
		}
		for y in y+base_height as i16..y+height as i16
		{
			for z in z-2..=z+2
			{
				for x in x-2..=x+2
				{
					if x < 0 || x >= self.size_x || y < 0 || y >= self.size_y || z < 0 || z >= self.size_z
					{
						return false;
					}
					if self.get_block(x, y, z) != 0
					{
						return false;
					}
				}
			}
		}
		true
	}
	pub fn grow_tree(&mut self, x: i16, y: i16, z: i16, height: i16, rng: &mut StdRng)
	{
		let top_start = y+(height-2);
		for y in y+height-4..top_start
		{
			for zz in -2..=2
			{
				for xx in -2..=2
				{
					let x = x + xx;
					let z = z + zz;

					if xx.abs() == 2 && zz.abs() == 2
					{
						if rng.gen()
						{
							self.set_block(x, y, z, 18);
						}
					}
					else
					{
						self.set_block(x, y, z, 18);
					}
				}
			}
		}
		for y in top_start..y+height
		{
			for zz in -1..=1
			{
				for xx in -1..=1
				{
					let x = x + xx;
					let z = z + zz;

					if xx.abs() == 1 && zz.abs() == 1
					{
						if rng.gen()
						{
							self.set_block(x, y, z, 18);
						}
					}
					else
					{
						self.set_block(x, y, z, 18);
					}
				}
			}
		}
		for y in y..y+height-1
		{
			self.set_block(x, y, z, 17)
		}
		self.set_block(x, y, z, 17);
	}
	pub fn get_gzip(&self, savetype: SaveType) -> Result<Vec<u8>, String>
	{
		let mut e = GzEncoder::new(Vec::new(), match savetype
		{
			SaveType::Network => Compression::fast(),
			SaveType::Disk => Compression::default()
		});
		let ok = match savetype
		{
			SaveType::Network =>
			{
				e.write_u32::<NetworkEndian>(self.size_x as u32 * self.size_y as u32 * self.size_z as u32).is_ok()
			}
			SaveType::Disk =>
			{
				e.write_i16::<NetworkEndian>(self.size_x).is_ok() &&
				e.write_i16::<NetworkEndian>(self.size_y).is_ok() &&
				e.write_i16::<NetworkEndian>(self.size_z).is_ok()
			}
		} &&
		e.write_all(&self.data).is_ok();
		if ok
		{
			let bytes = e.finish();
			if let Ok(bytes) = bytes
			{
				return Ok(bytes);
			}
		}
		Err(String::from("could not gzip world"))
	}
	pub fn max_y(&mut self, x: i16, z: i16) -> i16
	{
		for y in (0..self.size_y).rev()
		{
			if self.get_block(x, y, z) > 0
			{
				return y*32+61;
			}
		}
		29
	}
	pub fn reset_spawn(&mut self)
	{
		self.spawn_x = self.size_x*16 + 16;
		self.spawn_z = self.size_z*16 + 16;
		self.spawn_y = self.max_y(self.size_x/2, self.size_z/2);
	}
}