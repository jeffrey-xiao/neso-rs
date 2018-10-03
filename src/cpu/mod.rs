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
    pub acc: u8,
    pub x: u8,
    pub y: u8,
    pub ps: u8,
}

struct CpuMemoryMap {

}

impl Cpu {
    fn pop_u8(&mut self) -> u8 { 0 }
    fn pop_u16(&mut self) -> u16 { 0 }
    fn read_u8(&self, addr: u16) -> u8 { 0 }
    fn read_u16(&self, addr: u16) -> u16 { 0 }

    fn execute_opcode(&mut self, opcode: u8) {
        generate_instructions!(self, opcode, {
            adc: (
                (0x61, IndirectX, 6),
                (0x65, ZeroPage, 3),
                (0x69, Immediate, 2),
                (0x6D, Absolute, 4),
                (0x71, IndirectY, 5/* * */),
                (0x75, ZeroPageX, 4),
                (0x79, AbsoluteY, 4/* * */),
                (0x7D, AbsoluteX, 4/* * */),
            ),
            and: (
                (0x29, immidiate, 2),
                (0x25, zeropage, 3),
                (0x35, zeropage,X, 4),
                (0x2D, absolute, 4),
                (0x3D, absolute,X, 4/* * */),
                (0x39, absolute,Y, 4/* * */),
                (0x21, (indirect,X), 6),
                (0x31, (indirect),Y, 5/* * */),
            ),
            asl: (
                (0x0A, accumulator, 2),
                (0x06, zeropage, 5),
                (0x16, zeropage,X, 6),
                (0x0E, absolute, 6),
                (0x1E, absolute,X, 7),
            ),
            bcc: ((0x90, relative, 2/* ** */)),
            bcs: ((0xB0, relative, 2/* ** */)),
			beq: ((0xF0, relative, 2/* ** */)),
            bit: (
                (0x24, zeropage, 3),
                (0x2C, absolute, 4),
            ),
            bmi: ((0x30, relative, 2/* ** */)),
            bne: ((0xD0, relative, 2/* ** */)),
            bpl: ((0x10, relative, 2/* ** */)),
            brk: ((0x00, implied1, 7)),
            bvc: ((0x50, relative, 2/* ** */)),
            bvs: ((0x70, relative, 2/* ** */)),
            clc: ((0x18, implied1, 2)),
            cld: ((0xD8, implied1, 2)),
            cli: ((0x58, implied1, 2)),
            clv: ((0xB8, implied1, 2)),
            cmp: (
                (0xC9, immidiate, 2),
                (0xC5, zeropage, 3),
                (0xD5, zeropage,X, 4),
                (0xCD, absolute, 4),
                (0xDD, absolute,X, 4/* * */),
                (0xD9, absolute,Y, 4/* * */),
                (0xC1, (indirect,X), 6),
                (0xD1, (indirect),Y, 5/* * */),
            ),
            cpx: (
                (0xE0, immidiate, 2),
                (0xE4, zeropage, 3),
                (0xEC, absolute, 4),
            ),
            cpy: (
                (0xC0, immidiate, 2),
                (0xC4, zeropage, 3),
                (0xCC, absolute, 4),
            ),
            dec: (
                (0xC6, zeropage, 5),
                (0xD6, zeropage,X, 6),
                (0xCE, absolute, 3),
                (0xDE, absolute,X, 7),
            ),
            dex: ((0xCA, implied, 2)),
            dey: ((0x88, implied, 2)),
            eor: (
                (0x49, immidiate, 2),
                (0x45, zeropage, 3),
                (0x55, zeropage,X, 4),
                (0x4D, absolute, 4),
                (0x5D, absolute,X, 4/* * */),
                (0x59, absolute,Y, 4/* * */),
                (0x41, (indirect,X), 6),
                (0x51, (indirect),Y, 5/* * */),
            ),
            inc: (
                (0xE6, zeropage, 5),
                (0xF6, zeropage,X, 6),
                (0xEE, absolute, 6),
                (0xFE, absolute,X, 7),
            ),
            inx: ((0xE8, implied, 2)),
            iny: ((0xC8, implied, 2)),
            jmp: (
                (0x4C, absolute, 3),
                (0x6C, indirect, 5),
            ),
            jsr: ((0x20, absolute, 6)),
            lda: (
                (0xA9, immidiate, 2),
                (0xA5, zeropage, 3),
                (0xB5, zeropage,X, 4),
                (0xAD, absolute, 4),
                (0xBD, absolute,X, 4/* * */),
                (0xB9, absolute,Y, 4/* * */),
                (0xA1, (indirect,X), 6),
                (0xB1, (indirect),Y, 5/* * */),
            ),
            ldx: (
                (0xA2, immidiate, 2),
                (0xA6, zeropage, 3),
                (0xB6, zeropage,Y, 4),
                (0xAE, absolute, 4),
                (0xBE, absolute,Y, 4/* * */),
            ),
            ldy: (
                (0xA0, immidiate, 2),
                (0xA4, zeropage, 3),
                (0xB4, zeropage,X, 4),
                (0xAC, absolute, 4),
                (0xBC, absolute,X, 4/* * */),
            ),
            lsr: (
                (0x4A, accumulator, 2),
                (0x46, zeropage, 5),
                (0x56, zeropage,X, 6),
                (0x4E, absolute, 6),
                (0x5E, absolute,X, 7),
            ),
            nop: ((0xEA, implied, 2)),
            ora: (
                (0x09, immidiate, 2),
                (0x05, zeropage, 3),
                (0x15, zeropage,X, 4),
                (0x0D, absolute, 4),
                (0x1D, absolute,X, 4/* * */),
                (0x19, absolute,Y, 4/* * */),
                (0x01, (indirect,X), 6),
                (0x11, (indirect),Y, 5/* * */),
            ),
            pha: ((0x48, implied, 3)),
            php: ((0x08, implied, 3)),
            pla: ((0x68, implied, 4)),
            plp: ((0x28, implied, 4)),
            rol: (
                (0x2A, accumulator, 2),
                (0x26, zeropage, 5),
                (0x36, zeropage,X, 6),
                (0x2E, absolute, 6),
                (0x3E, absolute,X, 7),
            ),
            ror: (
                (0x6A, accumulator, 2),
                (0x66, zeropage, 5),
                (0x76, zeropage,X, 6),
                (0x6E, absolute, 6),
                (0x7E, absolute,X, 7),
            ),
            rti: ((0x40, implied, 6)),
            rts: ((0x60, implied, 6)),
            sbc: (
                (0xE9, immidiate, 2),
                (0xE5, zeropage, 3),
                (0xF5, zeropage,X, 4),
                (0xED, absolute, 4),
                (0xFD, absolute,X, 4/* * */),
                (0xF9, absolute,Y, 4/* * */),
                (0xE1, (indirect,X), 6),
                (0xF1, (indirect),Y, 5/* * */),
            ),
            sec: ((0x38, implied, 2)),
            sed: ((0xF8, implied, 2)),
            sei: ((0x78, implied, 2)),
            sta: (
                (0x85, zeropage, 3),
                (0x95, zeropage,X, 4),
                (0x8D, absolute, 4),
                (0x9D, absolute,X, 5),
                (0x99, absolute,Y, 5),
                (0x81, (indirect,X), 6),
                (0x91, (indirect),Y, 6),
            ),
            stx: (
                (0x86, zeropage, 3),
                (0x96, zeropage,Y, 4),
                (0x8E, absolute, 4),
            ),
            sty: (
                (0x84, zeropage, 3),
                (0x94, zeropage,X, 4),
                (0x8C, absolute, 4),
            ),
            tax: ((0xAA, implied, 2)),
            tay: ((0xA8, implied, 2)),
            tsx: ((0xBA, implied, 2)),
            txa: ((0x8A, implied, 2)),
            txs: ((0x9A, implied, 2)),
            tya: ((0x98, implied, 2)),
        })
    }
}

