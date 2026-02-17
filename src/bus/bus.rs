use crate::memory::memory::{Indexed, RAMSegment, ROMSegment};

pub trait Bus{
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, val: u8);
}

#[derive(Copy, Clone, Debug)]
enum Page{
    Unmapped,
    RAM {page_relative: usize},
    ROM {page_relative: usize},
    //IODevice,
}

fn split_address(address: u16) -> (usize, u8){
    ((address >> 8) as usize, (address & 0xff) as u8)
}

pub struct Machine{
    rom: ROMSegment,
    ram: RAMSegment,

    page_map: [Page; 256],
}
impl Machine{
    /// ram pages: 0x00 -> 0x7f, total address space: 0x0000 -> 0x7fff (32kb)
    /// rom pages: 0x80 -> 0xff, total address space: 0x8000 -> 0xffff (32kb)
    pub fn new_32k_ram_32k_rom(rom_image: &[u8]) -> Self{
        let ram = RAMSegment::new(128);
        let mut rom = ROMSegment::new(128);
        match rom.load(rom_image){
            Ok(_) => {},
            Err(_) => panic!("ROM image ({:X} bytes) exceeded size of ROM ({:X} bytes)", rom_image.len(), rom.len()),
        }

        let mut map = [Page::Unmapped; 256];

        // init ram in page_map
        for page in 0x00usize..=0x7fusize{
            map[page] = Page::RAM { page_relative: page };
        }

        // init ram in page_map
        for page in 0x80usize..=0xffusize{
            map[page] = Page::ROM { page_relative: page - 0x80 };
        }

        Self { ram: ram, rom: rom, page_map: map }
    }

    pub fn load_ram(&mut self, bytes: &[u8]){
        self.ram.load(bytes);
    }
    pub fn ram_contents(&self) -> Box<[u8]>{
        self.ram.contents()
    }
}
impl Bus for Machine{
    fn read(&mut self, address: u16) -> u8 {
        let (page, offset) = split_address(address);
        match self.page_map[page]{
            Page::ROM { page_relative } => self.rom.read_page_offset(page_relative, offset),
            Page::RAM { page_relative } => self.ram.read_page_offset(page_relative, offset),
            Page::Unmapped => panic!("Attempted to read from unmapped memory at address {:X}", address),
        }
    }

    fn write(&mut self, address: u16, val: u8){
        let (page, offset) = split_address(address);
        match self.page_map[page]{
            Page::RAM { page_relative } => self.ram.write_page_offset(page_relative, offset, val),
            Page::ROM { page_relative: _ } => panic!("Attempted to write to ROM at address {:X}", address),
            Page::Unmapped => panic!("Attempted to write to Unmapped memory at address {:X}", address),
        }
    }
}