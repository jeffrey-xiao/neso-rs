macro_rules! generate_instructions {
    (
        $cpu:ident,
        $opcode:expr,
        {
            $($instruction_fn:ident: ($((
                $opcode_matcher:pat,
                $addressing_mode:ident,
                $cycles:expr
            )$(,)*)*)$(,)*)*
        }
    ) => {
        match $opcode {
            $($(
                $opcode_matcher => {
                    $cpu.cycle += $cycles;
                    $instruction_fn($cpu, AddressingMode::$addressing_mode);
                }
            )*)*
            _ => panic!(format!("No matching instruction for: {:02x}", $opcode))
        }
    }
}

struct Cpu {
    pub cycle: u8,
    pub pc: u16,
    pub sp: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
}

struct CpuMemoryMap {

}

impl Cpu {
    fn pop_byte(&mut self) -> u8 { 0 }
    fn pop_word(&mut self) -> u16 { 0 }
    fn read_byte(&self, addr: u16) -> u8 { 0 }
    fn read_word(&self, addr: u16) -> u16 { 0 }

    fn execute_opcode(&mut self, opcode: u8) {
        generate_instructions!(self, opcode, {
            adc: (
                (0x61, IndirectX, 6),
                (0x65, ZeroPage, 3),
                (0x69, Immediate, 2),
                (0x6D, Absolute, 4),
                (0x71, IndirectY, 5),
                (0x75, ZeroPageX, 4),
                (0x79, AbsoluteY, 4),
                (0x7D, AbsoluteX, 4),
            ),
            and: (
                (0x29, Immediate, 2),
                (0x25, ZeroPage, 3),
                (0x35, ZeroPageX, 4),
                (0x2D, Absolute, 4),
                (0x3D, AbsoluteX, 4),
                (0x39, AbsoluteY, 4),
                (0x21, IndirectX, 6),
                (0x31, IndirectY, 5),
            ),
            asl: (
                (0x0A, Accumulator, 2),
                (0x06, ZeroPage, 5),
                (0x16, ZeroPageX, 6),
                (0x0E, Absolute, 6),
                (0x1E, AbsoluteX, 7),
            ),
            bcc: ((0x90, Relative, 2)),
            bcs: ((0xB0, Relative, 2)),
            beq: ((0xF0, Relative, 2)),
            bit: (
                (0x24, ZeroPage, 3),
                (0x2C, Absolute, 4),
            ),
            bmi: ((0x30, Relative, 2)),
            bne: ((0xD0, Relative, 2)),
            bpl: ((0x10, Relative, 2)),
            brk: ((0x00, Implied, 7)),
            bvc: ((0x50, Relative, 2)),
            bvs: ((0x70, Relative, 2)),
            clc: ((0x18, Implied, 2)),
            cld: ((0xD8, Implied, 2)),
            cli: ((0x58, Implied, 2)),
            clv: ((0xB8, Implied, 2)),
            cmp: (
                (0xC9, Immediate, 2),
                (0xC5, ZeroPage, 3),
                (0xD5, ZeroPageX, 4),
                (0xCD, Absolute, 4),
                (0xDD, AbsoluteX, 4),
                (0xD9, AbsoluteY, 4),
                (0xC1, IndirectX, 6),
                (0xD1, IndirectY, 5),
            ),
            cpx: (
                (0xE0, Immediate, 2),
                (0xE4, ZeroPage, 3),
                (0xEC, Absolute, 4),
            ),
            cpy: (
                (0xC0, Immediate, 2),
                (0xC4, ZeroPage, 3),
                (0xCC, Absolute, 4),
            ),
            dec: (
                (0xC6, ZeroPage, 5),
                (0xD6, ZeroPageX, 6),
                (0xCE, Absolute, 3),
                (0xDE, AbsoluteX, 7),
            ),
            dex: ((0xCA, Implied, 2)),
            dey: ((0x88, Implied, 2)),
            eor: (
                (0x49, Immediate, 2),
                (0x45, ZeroPage, 3),
                (0x55, ZeroPageX, 4),
                (0x4D, Absolute, 4),
                (0x5D, AbsoluteX, 4),
                (0x59, AbsoluteY, 4),
                (0x41, IndirectX, 6),
                (0x51, IndirectY, 5),
            ),
            inc: (
                (0xE6, ZeroPage, 5),
                (0xF6, ZeroPageX, 6),
                (0xEE, Absolute, 6),
                (0xFE, AbsoluteX, 7),
            ),
            inx: ((0xE8, Implied, 2)),
            iny: ((0xC8, Implied, 2)),
            jmp: (
                (0x4C, Absolute, 3),
                (0x6C, Indirect, 5),
            ),
            jsr: ((0x20, Absolute, 6)),
            lda: (
                (0xA9, Immediate, 2),
                (0xA5, ZeroPage, 3),
                (0xB5, ZeroPageX, 4),
                (0xAD, Absolute, 4),
                (0xBD, AbsoluteX, 4),
                (0xB9, AbsoluteY, 4),
                (0xA1, IndirectX, 6),
                (0xB1, IndirectY, 5),
            ),
            ldx: (
                (0xA2, Immediate, 2),
                (0xA6, ZeroPage, 3),
                (0xB6, ZeroPageY, 4),
                (0xAE, Absolute, 4),
                (0xBE, AbsoluteY, 4),
            ),
            ldy: (
                (0xA0, Immediate, 2),
                (0xA4, ZeroPage, 3),
                (0xB4, ZeroPageX, 4),
                (0xAC, Absolute, 4),
                (0xBC, AbsoluteX, 4),
            ),
            lsr: (
                (0x4A, Accumulator, 2),
                (0x46, ZeroPage, 5),
                (0x56, ZeroPageX, 6),
                (0x4E, Absolute, 6),
                (0x5E, AbsoluteX, 7),
            ),
            nop: ((0xEA, Implied, 2)),
            ora: (
                (0x09, Immediate, 2),
                (0x05, ZeroPage, 3),
                (0x15, ZeroPageX, 4),
                (0x0D, Absolute, 4),
                (0x1D, AbsoluteX, 4),
                (0x19, AbsoluteY, 4),
                (0x01, IndirectX, 6),
                (0x11, IndirectY, 5),
            ),
            pha: ((0x48, Implied, 3)),
            php: ((0x08, Implied, 3)),
            pla: ((0x68, Implied, 4)),
            plp: ((0x28, Implied, 4)),
            rol: (
                (0x2A, Accumulator, 2),
                (0x26, ZeroPage, 5),
                (0x36, ZeroPageX, 6),
                (0x2E, Absolute, 6),
                (0x3E, AbsoluteX, 7),
            ),
            ror: (
                (0x6A, Accumulator, 2),
                (0x66, ZeroPage, 5),
                (0x76, ZeroPageX, 6),
                (0x6E, Absolute, 6),
                (0x7E, AbsoluteX, 7),
            ),
            rti: ((0x40, Implied, 6)),
            rts: ((0x60, Implied, 6)),
            sbc: (
                (0xE9, Immediate, 2),
                (0xE5, ZeroPage, 3),
                (0xF5, ZeroPageX, 4),
                (0xED, Absolute, 4),
                (0xFD, AbsoluteX, 4),
                (0xF9, AbsoluteY, 4),
                (0xE1, IndirectX, 6),
                (0xF1, IndirectY, 5),
            ),
            sec: ((0x38, Implied, 2)),
            sed: ((0xF8, Implied, 2)),
            sei: ((0x78, Implied, 2)),
            sta: (
                (0x85, ZeroPage, 3),
                (0x95, ZeroPageX, 4),
                (0x8D, Absolute, 4),
                (0x9D, AbsoluteX, 5),
                (0x99, AbsoluteY, 5),
                (0x81, IndirectX, 6),
                (0x91, IndirectY, 6),
            ),
            stx: (
                (0x86, ZeroPage, 3),
                (0x96, ZeroPageY, 4),
                (0x8E, Absolute, 4),
            ),
            sty: (
                (0x84, ZeroPage, 3),
                (0x94, ZeroPageX, 4),
                (0x8C, Absolute, 4),
            ),
            tax: ((0xAA, Implied, 2)),
            tay: ((0xA8, Implied, 2)),
            tsx: ((0xBA, Implied, 2)),
            txa: ((0x8A, Implied, 2)),
            txs: ((0x9A, Implied, 2)),
            tya: ((0x98, Implied, 2)),
        })
    }
}

