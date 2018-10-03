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
        match opcode {
            _ => panic!(format!("Opcode not implemented {:2x}", opcode)),
        }
    }
}

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

