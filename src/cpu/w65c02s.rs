use std::num::Wrapping;

use crate::bus::bus::{Bus};

enum CpuError{
    InvalidOpcode(u8),
    InvalidOperand(Operand),
}

enum Status{
    C,
    Z,
    I,
    D,
    B,

    V,
    N,
}
impl Status{
    fn mask(&self) -> u8{
        match *self{
            Status::C => 0b0000_0001,
            Status::Z => 0b0000_0010,
            Status::I => 0b0000_0100,
            Status::D => 0b0000_1000,
            Status::B => 0b0001_0000,

            Status::V => 0b0100_0000,
            Status::N => 0b1000_0000,
        }
    }
}

/**
   Successor to 6502.

    Datasheet: https://www.westerndesigncenter.com/wdc/documentation/w65c02s.pdf
 */
struct W65C02S{
    program_counter: Wrapping<u16>,
    a_register: u8,
    y_register: u8,
    x_register: u8,
    stack_pointer: Wrapping<u8>,
    processor_status_register: u8,
}
impl W65C02S{
    // high byte for all vectors immediately follow the low byte in address space
    pub const IRQB_LOW: u16 = 0xFFFE; // At this address should be the lower 8 bits of the address to jump to when processing an interrupt request
    pub const RESB_LOW: u16 = 0xFFFC; // At this address should be the lower 8 bits of the address to jump to after resetting (ie. the entry point)
    pub const NMIB_LOW: u16 = 0xFFFA; // At this address should be the lower 8 bits of the address to jump to when processing a nonmaskable interrupt

    pub const STACK_POINTER_HIGH: u8 = 0x01; // When combined with the stack_pointer

