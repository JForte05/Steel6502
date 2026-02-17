mod memory;
mod cpu;
mod bus;

use std::fs;
use std::env;
use std::path::{Path, PathBuf};

use crate::bus::bus::{Machine};
use crate::cpu::w65c02s::{CpuError, Mnemomic, W65C02S};

macro_rules! match_sequence {
    ($coll:expr, [$($pattern:pat),+ $(,)?] => $($output:expr),+) => {{
        let __pattern_len: usize = <[()]>::len(&[ $( { let _ = stringify!($pattern); ()} ),+ ]);
        let mut __pos: usize = 0;
        
        loop {
            if $coll.len() < __pattern_len || __pos >= ($coll.len() - __pattern_len){
                break None;
            }
            if let Some(__slice) = $coll.get(__pos..__pos + __pattern_len){
                if let [$($pattern),+] = __slice{
                    break Some((__pos, $($output),+));
                }
                else{
                    __pos += 1;
                }
            }
            else{
                break None;
            }
        }
    }};
    ($coll:expr, [$($pattern:pat),+ $(,)?]) => {{
        let __pattern_len: usize = <[()]>::len(&[ $( { let _ = stringify!($pattern); ()} ),+ ]);
        let mut __pos: usize = 0;
        
        loop {
            if $coll.len() < __pattern_len || __pos >= ($coll.len() - __pattern_len){
                break None;
            }
            if let Some(__slice) = $coll.get(__pos..__pos + __pattern_len){
                if let [$($pattern),+] = __slice{
                    break Some(__pos);
                }
                else{
                    __pos += 1;
                }
            }
            else{
                break None;
            }
        }
    }};
}

#[derive(Debug)]
enum ProgramError{
    OutputPathIsNotDirectory(String),
    CouldNotLocateFile(String),
    CouldNotReadFile(String),
    CouldNotWriteFile(String),
    CpuError(CpuError),
    NoRomFile,
    MalformedRomFile,
}

fn parse_output_flag(args: &[&str]) -> Result<PathBuf, String>{
    if let Some((_, desired)) = match_sequence!(args, ["-o", o] => o){
        let pot = env::current_dir().unwrap().join(Path::new(desired));

        if !pot.is_dir(){ Err(pot.to_str().unwrap().to_owned()) } else { Ok(pot) }
    } else { Ok(env::current_dir().unwrap()) }
}

fn parse_flags(args: &[String]) -> Result<PathBuf, ProgramError>{
    let sendable: Box<[&str]> = args.iter().map(String::as_str).collect();

    Ok(parse_output_flag(&sendable).map_err(|f| ProgramError::OutputPathIsNotDirectory(f))?)
}

fn main() -> Result<(), ProgramError>{
    let args = env::args().skip(1).collect::<Vec<String>>();
    let output_dir = parse_flags(&args)?;

    let mut skipped = false;
    for arg in args{
        if arg.starts_with('-') || skipped{
            skipped = !skipped;
            continue;
        }

        let rom_path = PathBuf::from(&arg);
        if !rom_path.exists(){
            return Err(ProgramError::CouldNotLocateFile(arg.to_string()));
        }
        let file_name = rom_path.file_stem().expect("Could not extract file name").to_str().expect("Failed to convert").to_owned();
        let rom = fs::read(rom_path).map_err(|_| ProgramError::CouldNotReadFile(arg.to_string()))?;
        
        let mut cpu = W65C02S::default();
        let mut machine_bus = Machine::new_32k_ram_32k_rom(&rom[0x8000..]);
        let rom_size = 32768usize;


        if rom.len() < rom_size{
            return Err(ProgramError::MalformedRomFile);
        }

        println!("Emulating {}", file_name);
        cpu.reset(&mut machine_bus);

        loop{
            let op = cpu.step(&mut machine_bus).map_err(|e| ProgramError::CpuError(e))?;
            match op{
                Mnemomic::BRK => {break;},
                _ => {}
            }
        }

        let output_file = output_dir.join(format!("{}_ram.bin", file_name));
        fs::write(
            &output_file,
            machine_bus.ram_contents()
        ).map_err(|_| ProgramError::CouldNotWriteFile(output_file.to_str().unwrap().to_owned()))?;
    }

    //fs::write("./data/ram.bin", bus.ram_contents()).map_err(|e| Error::IO(e))?;

    Ok(())
}
