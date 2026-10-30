use core::panic;
use std::ops::{Index, IndexMut};

use crate::core::{cartridge::{self, Cartridge}, system::Byte};

pub const ROM0_RANGE: (usize, usize) = (0x0000, 0x3FFF);
pub const ROMX_RANGE: (usize, usize) = (0x4000, 0x7FFF);
const VRAM_RANGE: (usize, usize) = (0x8000, 0x9FFF);
pub const SRAM_RANGE: (usize, usize) = (0xA000, 0xBFFF);
const WRAM_RANGE: (usize, usize) = (0xC000, 0xDFFF);
const ERAM_RANGE: (usize, usize) = (0xE000, 0xFDFF);
const OAM_RANGE: (usize, usize) = (0xFE00, 0xFE9F);
const UNUSABLE_RANGE: (usize, usize) = (0xFEA0, 0xFEFF);
const IO_REGISTERS_RANGE: (usize, usize) = (0xFF00, 0xFF7F);
const HRAM_RANGE: (usize, usize) = (0xFF80, 0xFFFE);
const IE_REGISTER_RANGE: (usize, usize) = (0xFFFF, 0xFFFF);

pub struct Memory{
    vram: [Byte;(VRAM_RANGE.1 + 1) - VRAM_RANGE.0],
    wram: [Byte;(WRAM_RANGE.1 + 1) - WRAM_RANGE.0],
    eram: [Byte;(ERAM_RANGE.1 + 1) - ERAM_RANGE.0],
    oam: [Byte;(OAM_RANGE.1 + 1) - OAM_RANGE.0],
    unusable: [Byte;(UNUSABLE_RANGE.1 + 1) - UNUSABLE_RANGE.0],
    io_registers: [Byte;(IO_REGISTERS_RANGE.1 + 1) - IO_REGISTERS_RANGE.0],
    hram: [Byte;(HRAM_RANGE.1 + 1) - HRAM_RANGE.0],
    ie_register: [Byte;(IE_REGISTER_RANGE.1 + 1) - IE_REGISTER_RANGE.0],
    cartridge: &'static mut dyn Cartridge
}

impl Memory{
    pub fn new(cartridge: &'static mut dyn Cartridge) -> Self{
        Self { 
            vram: [0;(VRAM_RANGE.1 + 1) - VRAM_RANGE.0],
            wram: [0;(WRAM_RANGE.1 + 1) - WRAM_RANGE.0],
            eram: [0;(ERAM_RANGE.1 + 1) - ERAM_RANGE.0],
            oam: [0;(OAM_RANGE.1 + 1) - OAM_RANGE.0],
            unusable: [0;(UNUSABLE_RANGE.1 + 1) - UNUSABLE_RANGE.0],
            io_registers: [0;(IO_REGISTERS_RANGE.1 + 1) - IO_REGISTERS_RANGE.0],
            hram: [0;(HRAM_RANGE.1 + 1) - HRAM_RANGE.0],
            ie_register: [0;(IE_REGISTER_RANGE.1 + 1) - IE_REGISTER_RANGE.0],
            cartridge
        }   
    }
}

impl Index<usize> for Memory{
    type Output = Byte;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= ROM0_RANGE.0 && index <= ROM0_RANGE.1 {
            self.cartridge.index(index)
        }
        else if index >= ROMX_RANGE.0 && index <= ROMX_RANGE.1 {
            self.cartridge.index(index)
        }
        else if index >= VRAM_RANGE.0 && index <= VRAM_RANGE.1 {
            &self.vram[index - VRAM_RANGE.0]
        }
        else if index >= SRAM_RANGE.0 && index <= SRAM_RANGE.1 {
            self.cartridge.index(index)
        }
        else if index >= WRAM_RANGE.0 && index <= WRAM_RANGE.1 {
            &self.wram[index - WRAM_RANGE.0]
        }
        else if index >= ERAM_RANGE.0 && index <= ERAM_RANGE.1 {
            &self.eram[index - ERAM_RANGE.0]
        }
        else if index >= OAM_RANGE.0 && index <= OAM_RANGE.1 {
            &self.oam[index - OAM_RANGE.0]
        }
        else if index >= UNUSABLE_RANGE.0 && index <= UNUSABLE_RANGE.1 {
            &self.unusable[index - UNUSABLE_RANGE.0]
        }
        else if index >= HRAM_RANGE.0 && index <= HRAM_RANGE.1 {
            &self.hram[index - HRAM_RANGE.0]
        }
        else if index >= IE_REGISTER_RANGE.0 && index <= IE_REGISTER_RANGE.1 {
            &self.ie_register[index - IE_REGISTER_RANGE.0]
        }
        else {
            panic!("[ERROR] Memory index out of range")
        }
    }

}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= ROM0_RANGE.0 && index <= ROM0_RANGE.1 {
            self.cartridge.index_mut(index)
        }
        else if index >= ROMX_RANGE.0 && index <= ROMX_RANGE.1 {
            self.cartridge.index_mut(index)
        }
        else if index >= VRAM_RANGE.0 && index <= VRAM_RANGE.1 {
            &mut self.vram[index - VRAM_RANGE.0]
        }
        else if index >= SRAM_RANGE.0 && index <= SRAM_RANGE.1 {
            self.cartridge.index_mut(index)
        }
        else if index >= WRAM_RANGE.0 && index <= WRAM_RANGE.1 {
            &mut self.wram[index - WRAM_RANGE.0]
        }
        else if index >= ERAM_RANGE.0 && index <= ERAM_RANGE.1 {
            &mut self.eram[index - ERAM_RANGE.0]
        }
        else if index >= OAM_RANGE.0 && index <= OAM_RANGE.1 {
            &mut self.oam[index - OAM_RANGE.0]
        }
        else if index >= UNUSABLE_RANGE.0 && index <= UNUSABLE_RANGE.1 {
            &mut self.unusable[index - UNUSABLE_RANGE.0]
        }
        else if index >= HRAM_RANGE.0 && index <= HRAM_RANGE.1 {
            &mut self.hram[index - HRAM_RANGE.0]
        }
        else if index >= IE_REGISTER_RANGE.0 && index <= IE_REGISTER_RANGE.1 {
            &mut self.ie_register[index - IE_REGISTER_RANGE.0]
        }
        else {
            panic!("[ERROR] Memory index out of range")
        }
    }
}
