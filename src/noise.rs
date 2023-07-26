// this was mostly from the classicube source code.

use rand::Rng;
use rand::seq::SliceRandom;

pub trait Noise
{
	fn get(&self, x: f64, y: f64) -> f64;
}
pub struct PerlinNoise
{
	noise_table: [u8; 256]
}
impl PerlinNoise
{
	pub fn new<T>(rng: &mut T) -> Self where T: Rng
	{
		let mut noise_table = [0; 256];

		for i in 0..256
		{
			noise_table[i] = i as u8;
		}
		noise_table.shuffle(rng);

		PerlinNoise { noise_table }
	}
}
impl Noise for PerlinNoise
{
	fn get(&self, mut x: f64, mut y: f64) -> f64
	{
		let x_floor = x.floor();
		let y_floor = y.floor();
		let xx = x_floor as u8;
		let yy = y_floor as u8;
		x -= x_floor;
		y -= y_floor;
		let u = x*x*x*(x*(x*6.0-15.0)+10.0);
		let v = y*y*y*(y*(y*6.0-15.0)+10.0);
		let a = self.noise_table[xx as usize].wrapping_add(yy);
		let b = self.noise_table[xx.wrapping_add(1) as usize].wrapping_add(yy);

		const X_FLAGS: i32 = 0x46552222;
		const Y_FLAGS: i32 = 0x2222550A;

		let hash = (self.noise_table[self.noise_table[a as usize] as usize] & 0xF) << 1;
		let g22 = (((X_FLAGS >> hash) & 3) - 1) as f64 * x + (((Y_FLAGS >> hash) & 3) - 1) as f64 * y;
		let hash = (self.noise_table[self.noise_table[b as usize] as usize] & 0xF) << 1;
		let g12 = (((X_FLAGS >> hash) & 3) - 1) as f64 * (x - 1.0) + (((Y_FLAGS >> hash) & 3) - 1) as f64 * y;
		let c1 = g22 + u *(g12 - g22);

		let hash = (self.noise_table[self.noise_table[(a.wrapping_add(1)) as usize] as usize] & 0xF) << 1;
		let g21 = (((X_FLAGS >> hash) & 3) - 1) as f64 * x + (((Y_FLAGS >> hash) & 3) - 1) as f64 * (y - 1.0);
		let hash = (self.noise_table[self.noise_table[(b.wrapping_add(1)) as usize] as usize] & 0xF) << 1;
		let g11 = (((X_FLAGS >> hash) & 3) - 1) as f64 * (x - 1.0) + (((Y_FLAGS >> hash) & 3) - 1) as f64 * (y - 1.0);
		let c2 = g21 + u *(g11 - g21);

		c1 + v * (c2 - c1)
	}
}
pub struct OctaveNoise<N> where N: Noise
{
	noise: N,
	octave: u8
}
impl<N: Noise> OctaveNoise<N>
{
	pub fn new(noise: N, octave: u8) -> Self
	{
		OctaveNoise { noise, octave }
	}
}
impl<N: Noise> Noise for OctaveNoise<N>
{
	fn get(&self, x: f64, y: f64) -> f64
	{
		let mut amp = 1.0;
		let mut freq = 1.0;
		let mut sum = 0.0;

		for _ in 0..self.octave
		{
			sum += self.noise.get(x*freq, y*freq)*amp;
			amp *= 2.0;
			freq /= 2.0;
		}

		sum
	}
}
pub struct CombinedNoise<N> where N: Noise
{
	noise1: N,
	noise2: N
}
impl<N: Noise> CombinedNoise<N>
{
	pub fn new(noise1: N, noise2: N) -> Self
	{
		CombinedNoise { noise1, noise2 }
	}
}
impl<N: Noise> Noise for CombinedNoise<N>
{
	fn get(&self, x: f64, y: f64) -> f64
	{
		self.noise1.get(x + self.noise2.get(x,y), y)
	}
}