    // invalids = [3, 19, 35, 51, 67, 83, 99, 115, 131, 147, 163, 179, 195, 211, 227, 243, 2, 34, 66, 98, 130, 194, 226, 68, 84, 212, 244, 11, 27, 43, 59, 75, 91, 107, 123, 139, 155, 171, 187, 235, 251, 92, 220, 252]
    const OPERATIONS: [Option<Operation>; 256] = [
        Option::Some(Operation { addressing_mode: AddressingMode::Stack, mnemomic: Mnemomic::BRK }),                        // 0x00 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedIndirect, mnemomic: Mnemomic::ORA }),      // 0x01 
        Option::None,                                                                                                       // 0x02 [Invalid]
        Option::None,                                                                                                       // 0x03 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::TSB }),                     // 0x04 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::ORA }),                     // 0x05 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::ASL }),                     // 0x06 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::RMBN(0) }),                 // 0x07 
        Option::Some(Operation { addressing_mode: AddressingMode::Stack, mnemomic: Mnemomic::PHP }),                        // 0x08 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::ORA }),                    // 0x09 
        Option::Some(Operation { addressing_mode: AddressingMode::Accumulator, mnemomic: Mnemomic::ASL }),                  // 0x0A 
        Option::None,                                                                                                       // 0x0B [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::TSB }),                     // 0x0C 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::ORA }),                     // 0x0D 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::ASL }),                     // 0x0E 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BBRN(0) }),   // 0x0F 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BPL }),       // 0x10 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirectIndexedY, mnemomic: Mnemomic::ORA }),     // 0x11 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirect, mnemomic: Mnemomic::ORA }),             // 0x12 
        Option::None,                                                                                                       // 0x13 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::TRB }),                     // 0x14 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::ORA }),             // 0x15 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::ASL }),             // 0x16 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::RMBN(1) }),                 // 0x17 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::CLC }),                      // 0x18 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedY, mnemomic: Mnemomic::ORA }),             // 0x19 
        Option::Some(Operation { addressing_mode: AddressingMode::Accumulator, mnemomic: Mnemomic::INC }),                  // 0x1A 
        Option::None,                                                                                                       // 0x1B [Invalid]
            Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::TRB }),                 // 0x1C 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::ORA }),             // 0x1D 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::ASL }),             // 0x1E 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BBRN(1) }),   // 0x1F 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::JSR }),                     // 0x20 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedIndirect, mnemomic: Mnemomic::AND }),      // 0x21 
        Option::None,                                                                                                       // 0x22 [Invalid]
        Option::None,                                                                                                       // 0x23 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::BIT }),                     // 0x24 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::AND }),                     // 0x25 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::ROL }),                     // 0x26 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::RMBN(2) }),                 // 0x27 
        Option::Some(Operation { addressing_mode: AddressingMode::Stack, mnemomic: Mnemomic::PLP }),                        // 0x28 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::AND }),                    // 0x29 
        Option::Some(Operation { addressing_mode: AddressingMode::Accumulator, mnemomic: Mnemomic::ROL }),                  // 0x2A 
        Option::None,                                                                                                       // 0x2B [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::BIT }),                     // 0x2C 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::AND }),                     // 0x2D 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::ROL }),                     // 0x2E 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BBRN(2) }),   // 0x2F 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BMI }),       // 0x30 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirectIndexedY, mnemomic: Mnemomic::AND }),     // 0x31 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirect, mnemomic: Mnemomic::AND }),             // 0x32 
        Option::None,                                                                                                       // 0x33 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::BIT }),             // 0x34 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::AND }),             // 0x35 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::ROL }),             // 0x36 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::RMBN(3) }),                 // 0x37 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::SEC }),                      // 0x38 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedY, mnemomic: Mnemomic::AND }),             // 0x39 
        Option::Some(Operation { addressing_mode: AddressingMode::Accumulator, mnemomic: Mnemomic::DEC }),                  // 0x3A 
        Option::None,                                                                                                       // 0x3B [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::BIT }),             // 0x3C 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::AND }),             // 0x3D 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::ROL }),             // 0x3E 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BBRN(3) }),   // 0x3F 
        Option::Some(Operation { addressing_mode: AddressingMode::Stack, mnemomic: Mnemomic::RTI }),                        // 0x40 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedIndirect, mnemomic: Mnemomic::EOR }),      // 0x41 
        Option::None,                                                                                                       // 0x42 [Invalid]
        Option::None,                                                                                                       // 0x43 [Invalid]
        Option::None,                                                                                                       // 0x44 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::EOR }),                     // 0x45 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::LSR }),                     // 0x46 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::RMBN(4) }),                 // 0x47 
        Option::Some(Operation { addressing_mode: AddressingMode::Stack, mnemomic: Mnemomic::PHA }),                        // 0x48 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::EOR }),                    // 0x49 
        Option::Some(Operation { addressing_mode: AddressingMode::Accumulator, mnemomic: Mnemomic::LSR }),                  // 0x4A 
        Option::None,                                                                                                       // 0x4B [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::JMP }),                     // 0x4C 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::EOR }),                     // 0x4D 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::LSR }),                     // 0x4E 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BBRN(4) }),   // 0x4F 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BVC }),       // 0x50 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirectIndexedY, mnemomic: Mnemomic::EOR }),     // 0x51 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirect, mnemomic: Mnemomic::EOR }),             // 0x52 
        Option::None,                                                                                                       // 0x53 [Invalid]
        Option::None,                                                                                                       // 0x54 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::EOR }),             // 0x55 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::LSR }),             // 0x56 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::RMBN(5) }),                 // 0x57 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::CLI }),                      // 0x58 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedY, mnemomic: Mnemomic::EOR }),             // 0x59 
        Option::Some(Operation { addressing_mode: AddressingMode::Stack, mnemomic: Mnemomic::PHY }),                        // 0x5A 
        Option::None,                                                                                                       // 0x5B [Invalid]
        Option::None,                                                                                                       // 0x5C [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::EOR }),             // 0x5D 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::LSR }),             // 0x5E 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BBRN(5) }),   // 0x5F 
        Option::Some(Operation { addressing_mode: AddressingMode::Stack, mnemomic: Mnemomic::RTS }),                        // 0x60 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedIndirect, mnemomic: Mnemomic::ADC }),      // 0x61 
        Option::None,                                                                                                       // 0x62 [Invalid]
        Option::None,                                                                                                       // 0x63 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::STZ }),                     // 0x64 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::ADC }),                     // 0x65 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::ROR }),                     // 0x66 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::RMBN(6) }),                 // 0x67 
        Option::Some(Operation { addressing_mode: AddressingMode::Stack, mnemomic: Mnemomic::PLA }),                        // 0x68 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::ADC }),                    // 0x69 
        Option::Some(Operation { addressing_mode: AddressingMode::Accumulator, mnemomic: Mnemomic::ROR }),                  // 0x6A 
        Option::None,                                                                                                       // 0x6B [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndirect, mnemomic: Mnemomic::JMP }),             // 0x6C 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::ADC }),                     // 0x6D 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::ROR }),                     // 0x6E 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BBRN(6) }),   // 0x6F 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BVS }),       // 0x70 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirectIndexedY, mnemomic: Mnemomic::ADC }),     // 0x71 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirect, mnemomic: Mnemomic::ADC }),             // 0x72 
        Option::None,                                                                                                       // 0x73 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::STZ }),             // 0x74 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::ADC }),             // 0x75 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::ROR }),             // 0x76 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::RMBN(7) }),                 // 0x77 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::SEI }),                      // 0x78 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedY, mnemomic: Mnemomic::ADC }),             // 0x79 
        Option::Some(Operation { addressing_mode: AddressingMode::Stack, mnemomic: Mnemomic::PLY }),                        // 0x7A 
        Option::None,                                                                                                       // 0x7B [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedIndirect, mnemomic: Mnemomic::JMP }),      // 0x7C 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::ADC }),             // 0x7D 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::ROR }),             // 0x7E 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BBRN(7) }),   // 0x7F 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BRA }),       // 0x80 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedIndirect, mnemomic: Mnemomic::STA }),      // 0x81 
        Option::None,                                                                                                       // 0x82 [Invalid]
        Option::None,                                                                                                       // 0x83 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::STY }),                     // 0x84 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::STA }),                     // 0x85 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::STX }),                     // 0x86 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::SMBN(0) }),                 // 0x87 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::DEY }),                      // 0x88 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::BIT }),                    // 0x89 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::TXA }),                      // 0x8A 
        Option::None,                                                                                                       // 0x8B [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::STY }),                     // 0x8C 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::STA }),                     // 0x8D 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::STX }),                     // 0x8E 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BBSN(0) }),   // 0x8F 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BCC }),       // 0x90 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirectIndexedY, mnemomic: Mnemomic::STA }),     // 0x91 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirect, mnemomic: Mnemomic::STA }),             // 0x92 
        Option::None,                                                                                                       // 0x93 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::STY }),             // 0x94 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::STA }),             // 0x95 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedY, mnemomic: Mnemomic::STX }),             // 0x96 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::SMBN(1) }),                 // 0x97 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::TYA }),                      // 0x98 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedY, mnemomic: Mnemomic::STA }),             // 0x99 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::TXS }),                      // 0x9A 
        Option::None,                                                                                                       // 0x9B [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::STZ }),             // 0x9C 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::STA }),             // 0x9D 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedY, mnemomic: Mnemomic::STZ }),             // 0x9E 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BBSN(1) }),   // 0x9F 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::LDY }),                    // 0xA0 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedIndirect, mnemomic: Mnemomic::LDA }),      // 0xA1 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::LDX }),                    // 0xA2 
        Option::None,                                                                                                       // 0xA3 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::LDY }),                     // 0xA4 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::LDA }),                     // 0xA5 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::LDX }),                     // 0xA6 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::SMBN(2) }),                 // 0xA7 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::TAY }),                      // 0xA8 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::LDA }),                    // 0xA9 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::TAX }),                      // 0xAA 
        Option::None,                                                                                                       // 0xAB [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::LDY }),                     // 0xAC 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::LDA }),                     // 0xAD 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::LDX }),                     // 0xAE 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BBSN(2) }),   // 0xAF 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BCS }),       // 0xB0 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirectIndexedY, mnemomic: Mnemomic::LDA }),     // 0xB1 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirect, mnemomic: Mnemomic::LDA }),             // 0xB2 
        Option::None,                                                                                                       // 0xB3 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::LDY }),             // 0xB4 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::LDA }),             // 0xB5 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedY, mnemomic: Mnemomic::LDX }),             // 0xB6 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::SMBN(3) }),                 // 0xB7 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::CLV }),                      // 0xB8 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedY, mnemomic: Mnemomic::LDA }),             // 0xB9 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::TSX }),                      // 0xBA 
        Option::None,                                                                                                       // 0xBB [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::LDY }),             // 0xBC 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::LDA }),             // 0xBD 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedY, mnemomic: Mnemomic::LDX }),             // 0xBE 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BBSN(3) }),   // 0xBF 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::CPY }),                    // 0xC0 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedIndirect, mnemomic: Mnemomic::CMP }),      // 0xC1 
        Option::None,                                                                                                       // 0xC2 [Invalid]
        Option::None,                                                                                                       // 0xC3 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::CPY }),                     // 0xC4 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::CMP }),                     // 0xC5 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::DEC }),                     // 0xC6 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::SMBN(4) }),                 // 0xC7 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::INY }),                      // 0xC8 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::CMP }),                    // 0xC9 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::DEX }),                      // 0xCA 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::WAI }),                      // 0xCB 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::CPY }),                     // 0xCC 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::CMP }),                     // 0xCD 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::DEC }),                     // 0xCE 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BBSN(4) }),   // 0xCF 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BNE }),       // 0xD0 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirectIndexedY, mnemomic: Mnemomic::CMP }),     // 0xD1 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirect, mnemomic: Mnemomic::CMP }),             // 0xD2 
        Option::None,                                                                                                       // 0xD3 [Invalid]
        Option::None,                                                                                                       // 0xD4 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::CMP }),             // 0xD5 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::DEC }),             // 0xD6 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::SMBN(5) }),                 // 0xD7 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::CLD }),                      // 0xD8 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedY, mnemomic: Mnemomic::CMP }),             // 0xD9 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::PHX }),                      // 0xDA 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::STP }),                      // 0xDB [Invalid]
        Option::None,                                                                                                       // 0xDC [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::CMP }),             // 0xDD 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::DEC }),             // 0xDE 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BBSN(5) }),   // 0xDF 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::CPX }),                    // 0xE0 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedIndirect, mnemomic: Mnemomic::SBC }),      // 0xE1 
        Option::None,                                                                                                       // 0xE2 [Invalid]
        Option::None,                                                                                                       // 0xE3 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::CPX }),                     // 0xE4 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::SBC }),                     // 0xE5 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::INC }),                     // 0xE6 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::SMBN(6) }),                 // 0xE7 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::INX }),                      // 0xE8 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::SBC }),                    // 0xE9 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::NOP }),                      // 0xEA 
        Option::None,                                                                                                       // 0xEB [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::CPX }),                     // 0xEC 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::SBC }),                     // 0xED 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::INC }),                     // 0xEE 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BBSN(6) }),   // 0xEF 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BEQ }),       // 0xF0 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirectIndexedY, mnemomic: Mnemomic::SBC }),     // 0xF1 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirect, mnemomic: Mnemomic::SBC }),             // 0xF2 
        Option::None,                                                                                                       // 0xF3 [Invalid]
        Option::None,                                                                                                       // 0xF4 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::SBC }),             // 0xF5 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::INC }),             // 0xF6 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::SMBN(7) }),                 // 0xF7 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::SED }),                      // 0xF8 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedY, mnemomic: Mnemomic::SBC }),             // 0xF9 
        Option::Some(Operation { addressing_mode: AddressingMode::Stack, mnemomic: Mnemomic::PLX }),                        // 0xFA
        Option::None,                                                                                                       // 0xFB [Invalid] 
        Option::None,                                                                                                       // 0xFC [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::SBC }),             // 0xFD 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::INC }),             // 0xFE 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BBSN(7) }),   // 0xFF 
    ];

    //#GROUP: artery functions
    #[inline]
    fn fetch_u8(&mut self, bus: &mut dyn Bus) -> u8{
        let val = bus.read(self.program_counter.0);
        self.program_counter += 1;
        val
    }
    #[inline]
    fn fetch_u16(&mut self, bus: &mut dyn Bus) -> u16{
        let low = self.fetch_u8(bus) as u16;
        let high = self.fetch_u8(bus) as u16;
        (high << 8) | low
    }

    pub fn step(&mut self, bus: &mut dyn Bus) -> Result<(), CpuError>{
        let opcode = self.fetch_u8(bus);
        let operation = Self::OPERATIONS[opcode as usize].ok_or(CpuError::InvalidOpcode(opcode))?;

        let operand = resolve_operand(&mut self, bus, operation.addressing_mode);

        Ok(())
    }

    //#GROUP: processor status register helpers
    fn status_set(&mut self, flag: Status, val: bool){
        let mask = flag.mask();
        self.processor_status_register = (self.processor_status_register & !mask) | (mask * val as u8);
    }
    fn status_check(&mut self, flag: Status) -> bool{
        self.processor_status_register & flag.mask() > 0
    }

    fn set_p_default(&mut self){
        self.processor_status_register = 0x34; // 0b00110100
    }
    #[inline]
    fn set_p_negative_flag(&mut self, val: bool){
        let u8_val: u8 = (val as u8) << 7;
        self.processor_status_register = (self.processor_status_register & !(0x1 << 7)) | u8_val;
    }
    #[inline]
    fn set_p_overflow_flag(&mut self, val: bool){
        let u8_val: u8 = (val as u8) << 6;
        self.processor_status_register = (self.processor_status_register & !(0x1 << 6)) | u8_val;
    }
    #[inline]
    fn set_p_break_flag(&mut self, val: bool){
        let u8_val: u8 = (val as u8) << 4;
        self.processor_status_register = (self.processor_status_register & !(0x1 << 4)) | u8_val;
    }
    #[inline]
    fn set_p_decimal_flag(&mut self, val: bool){
        let u8_val: u8 = (val as u8) << 3;
        self.processor_status_register = (self.processor_status_register & !(0x1 << 3)) | u8_val;
    }
    #[inline]
    fn set_p_irq_disable_flag(&mut self, val: bool){
        let u8_val: u8 = (val as u8) << 2;
        self.processor_status_register = (self.processor_status_register & !(0x1 << 2)) | u8_val;
    }
    #[inline]
    fn set_p_zero_flag(&mut self, val: bool){
        let u8_val: u8 = (val as u8) << 1;
        self.processor_status_register = (self.processor_status_register & !(0x1 << 1)) | u8_val;
    }
    #[inline]
    fn set_p_carry_flag(&mut self, val: bool){
        let u8_val: u8 = val as u8;
        self.processor_status_register = (self.processor_status_register & !(0x1)) | u8_val;
    }
}