fn adc(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
fn and(cpu: &mut Cpu, addressing_mode: AddressingMode) {}
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

const ADDRESSING_MODE_TABLE: [fn(&mut Cpu) -> u16; 13] = [
    |cpu: &mut Cpu| { cpu.pop_u16() },
    |cpu: &mut Cpu| {
        let ret = cpu.pop_u16();
        if ret & 0xFF00 != (ret + cpu.x as u16) & 0xFF00 {
            cpu.cycle += 1;
        }
        ret + cpu.x as u16
    },
    |cpu: &mut Cpu| {
        let ret = cpu.pop_u16();
        if ret & 0xFF00 != (ret + cpu.x as u16) & 0xFF00 {
            cpu.cycle += 1;
        }
        ret + cpu.y as u16
    },
    |_: &mut Cpu| { panic!("No address associated with accumulator mode.") },
    |cpu: &mut Cpu| {
        cpu.pc += 1;
        cpu.pc
    },
    |_: &mut Cpu| { panic!("No address associated with implied mode.") },
    |cpu: &mut Cpu| { cpu.pop_u16() },
    |cpu: &mut Cpu| {
        let addr = cpu.pop_u8().wrapping_add(cpu.x) as u16;
        cpu.read_u16(addr)
    },
    |cpu: &mut Cpu| {
        let addr = cpu.pop_u8() as u16;
        cpu.read_u16(addr).wrapping_add(cpu.y as u16)
    },
    |cpu: &mut Cpu| { (cpu.pop_u8() as i32 + cpu.pc as i32) as u16 },
    |cpu: &mut Cpu| { cpu.pop_u8() as u16 },
    |cpu: &mut Cpu| { cpu.pop_u8().wrapping_add(cpu.x) as u16 },
    |cpu: &mut Cpu| { cpu.pop_u8().wrapping_add(cpu.y) as u16 },
];

