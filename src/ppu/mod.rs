#![allow(dead_code)]
use std::fmt;

mod register;
mod utils;

use crate::config::GAMEBOY_RESOLUTION;
use crate::ppu::register::{lcdc::LCDC, stat::STAT};
use crate::ppu::utils::pallete::Pallete;

const ACESSES_OAM_CYCLES: u32 = 20; // Mode 2 = 80 dots; 80 / 4 M-Cycle = 20
const ACESSES_VRAM_CYCLES: u32 = 43; // Mode 3 = 172 dots; 172 / 4 M-Cycle = 43
const HBLANK_CYCLES: u32 = 51; // Mode 0 = 204 dots; 204 / 4 M-Cycle = 51
const VBLANK_CYCLES: u32 = 114; // Mode 1 = 4560 dots (10 scanlines) ; 4560 / 4 M-Cycle = 1140 = 114 cycles per scanline

#[derive(Debug)]
pub enum Mode {
	HBlank,
	VBlank,
	AccessOAM,
	AccessVRAM,
}

#[derive(Debug)]
pub struct PPU {
	pub vram: [u8; 0x2000],
	pub oam: [u8; 0x00A0],
	pub lcdc: LCDC,            // FF40
	pub stat: STAT,            // FF41
	pub scy: u8,               // FF42
	pub scx: u8,               // FF43
	pub ly: u8,                // FF44
	pub lyc: u8,               // FF45
	pub bg_pallete: Pallete,   // FF47
	pub obj0_pallete: Pallete, // FF48
	pub obj1_pallete: Pallete, // FF49
	pub wy: u8,                // FF4A
	pub wx: u8,                // FF4B
	pub mode: Mode,
	pub cycles: u32,
	pub video_buffer: [u8; (160 * 144) as usize],
	pub current_line: u8,
}

impl PPU {
	pub fn new() -> Self {
		Self {
			vram: [0; 0x2000],
			oam: [0; 0x00A0],
			lcdc: LCDC::new(0x00),
			stat: STAT::new(0x00),
			scy: 0x00,
			scx: 0x00,
			ly: 0x00,
			lyc: 0x00,
			bg_pallete: Pallete::new(0x00),
			obj0_pallete: Pallete::new(0x00),
			obj1_pallete: Pallete::new(0x00),
			wx: 0x00,
			wy: 0x00,
			mode: Mode::AccessOAM,
			cycles: ACESSES_OAM_CYCLES,
			video_buffer: [0; (160 * 144) as usize],
			current_line: 0,
		}
	}

	pub fn read(&self, addr: u16) -> u8 {
		// read vram
		if addr >= 0x8000 && addr <= 0x9FFF {
			let addr = addr - 0x8000;
			return self.vram[addr as usize];
		}

		// read oam
		if addr >= 0xFE00 && addr <= 0xFE9F {
			let addr = addr - 0xFE00;
			return self.oam[addr as usize];
		}

		match addr {
			0xFF40 => return self.lcdc.get_lcdc(),
			0xFF41 => return self.stat.get_stat(),
			0xFF42 => return self.scy,
			0xFF43 => return self.scx,
			0xFF44 => return self.ly,
			0xFF45 => return self.lyc,
			0xFF47 => return self.bg_pallete.get_pallete(),
			0xFF48 => return self.obj0_pallete.get_pallete(),
			0xFF49 => return self.obj1_pallete.get_pallete(),
			0xFF4A => return self.wy,
			0xFF4B => return self.wx,
			_ => (),
		};

		return 0x00;
	}

	pub fn write(&mut self, addr: u16, data: u8) {
		// write on vram
		if addr >= 0x8000 && addr <= 0x9FFF {
			let addr = addr - 0x8000;
			self.vram[addr as usize] = data;
			return;
		}

		// write on oam
		if addr >= 0xFE00 && addr <= 0xFE9F {
			let addr = addr - 0xFE00;
			self.oam[addr as usize] = data;
			return;
		}

		match addr {
			0xFF40 => self.lcdc.set_lcdc(data),
			0xFF41 => self.stat.set_stat(data),
			0xFF42 => self.scy = data,
			0xFF43 => self.scx = data,
			0xFF44 => self.ly = data,
			0xFF45 => self.lyc = data,
			0xFF47 => self.bg_pallete.set_pallete(data),
			0xFF48 => self.obj0_pallete.set_pallete(data),
			0xFF49 => self.obj1_pallete.set_pallete(data),
			0xFF4A => self.wy = data,
			0xFF4B => self.wx = data,
			_ => (),
		};

		return;
	}

	pub fn draw_line(&mut self) {
		let bg_pallete = self.bg_pallete.extract_pallete();

		if self.lcdc.is_set(LCDC::BG_ON) {
			let bg_map = if self.lcdc.is_set(LCDC::BG_MAP) { 0x1C00 } else { 0x1800 };

			for i in 0..GAMEBOY_RESOLUTION.width {
				let background_x = (i as u8).wrapping_add(self.scx);
				let background_y = self.ly.wrapping_add(self.scy);

				let tile_map_x = (background_x / 8) as u16;
				let tile_map_y = (background_y / 8) as u16;

				let tile_address = bg_map + (tile_map_y * 32) + tile_map_x;
				let tile_id = self.vram[tile_address as usize] as u16;

				let tile_data = if self.lcdc.is_set(LCDC::BG_ADDR) {
					0x0000 + (tile_id * 16)
				} else {
					0x1000 + (((tile_id as i8) * 16) as u16)
				};

				let row_in_tile = (background_y % 8) * 2;
				let byte_1 = self.vram[(tile_data + row_in_tile as u16) as usize];
				let byte_2 = self.vram[(tile_data + row_in_tile as u16 + 1) as usize];

				let column_in_tile = background_x % 8;
				let bit_pos = 7 - column_in_tile;
				let hi = (byte_2 >> bit_pos) & 0x1;
				let lo = (byte_1 >> bit_pos) & 0x1;
				let color_index = ((hi << 1) | lo) & 0x11;

				let color = bg_pallete[color_index as usize];

				let pixel_index = (self.ly as usize * GAMEBOY_RESOLUTION.width as usize) + i as usize;

				self.video_buffer[pixel_index] = color
			}
		}
	}

