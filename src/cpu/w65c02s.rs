use crate::bus::bus::{Bus};

#[derive(Debug)]
pub enum CpuError{
    InvalidOpcode(u8),
    InvalidOperand(Operand),
}

enum Status{
    C,  // Carry
    Z,  // Zero
    I,  // Interrupt Disable
    D,  // Decimal
    B,

    V,  // Overflow
    N,  // Negative
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
#[derive(Default)]
 pub struct W65C02S{
    program_counter: u16,
    a_register: u8,
    y_register: u8,
    x_register: u8,
    stack_pointer: u8,
    processor_status_register: u8,
}
impl W65C02S{
    // high byte for all vectors immediately follow the low byte in address space
    pub const IRQB_LOW: u16 = 0xFFFE; // At this address should be the lower 8 bits of the address to jump to when processing an interrupt request
    pub const RESB_LOW: u16 = 0xFFFC; // At this address should be the lower 8 bits of the address to jump to after resetting (ie. the entry point)
    pub const NMIB_LOW: u16 = 0xFFFA; // At this address should be the lower 8 bits of the address to jump to when processing a nonmaskable interrupt

    pub const STACK_POINTER_BASE: u16 = 0x0100; // When combined with the stack_pointer

    // invalids = [3, 19, 35, 51, 67, 83, 99, 115, 131, 147, 163, 179, 195, 211, 227, 243, 2, 34, 66, 98, 130, 194, 226, 68, 84, 212, 244, 11, 27, 43, 59, 75, 91, 107, 123, 139, 155, 171, 187, 235, 251, 92, 220, 252]
    pub const OPERATIONS: [Option<Operation>; 256] = [
        Option::Some(Operation { addressing_mode: AddressingMode::Stack, mnemomic: Mnemomic::BRK, exec: op_brk }),                          // 0x00 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedIndirect, mnemomic: Mnemomic::ORA, exec: op_ora }),        // 0x01 
        Option::None,                                                                                                                       // 0x02 [Invalid]
        Option::None,                                                                                                                       // 0x03 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::TSB, exec: op_tsb }),                       // 0x04 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::ORA, exec: op_ora }),                       // 0x05 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::ASL, exec: op_asl }),                       // 0x06 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::RMBN(0), exec: op_alias_rmb0 }),            // 0x07 
        Option::Some(Operation { addressing_mode: AddressingMode::Stack, mnemomic: Mnemomic::PHP, exec: op_php }),                          // 0x08 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::ORA, exec: op_ora }),                      // 0x09 
        Option::Some(Operation { addressing_mode: AddressingMode::Accumulator, mnemomic: Mnemomic::ASL, exec: op_asl }),                    // 0x0A 
        Option::None,                                                                                                                       // 0x0B [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::TSB, exec: op_tsb }),                       // 0x0C 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::ORA, exec: op_ora }),                       // 0x0D 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::ASL, exec: op_asl }),                       // 0x0E 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageRelative, mnemomic: Mnemomic::BBRN(0), exec: op_alias_bbr0 }),    // 0x0F 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BPL, exec: op_bpl }),         // 0x10 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirectIndexedY, mnemomic: Mnemomic::ORA, exec: op_ora }),       // 0x11 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirect, mnemomic: Mnemomic::ORA, exec: op_ora }),               // 0x12 
        Option::None,                                                                                                                       // 0x13 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::TRB, exec: op_trb }),                       // 0x14 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::ORA, exec: op_ora }),               // 0x15 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::ASL, exec: op_asl }),               // 0x16 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::RMBN(1), exec: op_alias_rmb1 }),            // 0x17 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::CLC, exec: op_clc }),                        // 0x18 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedY, mnemomic: Mnemomic::ORA, exec: op_ora }),               // 0x19 
        Option::Some(Operation { addressing_mode: AddressingMode::Accumulator, mnemomic: Mnemomic::INC, exec: op_inc }),                    // 0x1A 
        Option::None,                                                                                                                       // 0x1B [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::TRB, exec: op_trb }),                       // 0x1C 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::ORA, exec: op_ora }),               // 0x1D 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::ASL, exec: op_asl }),               // 0x1E 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageRelative, mnemomic: Mnemomic::BBRN(1), exec: op_alias_bbr1 }),    // 0x1F 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::JSR, exec: op_jsr }),                       // 0x20 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedIndirect, mnemomic: Mnemomic::AND, exec: op_and }),        // 0x21 
        Option::None,                                                                                                                       // 0x22 [Invalid]
        Option::None,                                                                                                                       // 0x23 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::BIT, exec: op_bit }),                       // 0x24 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::AND, exec: op_and }),                       // 0x25 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::ROL, exec: op_rol }),                       // 0x26 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::RMBN(2), exec: op_alias_rmb2 }),            // 0x27 
        Option::Some(Operation { addressing_mode: AddressingMode::Stack, mnemomic: Mnemomic::PLP, exec: op_plp }),                          // 0x28 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::AND, exec: op_and }),                      // 0x29 
        Option::Some(Operation { addressing_mode: AddressingMode::Accumulator, mnemomic: Mnemomic::ROL, exec: op_rol }),                    // 0x2A 
        Option::None,                                                                                                                       // 0x2B [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::BIT, exec: op_bit }),                       // 0x2C 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::AND, exec: op_and }),                       // 0x2D 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::ROL, exec: op_rol }),                       // 0x2E 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageRelative, mnemomic: Mnemomic::BBRN(2), exec: op_alias_bbr2 }),    // 0x2F 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BMI, exec: op_bmi }),         // 0x30 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirectIndexedY, mnemomic: Mnemomic::AND, exec: op_and }),       // 0x31 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirect, mnemomic: Mnemomic::AND, exec: op_and }),               // 0x32 
        Option::None,                                                                                                                       // 0x33 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::BIT, exec: op_bit }),               // 0x34 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::AND, exec: op_and }),               // 0x35 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::ROL, exec: op_rol }),               // 0x36 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::RMBN(3), exec: op_alias_rmb3 }),            // 0x37 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::SEC, exec: op_sec }),                        // 0x38 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedY, mnemomic: Mnemomic::AND, exec: op_and }),               // 0x39 
        Option::Some(Operation { addressing_mode: AddressingMode::Accumulator, mnemomic: Mnemomic::DEC, exec: op_dec }),                    // 0x3A 
        Option::None,                                                                                                                       // 0x3B [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::BIT, exec: op_bit }),               // 0x3C 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::AND, exec: op_and }),               // 0x3D 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::ROL, exec: op_rol }),               // 0x3E 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageRelative, mnemomic: Mnemomic::BBRN(3), exec: op_alias_bbr3 }),    // 0x3F 
        Option::Some(Operation { addressing_mode: AddressingMode::Stack, mnemomic: Mnemomic::RTI, exec: op_rti }),                          // 0x40 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedIndirect, mnemomic: Mnemomic::EOR, exec: op_eor }),        // 0x41 
        Option::None,                                                                                                                       // 0x42 [Invalid]
        Option::None,                                                                                                                       // 0x43 [Invalid]
        Option::None,                                                                                                                       // 0x44 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::EOR, exec: op_eor }),                       // 0x45 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::LSR, exec: op_lsr }),                       // 0x46 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::RMBN(4), exec: op_alias_rmb4 }),            // 0x47 
        Option::Some(Operation { addressing_mode: AddressingMode::Stack, mnemomic: Mnemomic::PHA, exec: op_pha }),                          // 0x48 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::EOR, exec: op_eor }),                      // 0x49 
        Option::Some(Operation { addressing_mode: AddressingMode::Accumulator, mnemomic: Mnemomic::LSR, exec: op_lsr }),                    // 0x4A 
        Option::None,                                                                                                                       // 0x4B [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::JMP, exec: op_jmp }),                       // 0x4C 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::EOR, exec: op_eor }),                       // 0x4D 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::LSR, exec: op_lsr }),                       // 0x4E 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageRelative, mnemomic: Mnemomic::BBRN(4), exec: op_alias_bbr4 }),    // 0x4F 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BVC, exec: op_bvc }),         // 0x50 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirectIndexedY, mnemomic: Mnemomic::EOR, exec: op_eor }),       // 0x51 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirect, mnemomic: Mnemomic::EOR, exec: op_eor }),               // 0x52 
        Option::None,                                                                                                                       // 0x53 [Invalid]
        Option::None,                                                                                                                       // 0x54 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::EOR, exec: op_eor }),               // 0x55 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::LSR, exec: op_lsr }),               // 0x56 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::RMBN(5), exec: op_alias_rmb5 }),            // 0x57 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::CLI, exec: op_cli }),                        // 0x58 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedY, mnemomic: Mnemomic::EOR, exec: op_eor }),               // 0x59 
        Option::Some(Operation { addressing_mode: AddressingMode::Stack, mnemomic: Mnemomic::PHY, exec: op_phy }),                          // 0x5A 
        Option::None,                                                                                                                       // 0x5B [Invalid]
        Option::None,                                                                                                                       // 0x5C [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::EOR, exec: op_eor }),               // 0x5D 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::LSR, exec: op_lsr }),               // 0x5E 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageRelative, mnemomic: Mnemomic::BBRN(5), exec: op_alias_bbr5 }),    // 0x5F 
        Option::Some(Operation { addressing_mode: AddressingMode::Stack, mnemomic: Mnemomic::RTS, exec: op_rts }),                          // 0x60 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedIndirect, mnemomic: Mnemomic::ADC, exec: op_adc }),        // 0x61 
        Option::None,                                                                                                                       // 0x62 [Invalid]
        Option::None,                                                                                                                       // 0x63 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::STZ, exec: op_stz }),                       // 0x64 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::ADC, exec: op_adc }),                       // 0x65 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::ROR, exec: op_ror }),                       // 0x66 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::RMBN(6), exec: op_alias_rmb6 }),            // 0x67 
        Option::Some(Operation { addressing_mode: AddressingMode::Stack, mnemomic: Mnemomic::PLA, exec: op_pla }),                          // 0x68 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::ADC, exec: op_adc }),                      // 0x69 
        Option::Some(Operation { addressing_mode: AddressingMode::Accumulator, mnemomic: Mnemomic::ROR, exec: op_ror }),                    // 0x6A 
        Option::None,                                                                                                                       // 0x6B [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndirect, mnemomic: Mnemomic::JMP, exec: op_jmp }),               // 0x6C 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::ADC, exec: op_adc }),                       // 0x6D 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::ROR, exec: op_ror }),                       // 0x6E 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageRelative, mnemomic: Mnemomic::BBRN(6), exec: op_alias_bbr6 }),    // 0x6F 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BVS, exec: op_bvs }),         // 0x70 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirectIndexedY, mnemomic: Mnemomic::ADC, exec: op_adc }),       // 0x71 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirect, mnemomic: Mnemomic::ADC, exec: op_adc }),               // 0x72 
        Option::None,                                                                                                                       // 0x73 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::STZ, exec: op_stz }),               // 0x74 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::ADC, exec: op_adc }),               // 0x75 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::ROR, exec: op_ror }),               // 0x76 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::RMBN(7), exec: op_alias_rmb7 }),            // 0x77 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::SEI, exec: op_sei }),                        // 0x78 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedY, mnemomic: Mnemomic::ADC, exec: op_adc }),               // 0x79 
        Option::Some(Operation { addressing_mode: AddressingMode::Stack, mnemomic: Mnemomic::PLY, exec: op_ply }),                          // 0x7A 
        Option::None,                                                                                                                       // 0x7B [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedIndirect, mnemomic: Mnemomic::JMP, exec: op_jmp }),        // 0x7C 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::ADC, exec: op_adc }),               // 0x7D 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::ROR, exec: op_ror }),               // 0x7E 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageRelative, mnemomic: Mnemomic::BBRN(7), exec: op_alias_bbr7 }),    // 0x7F 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BRA, exec: op_bra }),         // 0x80 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedIndirect, mnemomic: Mnemomic::STA, exec: op_sta }),        // 0x81 
        Option::None,                                                                                                                       // 0x82 [Invalid]
        Option::None,                                                                                                                       // 0x83 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::STY, exec: op_sty }),                       // 0x84 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::STA, exec: op_sta }),                       // 0x85 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::STX, exec: op_stx }),                       // 0x86 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::SMBN(0), exec: op_alias_smb0 }),            // 0x87 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::DEY, exec: op_dey }),                        // 0x88 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::BIT, exec: op_bit }),                      // 0x89 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::TXA, exec: op_txa }),                        // 0x8A 
        Option::None,                                                                                                                       // 0x8B [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::STY, exec: op_sty }),                       // 0x8C 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::STA, exec: op_sta }),                       // 0x8D 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::STX, exec: op_stx }),                       // 0x8E 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageRelative, mnemomic: Mnemomic::BBSN(0), exec: op_alias_bbs0 }),    // 0x8F 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BCC, exec: op_bcc }),         // 0x90 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirectIndexedY, mnemomic: Mnemomic::STA, exec: op_sta }),       // 0x91 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirect, mnemomic: Mnemomic::STA, exec: op_sta }),               // 0x92 
        Option::None,                                                                                                                       // 0x93 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::STY, exec: op_sty }),               // 0x94 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::STA, exec: op_sta }),               // 0x95 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedY, mnemomic: Mnemomic::STX, exec: op_stx }),               // 0x96 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::SMBN(1), exec: op_alias_smb1 }),            // 0x97 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::TYA, exec: op_tya }),                        // 0x98 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedY, mnemomic: Mnemomic::STA, exec: op_sta }),               // 0x99 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::TXS, exec: op_txs }),                        // 0x9A 
        Option::None,                                                                                                                       // 0x9B [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::STZ, exec: op_stz }),               // 0x9C 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::STA, exec: op_sta }),               // 0x9D 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedY, mnemomic: Mnemomic::STZ, exec: op_stz }),               // 0x9E 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageRelative, mnemomic: Mnemomic::BBSN(1), exec: op_alias_bbs1 }),    // 0x9F 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::LDY, exec: op_ldy }),                      // 0xA0 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedIndirect, mnemomic: Mnemomic::LDA, exec: op_lda }),        // 0xA1 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::LDX, exec: op_ldx }),                      // 0xA2 
        Option::None,                                                                                                                       // 0xA3 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::LDY, exec: op_ldy }),                       // 0xA4 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::LDA, exec: op_lda }),                       // 0xA5 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::LDX, exec: op_ldx }),                       // 0xA6 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::SMBN(2), exec: op_alias_smb2 }),            // 0xA7 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::TAY, exec: op_tay }),                        // 0xA8 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::LDA, exec: op_lda }),                      // 0xA9 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::TAX, exec: op_tax }),                        // 0xAA 
        Option::None,                                                                                                                       // 0xAB [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::LDY, exec: op_ldy }),                       // 0xAC 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::LDA, exec: op_lda }),                       // 0xAD 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::LDX, exec: op_ldx }),                       // 0xAE 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageRelative, mnemomic: Mnemomic::BBSN(2), exec: op_alias_bbs2 }),    // 0xAF 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BCS, exec: op_bcs }),         // 0xB0 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirectIndexedY, mnemomic: Mnemomic::LDA, exec: op_lda }),       // 0xB1 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirect, mnemomic: Mnemomic::LDA, exec: op_lda }),               // 0xB2 
        Option::None,                                                                                                                       // 0xB3 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::LDY, exec: op_ldy }),               // 0xB4 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::LDA, exec: op_lda }),               // 0xB5 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedY, mnemomic: Mnemomic::LDX, exec: op_ldx }),               // 0xB6 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::SMBN(3), exec: op_alias_smb3 }),            // 0xB7 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::CLV, exec: op_clv }),                        // 0xB8 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedY, mnemomic: Mnemomic::LDA, exec: op_lda }),               // 0xB9 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::TSX, exec: op_tsx }),                        // 0xBA 
        Option::None,                                                                                                                       // 0xBB [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::LDY, exec: op_ldy }),               // 0xBC 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::LDA, exec: op_lda }),               // 0xBD 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedY, mnemomic: Mnemomic::LDX, exec: op_ldx }),               // 0xBE 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageRelative, mnemomic: Mnemomic::BBSN(3), exec: op_alias_bbs3 }),    // 0xBF 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::CPY, exec: op_cpy }),                      // 0xC0 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedIndirect, mnemomic: Mnemomic::CMP, exec: op_cmp }),        // 0xC1 
        Option::None,                                                                                                                       // 0xC2 [Invalid]
        Option::None,                                                                                                                       // 0xC3 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::CPY, exec: op_cpy }),                       // 0xC4 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::CMP, exec: op_cmp }),                       // 0xC5 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::DEC, exec: op_dec }),                       // 0xC6 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::SMBN(4), exec: op_alias_smb4 }),            // 0xC7 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::INY, exec: op_iny }),                        // 0xC8 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::CMP, exec: op_cmp }),                      // 0xC9 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::DEX, exec: op_dex }),                        // 0xCA 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::WAI, exec: op_wai }),                        // 0xCB 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::CPY, exec: op_cpy }),                       // 0xCC 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::CMP, exec: op_cmp }),                       // 0xCD 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::DEC, exec: op_dec }),                       // 0xCE 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageRelative, mnemomic: Mnemomic::BBSN(4), exec: op_alias_bbs4 }),    // 0xCF 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BNE, exec: op_bne }),         // 0xD0 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirectIndexedY, mnemomic: Mnemomic::CMP, exec: op_cmp }),       // 0xD1 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirect, mnemomic: Mnemomic::CMP, exec: op_cmp }),               // 0xD2 
        Option::None,                                                                                                                       // 0xD3 [Invalid]
        Option::None,                                                                                                                       // 0xD4 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::CMP, exec: op_cmp }),               // 0xD5 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::DEC, exec: op_dec }),               // 0xD6 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::SMBN(5), exec: op_alias_smb5 }),            // 0xD7 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::CLD, exec: op_cld }),                        // 0xD8 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedY, mnemomic: Mnemomic::CMP, exec: op_cmp }),               // 0xD9 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::PHX, exec: op_phx }),                        // 0xDA 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::STP, exec: op_stp }),                        // 0xDB [Invalid]
        Option::None,                                                                                                                       // 0xDC [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::CMP, exec: op_cmp }),               // 0xDD 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::DEC, exec: op_dec }),               // 0xDE 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageRelative, mnemomic: Mnemomic::BBSN(5), exec: op_alias_bbs5 }),    // 0xDF 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::CPX, exec: op_cpx }),                      // 0xE0 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedIndirect, mnemomic: Mnemomic::SBC, exec: op_sbc }),        // 0xE1 
        Option::None,                                                                                                                       // 0xE2 [Invalid]
        Option::None,                                                                                                                       // 0xE3 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::CPX, exec: op_cpx }),                       // 0xE4 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::SBC, exec: op_sbc }),                       // 0xE5 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::INC, exec: op_inc }),                       // 0xE6 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::SMBN(6), exec: op_alias_smb6 }),            // 0xE7 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::INX, exec: op_inx }),                        // 0xE8 
        Option::Some(Operation { addressing_mode: AddressingMode::Immediate, mnemomic: Mnemomic::SBC, exec: op_sbc }),                      // 0xE9 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::NOP, exec: op_nop }),                        // 0xEA 
        Option::None,                                                                                                                       // 0xEB [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::CPX, exec: op_cpx }),                       // 0xEC 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::SBC, exec: op_sbc }),                       // 0xED 
        Option::Some(Operation { addressing_mode: AddressingMode::Absolute, mnemomic: Mnemomic::INC, exec: op_inc }),                       // 0xEE 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageRelative, mnemomic: Mnemomic::BBSN(6), exec: op_alias_bbs6 }),    // 0xEF 
        Option::Some(Operation { addressing_mode: AddressingMode::ProgramCounterRelative, mnemomic: Mnemomic::BEQ, exec: op_beq }),         // 0xF0 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirectIndexedY, mnemomic: Mnemomic::SBC, exec: op_sbc }),       // 0xF1 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndirect, mnemomic: Mnemomic::SBC, exec: op_sbc }),               // 0xF2 
        Option::None,                                                                                                                       // 0xF3 [Invalid]
        Option::None,                                                                                                                       // 0xF4 [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::SBC, exec: op_sbc }),               // 0xF5 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageIndexedX, mnemomic: Mnemomic::INC, exec: op_inc }),               // 0xF6 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPage, mnemomic: Mnemomic::SMBN(7), exec: op_alias_smb7 }),            // 0xF7 
        Option::Some(Operation { addressing_mode: AddressingMode::Implied, mnemomic: Mnemomic::SED, exec: op_sed }),                        // 0xF8 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedY, mnemomic: Mnemomic::SBC, exec: op_sbc }),               // 0xF9 
        Option::Some(Operation { addressing_mode: AddressingMode::Stack, mnemomic: Mnemomic::PLX, exec: op_plx }),                          // 0xFA
        Option::None,                                                                                                                       // 0xFB [Invalid] 
        Option::None,                                                                                                                       // 0xFC [Invalid]
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::SBC, exec: op_sbc }),               // 0xFD 
        Option::Some(Operation { addressing_mode: AddressingMode::AbsoluteIndexedX, mnemomic: Mnemomic::INC, exec: op_inc }),               // 0xFE 
        Option::Some(Operation { addressing_mode: AddressingMode::ZeroPageRelative, mnemomic: Mnemomic::BBSN(7), exec: op_alias_bbs7 }),    // 0xFF 
    ];

    //#GROUP: artery functions
    #[inline]
    fn fetch_u8(&mut self, bus: &mut dyn Bus) -> u8{
        let val = bus.read(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(1);
        val
    }
    #[inline]
    fn fetch_u16(&mut self, bus: &mut dyn Bus) -> u16{
        let low = self.fetch_u8(bus) as u16;
        let high = self.fetch_u8(bus) as u16;
        (high << 8) | low
    }

    #[inline]
    fn stack_push_u8(&mut self, bus: &mut dyn Bus, val: u8){
        bus.write(Self::STACK_POINTER_BASE | self.stack_pointer as u16, val);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }
    #[inline]
    fn stack_pull_u8(&mut self, bus: &mut dyn Bus) -> u8{
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        bus.read(Self::STACK_POINTER_BASE | self.stack_pointer as u16)
    }

    fn irq_run(&mut self, _bus: &mut dyn Bus){
        todo!();
    }
    fn nmi_run(&mut self, _bus: &mut dyn Bus){
        todo!();
    }

    pub fn reset(&mut self, bus: &mut dyn Bus){
        let entry = read_u16(bus, Self::RESB_LOW);
        self.set_p_default();
        self.program_counter = entry;
    }

    pub fn step(&mut self, bus: &mut dyn Bus) -> Result<Mnemomic, CpuError>{
        let opcode = self.fetch_u8(bus);
        let operation = Self::OPERATIONS[opcode as usize].as_ref().ok_or(CpuError::InvalidOpcode(opcode))?;
        
        let operand = resolve_operand(self, bus, &operation.addressing_mode);
        (operation.exec)(self, bus, operand)?;

        //check lines
        //run appropriate interrupt if applicable
        //nmi_run()
        //irq_run()

        Ok(operation.mnemomic)
    }

    //#GROUP: processor status register helpers
    #[inline]
    fn status_set(&mut self, flag: Status, val: bool){
        let mask = flag.mask();
        self.processor_status_register = (self.processor_status_register & !mask) | (mask * val as u8);
    }
    #[inline]
    fn status_check(&self, flag: Status) -> bool{
        self.processor_status_register & flag.mask() > 0
    }
    #[inline]
    fn status_update_zn(&mut self, val: u8){
        self.status_set(Status::Z, val == 0);
        self.status_set(Status::N, (val >> 7) > 0);
    }

    fn set_p_default(&mut self){
        self.processor_status_register = 0x34; // 0b00110100
    }
}