fn adc(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn and(cpu: &mut Cpu, addressing_mode: AddressingMode) {

}
fn asl(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn bcc(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn bcs(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn beq(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn bit(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn bmi(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn bne(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn bpl(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn brk(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn bvc(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn bvs(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn clc(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn cld(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn cli(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn clv(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn cmp(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn cpx(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn cpy(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn dec(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn dex(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn dey(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn eor(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn inc(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn inx(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn iny(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn jmp(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn jsr(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn lda(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn ldx(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn ldy(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn lsr(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn nop(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn ora(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn pha(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn php(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn pla(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn plp(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn rol(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn ror(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn rti(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn rts(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn sbc(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn sec(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn sed(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn sei(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn sta(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn stx(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn sty(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn tax(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn tay(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn tsx(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn txa(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn txs(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn tya(cpu: &mut Cpu, addressing_mode: AddressingMode) {}

enum AddressingMode {
    Absolute = 0,
    AbsoluteX = 1,
    AbsoluteY = 2,
    Accumulator = 3,
    Immediate = 4,
    Implied = 5,
    Indirect = 6,
    IndirectX = 7,
    IndirectY = 8,
    Relative = 9,
    ZeroPage = 10,
    ZeroPageX = 11,
    ZeroPageY = 12,
}

const ADDRESSING_MODE_TABLE: [fn(&mut Cpu, &mut bool) -> u16; 13] = [
    |cpu: &mut Cpu, _: &mut bool| { cpu.pop_word() },
    |cpu: &mut Cpu, page_crossing: &mut bool| {
        let addr = cpu.pop_word();
        let ret = addr + cpu.x as u16;
        if addr & 0xFF00 != ret & 0xFF00 {
            *page_crossing = true;
        }
        ret
    },
    |cpu: &mut Cpu, page_crossing: &mut bool| {
        let addr = cpu.pop_word();
        let ret = addr + cpu.y as u16;
        if addr & 0xFF00 != ret & 0xFF00 {
            *page_crossing = true;
        }
        ret
    },
    |_: &mut Cpu, _: &mut bool| { panic!("No address associated with accumulator mode.") },
    |cpu: &mut Cpu, _: &mut bool| {
        cpu.pc += 1;
        cpu.pc
    },
    |_: &mut Cpu, _: &mut bool| { panic!("No address associated with implied mode.") },
    // TODO(jeffreyxiao): Handle errata with JMP
    |cpu: &mut Cpu, _: &mut bool| { cpu.pop_word() },
    |cpu: &mut Cpu, _: &mut bool| {
        let addr = cpu.pop_byte().wrapping_add(cpu.x) as u16;
        cpu.read_word(addr)
    },
    |cpu: &mut Cpu, page_crossing: &mut bool| {
        let addr = cpu.pop_byte() as u16;
        let ret = cpu.read_word(addr).wrapping_add(cpu.y as u16);
        if addr & 0xFF00 != ret & 0xFF00 {
            *page_crossing = true;
        }
        ret
    },
    |cpu: &mut Cpu, _: &mut bool| { (cpu.pop_byte() as i32 + cpu.pc as i32) as u16 },
    |cpu: &mut Cpu, _: &mut bool| { cpu.pop_byte() as u16 },
    |cpu: &mut Cpu, _: &mut bool| { cpu.pop_byte().wrapping_add(cpu.x) as u16 },
    |cpu: &mut Cpu, _: &mut bool| { cpu.pop_byte().wrapping_add(cpu.y) as u16 },
];