	pub fn step(&mut self) {
		if !self.lcdc.is_set(LCDC::LCD_ON) {
			return;
		}

		self.cycles = self.cycles.wrapping_sub(1);

		if self.cycles > 0 {
			return;
		}

		match self.mode {
			Mode::AccessOAM => {
				self.mode = Mode::AccessVRAM;
				self.cycles = ACESSES_VRAM_CYCLES;
			}

			Mode::AccessVRAM => {
				self.draw_line();
				self.mode = Mode::HBlank;
				self.cycles = HBLANK_CYCLES;
			}

			Mode::HBlank => {
				self.ly += 1;

				if self.ly == 144 {
					self.mode = Mode::VBlank;
					self.cycles = VBLANK_CYCLES;
				} else {
					self.mode = Mode::AccessOAM;
					self.cycles = ACESSES_OAM_CYCLES;
				}
			}

			Mode::VBlank => {
				self.ly += 1;

				if self.ly > 153 {
					self.ly = 0;
					self.mode = Mode::AccessOAM;
					self.cycles = ACESSES_OAM_CYCLES;
				} else {
					self.cycles = VBLANK_CYCLES;
				}
			}
		}
	}

	pub fn show_video_buffer(&self) {
		for y in 0..GAMEBOY_RESOLUTION.height {
			for x in 0..GAMEBOY_RESOLUTION.width {
				let pixel_index = (y as usize * GAMEBOY_RESOLUTION.width as usize) + x as usize;
				print!("{}", self.video_buffer[pixel_index]);
			}
			println!();
		}
	}

	pub fn _dump_vram(&self, start: u16, end: u16) {
		if start < 0x8000 || end > 0x9FFF || start > end {
			println!("Invalid Interval! The VRAM goes from 0x8000 to 0x9FFF.");
			return;
		}

		let start_idx = (start - 0x8000) as usize;
		let end_idx = (end - 0x8000) as usize;

		println!("VRAM Dump (0x{:04X} - 0x{:04X}):", start, end);
		for (i, byte) in self.vram[start_idx..=end_idx].iter().enumerate() {
			if i % 16 == 0 {
				print!("\n0x{:04X}: ", start + i as u16);
			}
			print!("{:02X} ", byte);
		}
		println!();
	}

	pub fn _dump_oam(&self, start: u16, end: u16) {
		if start < 0xFE00 || end > 0xFE9F || start > end {
			println!("Invalid Interval! The OAM goes from 0xFE00 to 0xFE9F.");
			return;
		}

		let start_idx = (start - 0xFE00) as usize;
		let end_idx = (end - 0xFE00) as usize;

		println!("OAM Dump (0x{:04X} - 0x{:04X}):", start, end);
		for (i, byte) in self.oam[start_idx..=end_idx].iter().enumerate() {
			if i % 16 == 0 {
				print!("\n0x{:04X}: ", start + i as u16);
			}
			print!("{:02X} ", byte);
		}
		println!();
	}
}

impl fmt::Display for PPU {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		writeln!(f, "PPU State:")?;
		writeln!(f, "  LCDC:    	{:08b}", self.lcdc.get_lcdc())?;
		writeln!(f, "  STAT:    	{:08b}", self.stat.get_stat())?;
		writeln!(f, "  SCY:     	{:#04X}", self.scy)?;
		writeln!(f, "  SCX:     	{:#04X}", self.scx)?;
		writeln!(f, "  LY:      	{:#04X}", self.ly)?;
		writeln!(f, "  LYC:     	{:#04X}", self.lyc)?;
		writeln!(f, "  WY:      	{:#04X}", self.wy)?;
		writeln!(f, "  WX:      	{:#04X}", self.wx)?;
		writeln!(f, "  BG Pal:  	{:#04X}", self.bg_pallete.get_pallete())?;
		writeln!(f, "  OBJ0 Pal:	{:#04X}", self.obj0_pallete.get_pallete())?;
		writeln!(f, "  OBJ1 Pal:	{:#04X}", self.obj1_pallete.get_pallete())?;

		writeln!(f, "\n  VRAM: [first 64 bytes]")?;
		for (i, byte) in self.vram[..64].iter().enumerate() {
			if i % 16 == 0 {
				writeln!(f)?;
			}
			write!(f, "{:02X} ", byte)?;
		}
		writeln!(f)?;

		writeln!(f, "\n  OAM: [first 64 bytes]")?;
		for (i, byte) in self.oam[..64].iter().enumerate() {
			if i % 16 == 0 {
				writeln!(f)?;
			}
			write!(f, "{:02X} ", byte)?;
		}
		writeln!(f)
	}
}