type OpReturn = Result<(), CpuError>;
type OpFn = fn(&mut W65C02S, &mut dyn Bus, ResolvedOperand) -> OpReturn;
//#GROUP: op implementations
fn op_adc(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    let val = r.operand.read(cpu, bus)?;
    let sum = cpu.a_register as u16 + val as u16 + cpu.status_check(Status::C) as u16;
    let result = sum as u8;

    cpu.status_set(Status::C, sum > 0xFF);

    let overflow = ((!(cpu.a_register ^ val) & (cpu.a_register ^ result)) & 0x80) != 0;

    cpu.status_set(Status::V, overflow);
    cpu.status_update_zn(result);

    cpu.a_register = result;

    Ok(())
}
fn op_and(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    let val = r.operand.read(cpu, bus)?;
    let result = cpu.a_register & val;

    cpu.status_update_zn(result);

    cpu.a_register = result;

    Ok(())
}
fn op_asl(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    let val = r.operand.read(cpu, bus)?;
    let result = val << 1;

    cpu.status_set(Status::C, (val & 0b1000_0000) > 0);
    cpu.status_update_zn(result);

    r.operand.write(cpu, bus, result)?;

    Ok(())
}
fn op_bbrn(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand, n: u8) -> OpReturn{
    let mask = 1u8 << n;

    match r.operand{
        Operand::ZpAddrRelative(addr, offset) => {
            let val = bus.read(addr as u16);

            if (val & mask) == 0{
                cpu.program_counter = cpu.program_counter.wrapping_add_signed(offset as i16);
            }

            Ok(())
        }
        _ => unreachable!(),
    }
}
fn op_bbsn(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand, n: u8) -> OpReturn{
    let mask = 1u8 << n;

    match r.operand{
        Operand::ZpAddrRelative(addr, offset) => {
            let val = bus.read(addr as u16);

            if (val & mask) > 0{
                cpu.program_counter = cpu.program_counter.wrapping_add_signed(offset as i16);
            }

            Ok(())
        }
        _ => unreachable!(),
    }
}
fn op_bcc(cpu: &mut W65C02S, _bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    match r.operand{
        Operand::Relative(offset) => { 
            if !cpu.status_check(Status::C){
                cpu.program_counter = cpu.program_counter.wrapping_add_signed(offset as i16);
            }

            Ok(())
        },
        _ => unreachable!(),
    }
}
fn op_bcs(cpu: &mut W65C02S, _bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    match r.operand{
        Operand::Relative(offset) => { 
            if cpu.status_check(Status::C){
                cpu.program_counter = cpu.program_counter.wrapping_add_signed(offset as i16);
            }

            Ok(())
        },
        _ => unreachable!(),
    }
}
fn op_beq(cpu: &mut W65C02S, _bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    match r.operand{
        Operand::Relative(offset) => { 
            if cpu.status_check(Status::Z){
                cpu.program_counter = cpu.program_counter.wrapping_add_signed(offset as i16);
            }

            Ok(())
        },
        _ => unreachable!(),
    }
}
fn op_bit(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    let val = r.operand.read(cpu, bus)?;
    cpu.status_set(Status::Z, (cpu.a_register & val) == 0);

    match r.operand {
        Operand::Value(_) => { },
        _ =>{
            cpu.status_set(Status::N, (val >> 7) > 0);
            cpu.status_set(Status::V, (val & 0b0100_0000) > 0);
        },
    }

    Ok(())
}
fn op_bmi(cpu: &mut W65C02S, _bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    match r.operand{
        Operand::Relative(offset) => { 
            if cpu.status_check(Status::N){
                cpu.program_counter = cpu.program_counter.wrapping_add_signed(offset as i16);
            }

            Ok(())
        },
        _ => unreachable!(),
    }
}
fn op_bne(cpu: &mut W65C02S, _bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    match r.operand{
        Operand::Relative(offset) => { 
            if !cpu.status_check(Status::Z){
                cpu.program_counter = cpu.program_counter.wrapping_add_signed(offset as i16);
            }

            Ok(())
        },
        _ => unreachable!(),
    }
}
fn op_bpl(cpu: &mut W65C02S, _bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    match r.operand{
        Operand::Relative(offset) => { 
            if !cpu.status_check(Status::N){
                cpu.program_counter = cpu.program_counter.wrapping_add_signed(offset as i16);
            }

            Ok(())
        },
        _ => unreachable!(),
    }
}
fn op_bra(cpu: &mut W65C02S, _bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    match r.operand{
        Operand::Relative(offset) => {
            cpu.program_counter = cpu.program_counter.wrapping_add_signed(offset as i16);

            Ok(())
        },
        _ => unreachable!(),
    }
}
fn op_brk(cpu: &mut W65C02S, bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    let return_addr = cpu.program_counter.wrapping_add(1);

    cpu.stack_push_u8(bus, (return_addr >> 8) as u8);
    cpu.stack_push_u8(bus, (return_addr & 0xff) as u8);
    cpu.stack_push_u8(bus, cpu.processor_status_register | 0x10);

    cpu.status_set(Status::I, true);

    let low = bus.read(W65C02S::IRQB_LOW) as u16;
    let high = bus.read(W65C02S::IRQB_LOW + 1) as u16;
    let target = (high << 8) | low;

    cpu.program_counter = target;

    Ok(())
}
fn op_bvc(cpu: &mut W65C02S, _bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    match r.operand{
        Operand::Relative(offset) => { 
            if !cpu.status_check(Status::V){
                cpu.program_counter = cpu.program_counter.wrapping_add_signed(offset as i16);
            }

            Ok(())
        },
        _ => unreachable!(),
    }
}
fn op_bvs(cpu: &mut W65C02S, _bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    match r.operand{
        Operand::Relative(offset) => { 
            if cpu.status_check(Status::V){
                cpu.program_counter = cpu.program_counter.wrapping_add_signed(offset as i16);
            }

            Ok(())
        },
        _ => unreachable!(),
    }
}
fn op_clc(cpu: &mut W65C02S, _bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.status_set(Status::C, false);

    Ok(())
}
fn op_cld(cpu: &mut W65C02S, _bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.status_set(Status::D, false);

    Ok(())
}
fn op_cli(cpu: &mut W65C02S, _bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.status_set(Status::I, false);

    Ok(())
}
fn op_clv(cpu: &mut W65C02S, _bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.status_set(Status::V, false);

    Ok(())
}
fn op_cmp(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    let val = r.operand.read(cpu, bus)?;
    let result = cpu.a_register.wrapping_sub(val);

    cpu.status_set(Status::C, cpu.a_register >= val);
    cpu.status_update_zn(result);

    Ok(())
}
fn op_cpx(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    let val = r.operand.read(cpu, bus)?;
    let result = cpu.x_register.wrapping_sub(val);

    cpu.status_set(Status::C, cpu.x_register >= val);
    cpu.status_update_zn(result);

    Ok(())
}
fn op_cpy(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    let val = r.operand.read(cpu, bus)?;
    let result = cpu.y_register.wrapping_sub(val);

    cpu.status_set(Status::C, cpu.y_register >= val);
    cpu.status_update_zn(result);

    Ok(())
}
fn op_dec(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    let val = r.operand.read(cpu, bus)?;
    let result = val.wrapping_sub(1);

    cpu.status_update_zn(result);

    r.operand.write(cpu, bus, result)?;

    Ok(())
}
fn op_dex(cpu: &mut W65C02S, _bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    let result = cpu.x_register.wrapping_sub(1);

    cpu.status_update_zn(result);

    cpu.x_register = result;

    Ok(())
}
fn op_dey(cpu: &mut W65C02S, _bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    let result = cpu.y_register.wrapping_sub(1);

    cpu.status_update_zn(result);

    cpu.y_register = result;

    Ok(())
}
fn op_eor(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    let val = r.operand.read(cpu, bus)?;
    let result = cpu.a_register ^ val;

    cpu.status_update_zn(result);

    cpu.a_register = result;

    Ok(())
}
fn op_inc(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    let val = r.operand.read(cpu, bus)?;
    let result = val.wrapping_add(1);

    cpu.status_update_zn(result);

    r.operand.write(cpu, bus, result)?;

    Ok(())
}
fn op_inx(cpu: &mut W65C02S, _bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    let result = cpu.x_register.wrapping_add(1);
    
    cpu.status_update_zn(result);

    cpu.x_register = result;

    Ok(())
}
fn op_iny(cpu: &mut W65C02S, _bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    let result = cpu.y_register.wrapping_add(1);
    
    cpu.status_update_zn(result);

    cpu.y_register = result;

    Ok(())
}
fn op_jmp(cpu: &mut W65C02S, _bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    match r.operand{
        Operand::Address(addr) => { cpu.program_counter = addr; Ok(())},
        _ => unreachable!(),
    }
}
fn op_jsr(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    match r.operand{
        Operand::Address(addr) => {
            let return_addr = cpu.program_counter.wrapping_sub(1);
            let return_low = (return_addr & 0x00ff) as u8;
            let return_high = (return_addr >> 8) as u8;

            cpu.stack_push_u8(bus, return_high);
            cpu.stack_push_u8(bus, return_low);

            cpu.program_counter = addr;

            Ok(())
        }
        _ => unreachable!()
    }
}
fn op_lda(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    cpu.a_register = r.operand.read(cpu, bus)?;

    cpu.status_update_zn(cpu.a_register);

    Ok(())
}
fn op_ldx(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    cpu.x_register = r.operand.read(cpu, bus)?;

    cpu.status_update_zn(cpu.x_register);

    Ok(())
}
fn op_ldy(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    cpu.y_register = r.operand.read(cpu, bus)?;

    cpu.status_update_zn(cpu.y_register);

    Ok(())
}
fn op_lsr(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    let val = r.operand.read(cpu, bus)?;
    let result = val >> 1;

    cpu.status_set(Status::C, (val & 1) > 0);
    cpu.status_update_zn(result);

    r.operand.write(cpu, bus, result)?;

    Ok(())
}
fn op_nop(_cpu: &mut W65C02S, _bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    Ok(())
}
fn op_ora(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    let val = r.operand.read(cpu, bus)?;
    let result = cpu.a_register | val;

    cpu.status_update_zn(result);

    cpu.a_register = result;

    Ok(())
}
fn op_pha(cpu: &mut W65C02S, bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.stack_push_u8(bus, cpu.a_register);

    Ok(())
}
fn op_php(cpu: &mut W65C02S, bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.stack_push_u8(bus, cpu.processor_status_register | 0x30);

    Ok(())
}
fn op_phx(cpu: &mut W65C02S, bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.stack_push_u8(bus, cpu.x_register);

    Ok(())
}
fn op_phy(cpu: &mut W65C02S, bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.stack_push_u8(bus, cpu.y_register);

    Ok(())
}
fn op_pla(cpu: &mut W65C02S, bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.a_register = cpu.stack_pull_u8(bus);

    cpu.status_update_zn(cpu.a_register);

    Ok(())
}
fn op_plp(cpu: &mut W65C02S, bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.processor_status_register = (cpu.stack_pull_u8(bus) | 0x20) & (!0x10);

    Ok(())
}
fn op_plx(cpu: &mut W65C02S, bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.x_register = cpu.stack_pull_u8(bus);

    cpu.status_update_zn(cpu.x_register);

    Ok(())
}
fn op_ply(cpu: &mut W65C02S, bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.y_register = cpu.stack_pull_u8(bus);

    cpu.status_update_zn(cpu.y_register);

    Ok(())
}
fn op_rmbn(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand, n: u8) -> OpReturn{
    let val = r.operand.read(cpu, bus)?;
    let mask = 1u8 << n;

    r.operand.write(cpu, bus, val & (!mask))?;

    Ok(())
}
fn op_rol(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    let val = r.operand.read(cpu, bus)?;
    let c = (val >> 7) > 0;
    let result = (val << 1) | (cpu.status_check(Status::C) as u8);

    cpu.status_set(Status::C, c);
    cpu.status_update_zn(result);

    r.operand.write(cpu, bus, result)?;

    Ok(())
}
fn op_ror(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    let val = r.operand.read(cpu, bus)?;
    let c = (val & 1) > 0;
    let result = (val >> 1) | ((cpu.status_check(Status::C) as u8) << 7);

    cpu.status_set(Status::C, c);
    cpu.status_update_zn(result);

    r.operand.write(cpu, bus, result)?;

    Ok(())
}
fn op_rti(cpu: &mut W65C02S, bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    let p = (cpu.stack_pull_u8(bus) | 0x20) & (!0x10);

    let low = cpu.stack_pull_u8(bus);
    let high = cpu.stack_pull_u8(bus);
    let addr = ((high as u16) << 8) | (low as u16);

    cpu.processor_status_register = p;
    cpu.program_counter = addr;

    Ok(())
}
fn op_rts(cpu: &mut W65C02S, bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    let low = cpu.stack_pull_u8(bus);
    let high = cpu.stack_pull_u8(bus);
    let addr = ((high as u16) << 8) | (low as u16);

    cpu.program_counter = addr.wrapping_add(1);

    Ok(())
}
fn op_sbc(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    let val = r.operand.read(cpu, bus)?;
    let diff = (cpu.a_register as u16).wrapping_add(!val as u16).wrapping_add(cpu.status_check(Status::C) as u16);
    let result = diff as u8;

    cpu.status_set(Status::C, diff > 0xff);
    cpu.status_set(Status::V, ((result ^ cpu.a_register) & (cpu.a_register ^ val) & 0x80) != 0);
    cpu.status_update_zn(result);

    cpu.a_register = result;

    Ok(())
}
fn op_sec(cpu: &mut W65C02S, _bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.status_set(Status::C, true);

    Ok(())
}
fn op_sed(cpu: &mut W65C02S, _bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.status_set(Status::D, true);

    Ok(())
}
fn op_sei(cpu: &mut W65C02S, _bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.status_set(Status::I, true);

    Ok(())
}
fn op_smbn(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand, n: u8) -> OpReturn{
    let val = r.operand.read(cpu, bus)?;
    let mask = 1u8 << n;

    r.operand.write(cpu, bus, val | mask)?;

    Ok(())
}
fn op_sta(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    r.operand.write(cpu, bus, cpu.a_register)?;

    Ok(())
}
fn op_stp(_cpu: &mut W65C02S, _bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{ //
    unimplemented!();
}
fn op_stx(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    r.operand.write(cpu, bus, cpu.x_register)?;

    Ok(())
}
fn op_sty(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    r.operand.write(cpu, bus, cpu.y_register)?;

    Ok(())
}
fn op_stz(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    r.operand.write(cpu, bus, 0)?;

    Ok(())
}
fn op_tax(cpu: &mut W65C02S, _bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.x_register = cpu.a_register;

    cpu.status_update_zn(cpu.x_register);

    Ok(())
}
fn op_tay(cpu: &mut W65C02S, _bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.y_register = cpu.a_register;

    cpu.status_update_zn(cpu.y_register);

    Ok(())
}
fn op_trb(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    let val = r.operand.read(cpu, bus)?;
    let result = val & !cpu.a_register;

    cpu.status_set(Status::Z, (val & cpu.a_register) == 0);

    r.operand.write(cpu, bus, result)?;

    Ok(())
}
fn op_tsb(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn{
    let val = r.operand.read(cpu, bus)?;
    let result = val | cpu.a_register;

    cpu.status_set(Status::Z, (val & cpu.a_register) == 0);

    r.operand.write(cpu, bus, result)?;

    Ok(())
}
fn op_tsx(cpu: &mut W65C02S, _bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.x_register = cpu.stack_pointer;

    cpu.status_update_zn(cpu.x_register);

    Ok(())
}
fn op_txa(cpu: &mut W65C02S, _bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.a_register = cpu.x_register;

    cpu.status_update_zn(cpu.a_register);

    Ok(())
}
fn op_txs(cpu: &mut W65C02S, _bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.stack_pointer = cpu.x_register;

    Ok(())
}
fn op_tya(cpu: &mut W65C02S, _bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{
    cpu.a_register = cpu.y_register;

    cpu.status_update_zn(cpu.a_register);

    Ok(())
}
fn op_wai(_cpu: &mut W65C02S, _bus: &mut dyn Bus, _r: ResolvedOperand) -> OpReturn{ //
    unimplemented!();
}

fn op_alias_bbr0(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_bbrn(cpu, bus, r, 0) }
fn op_alias_bbr1(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_bbrn(cpu, bus, r, 1) }
fn op_alias_bbr2(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_bbrn(cpu, bus, r, 2) }
fn op_alias_bbr3(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_bbrn(cpu, bus, r, 3) }
fn op_alias_bbr4(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_bbrn(cpu, bus, r, 4) }
fn op_alias_bbr5(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_bbrn(cpu, bus, r, 5) }
fn op_alias_bbr6(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_bbrn(cpu, bus, r, 6) }
fn op_alias_bbr7(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_bbrn(cpu, bus, r, 7) }
fn op_alias_bbs0(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_bbsn(cpu, bus, r, 0) }
fn op_alias_bbs1(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_bbsn(cpu, bus, r, 1) }
fn op_alias_bbs2(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_bbsn(cpu, bus, r, 2) }
fn op_alias_bbs3(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_bbsn(cpu, bus, r, 3) }
fn op_alias_bbs4(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_bbsn(cpu, bus, r, 4) }
fn op_alias_bbs5(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_bbsn(cpu, bus, r, 5) }
fn op_alias_bbs6(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_bbsn(cpu, bus, r, 6) }
fn op_alias_bbs7(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_bbsn(cpu, bus, r, 7) }
fn op_alias_rmb0(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_rmbn(cpu, bus, r, 0) }
fn op_alias_rmb1(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_rmbn(cpu, bus, r, 1) }
fn op_alias_rmb2(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_rmbn(cpu, bus, r, 2) }
fn op_alias_rmb3(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_rmbn(cpu, bus, r, 3) }
fn op_alias_rmb4(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_rmbn(cpu, bus, r, 4) }
fn op_alias_rmb5(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_rmbn(cpu, bus, r, 5) }
fn op_alias_rmb6(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_rmbn(cpu, bus, r, 6) }
fn op_alias_rmb7(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_rmbn(cpu, bus, r, 7) }
fn op_alias_smb0(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_smbn(cpu, bus, r, 0) }
fn op_alias_smb1(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_smbn(cpu, bus, r, 1) }
fn op_alias_smb2(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_smbn(cpu, bus, r, 2) }
fn op_alias_smb3(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_smbn(cpu, bus, r, 3) }
fn op_alias_smb4(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_smbn(cpu, bus, r, 4) }
fn op_alias_smb5(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_smbn(cpu, bus, r, 5) }
fn op_alias_smb6(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_smbn(cpu, bus, r, 6) }
fn op_alias_smb7(cpu: &mut W65C02S, bus: &mut dyn Bus, r: ResolvedOperand) -> OpReturn { op_smbn(cpu, bus, r, 7) }

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

fn resolve_operand(cpu: &mut W65C02S, bus: &mut dyn Bus, mode: &AddressingMode) -> ResolvedOperand{
    match mode{
        AddressingMode::Absolute => {
            let val = cpu.fetch_u16(bus);
            ResolvedOperand{ operand: Operand::Address(val), page_crossed: false}
        },
        AddressingMode::AbsoluteIndexedIndirect => {
            let base = cpu.fetch_u16(bus);
            let addr = base.wrapping_add(cpu.x_register as u16);

            let target = read_u16(bus, addr);
            ResolvedOperand{ operand: Operand::Address(target), page_crossed: false}
        },
        AddressingMode::AbsoluteIndexedX => {
            let base = cpu.fetch_u16(bus);
            let addr = base.wrapping_add(cpu.x_register as u16);

            ResolvedOperand { operand: Operand::Address(addr), page_crossed: crosses_pages(base, addr) }
        },
        AddressingMode::AbsoluteIndexedY => {
            let base = cpu.fetch_u16(bus);
            let addr = base.wrapping_add(cpu.y_register as u16);

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
            let zp_addr = cpu.fetch_u8(bus).wrapping_add(cpu.x_register);
            let low = bus.read(zp_addr as u16) as u16;
            let high = bus.read((zp_addr.wrapping_add(1)) as u16) as u16;

            let target = (high << 8) | low;
            ResolvedOperand { operand: Operand::Address(target), page_crossed: false }
        },
        AddressingMode::ZeroPageIndexedX => {
            let zp_addr = cpu.fetch_u8(bus).wrapping_add(cpu.x_register);

            ResolvedOperand { operand: Operand::Address(zp_addr as u16), page_crossed: false }
        },
        AddressingMode::ZeroPageIndexedY => {
            let zp_addr = cpu.fetch_u8(bus).wrapping_add(cpu.y_register);

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

            let base = (high << 8) | low;
            let target = base.wrapping_add(cpu.y_register as u16);
            ResolvedOperand { operand: Operand::Address(target), page_crossed: crosses_pages(base, target) }
        },

        AddressingMode::ZeroPageRelative => {
            let zp_addr = cpu.fetch_u8(bus);
            let rel = cpu.fetch_u8(bus) as i8;

            ResolvedOperand { operand: Operand::ZpAddrRelative(zp_addr, rel), page_crossed: false }
        }
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
    ZeroPageIndirectIndexedY,   // (zp), y

    ZeroPageRelative,           // for BBRN and BBSN
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

            AddressingMode::ZeroPageRelative => 2,
        }
    }
}

pub struct Operation{
    addressing_mode: AddressingMode,
    mnemomic: Mnemomic,
    exec: OpFn,
}

#[derive(Copy, Clone, Debug)]
pub enum Mnemomic{
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
impl Mnemomic{
    pub fn from_str(mnem: &str) -> Option<Self>{
        match mnem.to_lowercase().as_str(){
            "adc" => Some(Mnemomic::ADC),
            "and" => Some(Mnemomic::AND),
            "asl" => Some(Mnemomic::ASL),
            "bbr0" => Some(Mnemomic::BBRN(0)),
            "bbr1" => Some(Mnemomic::BBRN(1)),
            "bbr2" => Some(Mnemomic::BBRN(2)),
            "bbr3" => Some(Mnemomic::BBRN(3)),
            "bbr4" => Some(Mnemomic::BBRN(4)),
            "bbr5" => Some(Mnemomic::BBRN(5)),
            "bbr6" => Some(Mnemomic::BBRN(6)),
            "bbr7" => Some(Mnemomic::BBRN(7)),
            "bbs0" => Some(Mnemomic::BBSN(0)),
            "bbs1" => Some(Mnemomic::BBSN(1)),
            "bbs2" => Some(Mnemomic::BBSN(2)),
            "bbs3" => Some(Mnemomic::BBSN(3)),
            "bbs4" => Some(Mnemomic::BBSN(4)),
            "bbs5" => Some(Mnemomic::BBSN(5)),
            "bbs6" => Some(Mnemomic::BBSN(6)),
            "bbs7" => Some(Mnemomic::BBSN(7)),
            "bcc" => Some(Mnemomic::BCC),
            "bcs" => Some(Mnemomic::BCS),
            "beq" => Some(Mnemomic::BEQ),
            "bit" => Some(Mnemomic::BIT),
            "bmi" => Some(Mnemomic::BMI),
            "bne" => Some(Mnemomic::BNE),
            "bpl" => Some(Mnemomic::BPL),
            "bra" => Some(Mnemomic::BRA),
            "brk" => Some(Mnemomic::BRK),
            "bvc" => Some(Mnemomic::BVC),
            "bvs" => Some(Mnemomic::BVS),
            "clc" => Some(Mnemomic::CLC),
            "cld" => Some(Mnemomic::CLD),
            "cli" => Some(Mnemomic::CLI),
            "clv" => Some(Mnemomic::CLV),
            "cmp" => Some(Mnemomic::CMP),
            "cpx" => Some(Mnemomic::CPX),
            "cpy" => Some(Mnemomic::CPY),
            "dec" => Some(Mnemomic::DEC),
            "dex" => Some(Mnemomic::DEX),
            "dey" => Some(Mnemomic::DEY),
            "eor" => Some(Mnemomic::EOR),
            "inc" => Some(Mnemomic::INC),
            "inx" => Some(Mnemomic::INX),
            "iny" => Some(Mnemomic::INY),
            "jmp" => Some(Mnemomic::JMP),
            "jsr" => Some(Mnemomic::JSR),
            "lda" => Some(Mnemomic::LDA),
            "ldx" => Some(Mnemomic::LDX),
            "ldy" => Some(Mnemomic::LDY),
            "lsr" => Some(Mnemomic::LSR),
            "nop" => Some(Mnemomic::NOP),
            "ora" => Some(Mnemomic::ORA),
            "pha" => Some(Mnemomic::PHA),
            "php" => Some(Mnemomic::PHP),
            "phx" => Some(Mnemomic::PHX),
            "phy" => Some(Mnemomic::PHY),
            "pla" => Some(Mnemomic::PLA),
            "plp" => Some(Mnemomic::PLP),
            "plx" => Some(Mnemomic::PLX),
            "ply" => Some(Mnemomic::PLY),
            "rmb0" => Some(Mnemomic::RMBN(0)),
            "rmb1" => Some(Mnemomic::RMBN(1)),
            "rmb2" => Some(Mnemomic::RMBN(2)),
            "rmb3" => Some(Mnemomic::RMBN(3)),
            "rmb4" => Some(Mnemomic::RMBN(4)),
            "rmb5" => Some(Mnemomic::RMBN(5)),
            "rmb6" => Some(Mnemomic::RMBN(6)),
            "rmb7" => Some(Mnemomic::RMBN(7)),
            "rol" => Some(Mnemomic::ROL),
            "ror" => Some(Mnemomic::ROR),
            "rti" => Some(Mnemomic::RTI),
            "rts" => Some(Mnemomic::RTS),
            "sbc" => Some(Mnemomic::SBC),
            "sec" => Some(Mnemomic::SEC),
            "sed" => Some(Mnemomic::SED),
            "sei" => Some(Mnemomic::SEI),
            "smb0" => Some(Mnemomic::SMBN(0)),
            "smb1" => Some(Mnemomic::SMBN(1)),
            "smb2" => Some(Mnemomic::SMBN(2)),
            "smb3" => Some(Mnemomic::SMBN(3)),
            "smb4" => Some(Mnemomic::SMBN(4)),
            "smb5" => Some(Mnemomic::SMBN(5)),
            "smb6" => Some(Mnemomic::SMBN(6)),
            "smb7" => Some(Mnemomic::SMBN(7)),
            "sta" => Some(Mnemomic::STA),
            "stp" => Some(Mnemomic::STP),
            "stx" => Some(Mnemomic::STX),
            "sty" => Some(Mnemomic::STY),
            "stz" => Some(Mnemomic::STZ),
            "tax" => Some(Mnemomic::TAX),
            "tay" => Some(Mnemomic::TAY),
            "trb" => Some(Mnemomic::TRB),
            "tsb" => Some(Mnemomic::TSB),
            "tsx" => Some(Mnemomic::TSX),
            "txa" => Some(Mnemomic::TXA),
            "txs" => Some(Mnemomic::TXS),
            "tya" => Some(Mnemomic::TYA),
            "wai" => Some(Mnemomic::WAI),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Operand{
    Implied,
    Accumulator,
    Value(u8),          // immediate
    Address(u16),       // value from memory
    Relative(i8),       // relative to PC
    ZpAddrRelative(u8, i8)  // for BBRN and BBSn
}
impl Operand{
    fn read(self, cpu: &W65C02S, bus: &mut dyn Bus) -> Result<u8, CpuError>{
        match self{
            Operand::Value(v) => Ok(v),
            Operand::Address(a) => Ok(bus.read(a)),
            Operand::Accumulator => Ok(cpu.a_register),
            Operand::ZpAddrRelative(a, _) => Ok(bus.read(a as u16)),
            _ => Err(CpuError::InvalidOperand(self))
        }
    }
    fn write(self, cpu: &mut W65C02S, bus: &mut dyn Bus, val: u8) -> Result<(), CpuError>{
        match self{
            Operand::Address(a) => { bus.write(a, val); Ok(()) },
            Operand::Accumulator => { cpu.a_register = val; Ok(())},
            _ => Err(CpuError::InvalidOperand(self))
        }
    }
}
struct ResolvedOperand{
    operand: Operand,
    page_crossed: bool
}