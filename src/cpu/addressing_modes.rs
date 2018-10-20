use cpu::Cpu;

pub const ABSOLUTE: usize = 1;
pub const ABSOLUTE_X: usize = 2;
pub const ABSOLUTE_Y: usize = 3;
pub const ACCUMULATOR: usize = 4;
pub const IMMEDIATE: usize = 5;
pub const IMPLIED: usize = 6;
pub const INDIRECT: usize = 7;
pub const INDIRECT_X: usize = 8;
pub const INDIRECT_Y: usize = 9;
pub const RELATIVE: usize = 10;
pub const ZERO_PAGE: usize = 11;
pub const ZERO_PAGE_X: usize = 12;
pub const ZERO_PAGE_Y: usize = 13;

pub const FUNCTION_TABLE: [fn(&mut Cpu) -> (u16, bool); 14] = [
    |_: &mut Cpu| panic!("Invalid addressing mode."),
    |cpu: &mut Cpu| (cpu.decode_word(), false),
    |cpu: &mut Cpu| {
        let addr = cpu.decode_word();
        let ret = addr.wrapping_add(cpu.r.x as u16);
        (ret, addr & 0xFF00 != ret & 0xFF00)
    },
    |cpu: &mut Cpu| {
        let addr = cpu.decode_word();
        let ret = addr.wrapping_add(cpu.r.y as u16);
        (ret, addr & 0xFF00 != ret & 0xFF00)
    },
    |_: &mut Cpu| panic!("No address associated with accumulator mode."),
    |cpu: &mut Cpu| {
        let ret = cpu.r.pc;
        cpu.r.pc += 1;
        (ret, false)
    },
    |_: &mut Cpu| panic!("No address associated with implied mode."),
    |cpu: &mut Cpu| {
        let addr = cpu.decode_word();
        // println!("INDIRECT ADDR {}", addr);
        if addr & 0xFF == 0xFF {
            let hi = (cpu.read_byte(addr & 0xFF00) as u16) << 8;
            let lo = cpu.read_byte(addr) as u16;
            (hi | lo, false)
        } else {
            // println!("INDIRECTION IS {:x}", cpu.read_word(addr));
            (cpu.read_word(addr), false)
        }
    },
    |cpu: &mut Cpu| {
        let addr = (cpu.decode_byte()).wrapping_add(cpu.r.x) as u16;
        // read 2-byte address without carry
        let hi = (cpu.read_byte((addr + 1) & 0xFF) as u16) << 8;
        let lo = cpu.read_byte(addr) as u16;
        (hi | lo, false)
    },
    |cpu: &mut Cpu| {
        let addr = cpu.decode_byte() as u16;
        // read 2-byte address without carry
        let hi = (cpu.read_byte((addr + 1) & 0xFF) as u16) << 8;
        let lo = cpu.read_byte(addr) as u16;
        let addr = hi | lo;

        let ret = addr.wrapping_add(cpu.r.y as u16);
        (ret, addr & 0xFF00 != ret & 0xFF00)
    },
    |cpu: &mut Cpu| {
        (
            (cpu.r.pc as i16 + 1 + i16::from(cpu.decode_byte() as i8)) as u16,
            false,
        )
    },
    |cpu: &mut Cpu| (cpu.decode_byte() as u16, false),
    |cpu: &mut Cpu| (cpu.decode_byte().wrapping_add(cpu.r.x) as u16, false),
    |cpu: &mut Cpu| (cpu.decode_byte().wrapping_add(cpu.r.y) as u16, false),
];
