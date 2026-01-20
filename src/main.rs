mod memory;
mod cpu;
mod bus;

use std::fs;

use crate::{bus::bus::{Bus, Machine}, cpu::w65c02s::{CpuError, Mnemomic, W65C02S}};

#[derive(Debug)]
enum Error{
    IO(std::io::Error),
    Cpu(CpuError),
}
fn main() -> Result<(), Error>{
    let rom_image = fs::read("data/rom.bin").map_err(|e| Error::IO(e))?;

    let mut cpu = W65C02S::default();
    let mut bus = Machine::new_32k_ram_32k_rom(&rom_image);

    //bus.print_ram_map();

    //return Ok(());

    cpu.reset(&mut bus);

    loop{
        let op = cpu.step(&mut bus).map_err(|e| Error::Cpu(e))?;
        match op{
            Mnemomic::BRK => {break;},
            _ => {println!("{:?}", op);}
        }
    }

    Ok(())
}