//#GROUP: op implementations
fn op_adc(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{
    let val = r.operand.read(cpu, bus)?;
    let sum = cpu.a_register as u16 + val as u16 + cpu.status_check(Status::C) as u16;

    cpu.status_set(Status::C, sum > 0xFF);

    Ok(())
}
fn op_and(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_asl(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_bbrn(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand, n: u8) -> Result<(), CpuError>{


    Ok(())
}
fn op_bbsn(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand, n: u8) -> Result<(), CpuError>{


    Ok(())
}
fn op_bcc(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_bcs(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_beq(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_bit(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_bmi(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_bne(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_bpl(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_bra(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_brk(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_bvc(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_bvs(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_clc(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_cld(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_cli(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_clv(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_cmp(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_cpx(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_cpy(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_dec(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_dex(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_dey(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_eor(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_inc(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_inx(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_iny(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_jmp(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_jsr(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_lda(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_ldx(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_ldy(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_lsr(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_nop(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_ora(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_pha(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_php(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_phx(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_phy(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_pla(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_plp(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_plx(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_ply(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_rmbn(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand, n: u8) -> Result<(), CpuError>{


    Ok(())
}
fn op_rol(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_ror(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_rti(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_rts(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_sbc(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_sec(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_sed(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_sei(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_smbn(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand, n: u8) -> Result<(), CpuError>{


    Ok(())
}
fn op_sta(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_stp(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_stx(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_sty(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_stz(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_tax(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_tay(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_trb(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_tsb(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_tsx(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_txa(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_txs(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_tya(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}
fn op_wai(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> Result<(), CpuError>{


    Ok(())
}

#[inline]
fn crosses_pages(a: u16, b: u16) -> bool{
    (a & 0xff00) != (b & 0xff00)
}
#[inline]
fn read_u16(bus: &mut dyn Bus, address: u16) -> u16{
    let low = bus.read(address) as u16;
    let high = bus.read(address.wrapping_add(1)) as u16;

    (high << 8) | low
}

fn resolve_operand(cpu: &mut W65C02S, bus: &mut dyn Bus, mode: AddressingMode) -> ResolvedOperand{
    match mode{
        AddressingMode::Absolute => {
            let val = cpu.fetch_u16(bus);
            ResolvedOperand{ operand: Operand::Address(val), page_crossed: false}
        },
        AddressingMode::AbsoluteIndexedIndirect => {
            let base = cpu.fetch_u16(bus);
            let addr = base.wrapping_add(cpu.x_register.0 as u16);

            let target = read_u16(bus, addr);
            ResolvedOperand{ operand: Operand::Address(target), page_crossed: false}
        },
        AddressingMode::AbsoluteIndexedX => {
            let base = cpu.fetch_u16(bus);
            let addr = base.wrapping_add(cpu.x_register.0 as u16);

            ResolvedOperand { operand: Operand::Address(addr), page_crossed: crosses_pages(base, addr) }
        },
        AddressingMode::AbsoluteIndexedY => {
            let base = cpu.fetch_u16(bus);
            let addr = base.wrapping_add(cpu.y_register.0 as u16);

            ResolvedOperand { operand: Operand::Address(addr), page_crossed: crosses_pages(base, addr) }
        },
        AddressingMode::AbsoluteIndirect => {
            let ptr = cpu.fetch_u16(bus);
            let target = read_u16(bus, ptr);

            ResolvedOperand { operand: Operand::Address(target), page_crossed: false }
        },
        AddressingMode::Accumulator => {
            ResolvedOperand { operand: Operand::Accumulator, page_crossed: false }
        },
        AddressingMode::Immediate => {
            let val = cpu.fetch_u8(bus);
            
            ResolvedOperand { operand: Operand::Value(val), page_crossed: false }
        },
        AddressingMode::Implied => {
            ResolvedOperand { operand: Operand::Implied, page_crossed: false }
        },
        AddressingMode::ProgramCounterRelative => {
            let offset = cpu.fetch_u8(bus) as i8;
            
            ResolvedOperand { operand: Operand::Relative(offset), page_crossed: false }
        },
        AddressingMode::Stack => {
            ResolvedOperand { operand: Operand::Implied, page_crossed: false }
        },
        AddressingMode::ZeroPage => {
            let addr = cpu.fetch_u8(bus) as u16;

            ResolvedOperand { operand: Operand::Address(addr), page_crossed: false }
        },
        AddressingMode::ZeroPageIndexedIndirect => {
            let zp_addr = cpu.fetch_u8(bus).wrapping_add(cpu.x_register.0);
            let low = bus.read(zp_addr as u16) as u16;
            let high = bus.read((zp_addr.wrapping_add(1)) as u16) as u16;

            let target = (high << 8) | low;
            ResolvedOperand { operand: Operand::Address(target), page_crossed: false }
        },
        AddressingMode::ZeroPageIndexedX => {
            let zp_addr = cpu.fetch_u8(bus).wrapping_add(cpu.x_register.0);

            ResolvedOperand { operand: Operand::Address(zp_addr as u16), page_crossed: false }
        },
        AddressingMode::ZeroPageIndexedY => {
            let zp_addr = cpu.fetch_u8(bus).wrapping_add(cpu.y_register.0);

            ResolvedOperand { operand: Operand::Address(zp_addr as u16), page_crossed: false }
        },
        AddressingMode::ZeroPageIndirect => {
            let zp_addr = cpu.fetch_u8(bus);
            let low = bus.read(zp_addr as u16) as u16;
            let high = bus.read((zp_addr.wrapping_add(1)) as u16) as u16;

            let target = (high << 8) | low;
            ResolvedOperand { operand: Operand::Address(target), page_crossed: false }
        },
        AddressingMode::ZeroPageIndirectIndexedY => {
            let zp_addr = cpu.fetch_u8(bus);
            let low = bus.read(zp_addr as u16) as u16;
            let high = bus.read((zp_addr.wrapping_add(1)) as u16) as u16;

            let target = ((high << 8) | low).wrapping_add(cpu.y_register.0 as u16);
            ResolvedOperand { operand: Operand::Address(target), page_crossed: false }
        },
    }
}

enum AddressingMode{
    Absolute,                   // a
    AbsoluteIndexedIndirect,    // (a, x)
    AbsoluteIndexedX,           // a, x
    AbsoluteIndexedY,           // a, y
    AbsoluteIndirect,           // (a)
    Accumulator,                // A
    Immediate,                  // #
    Implied,                    // i
    ProgramCounterRelative,     // r
    Stack,                      // s
    ZeroPage,                   // zp
    ZeroPageIndexedIndirect,    // (zp, x)
    ZeroPageIndexedX,           // zp, x
    ZeroPageIndexedY,           // zp, y
    ZeroPageIndirect,           // (zp)
    ZeroPageIndirectIndexedY    // (zp), y
}
impl AddressingMode{
    #[inline]
    fn num_operand_bytes(&self) -> u8{
        match *self{
            AddressingMode::Absolute => 2,
            AddressingMode::AbsoluteIndexedIndirect => 2,
            AddressingMode::AbsoluteIndexedX => 2,
            AddressingMode::AbsoluteIndexedY => 2,
            AddressingMode::AbsoluteIndirect => 2,
            AddressingMode::Accumulator => 0,
            AddressingMode::Immediate => 1,
            AddressingMode::Implied => 0,
            AddressingMode::ProgramCounterRelative => 1,
            AddressingMode::Stack => 0,
            AddressingMode::ZeroPage => 1,
            AddressingMode::ZeroPageIndexedIndirect => 1,
            AddressingMode::ZeroPageIndexedX => 1,
            AddressingMode::ZeroPageIndexedY => 1,
            AddressingMode::ZeroPageIndirect => 1,
            AddressingMode::ZeroPageIndirectIndexedY => 1,
        }
    }
}

struct Operation{
    addressing_mode: AddressingMode,
    mnemomic: Mnemomic,
}
enum Mnemomic{
    ADC,
    AND,
    ASL,
    BBRN(u8),
    BBSN(u8),
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRA,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PHX,
    PHY,
    PLA,
    PLP,
    PLX,
    PLY,
    RMBN(u8),
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    SMBN(u8),
    STA,
    STP,
    STX,
    STY,
    STZ,
    TAX,
    TAY,
    TRB,
    TSB,
    TSX,
    TXA,
    TXS,
    TYA,
    WAI,
}

#[derive(Debug)]
enum Operand{
    Implied,
    Accumulator,
    Value(u8),      // immediate
    Address(u16),   // value from memory
    Relative(i8)    // relative to PC
}
impl Operand{
    fn read(self, cpu: &W65C02S, bus: &mut dyn Bus) -> Result<u8, CpuError>{
        match self{
            Operand::Value(v) => Ok(v),
            Operand::Address(a) => Ok(bus.read(a)),
            Operand::Accumulator => Ok(cpu.a_register.0),
            _ => Err(CpuError::InvalidOperand(self))
        }
    }
    fn write(self, cpu: &mut W65C02S, bus: &mut dyn Bus, val: u8) -> Result<(), CpuError>{
        match self{
            Operand::Address(a) => { bus.write(a, val); Ok(()) },
            Operand::Accumulator => { cpu.a_register.0 = val; Ok(())},
            _ => Err(CpuError::InvalidOperand(self))
        }
    }
}
struct ResolvedOperand{
    operand: Operand,
    page_crossed: bool
}