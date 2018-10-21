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
    |_: &mut Cpu| panic!("[CPU] Invalid addressing mode."),
    // absolute
    |cpu: &mut Cpu| (cpu.decode_word(), false),
    // absolute x
    |cpu: &mut Cpu| {
        let addr = cpu.decode_word();
        let ret = addr.wrapping_add(cpu.r.x as u16);
        (ret, addr & 0xFF00 != ret & 0xFF00)
    },
    // absolute y
    |cpu: &mut Cpu| {
        let addr = cpu.decode_word();
        let ret = addr.wrapping_add(cpu.r.y as u16);
        (ret, addr & 0xFF00 != ret & 0xFF00)
    },
    // accumulator
    |_: &mut Cpu| panic!("[CPU] No address associated with accumulator mode."),
    // immediate
    |cpu: &mut Cpu| {
        let ret = cpu.r.pc;
        cpu.r.pc += 1;
        (ret, false)
    },
    // implied
    |_: &mut Cpu| panic!("[CPU] No address associated with implied mode."),
    // indirect
    |cpu: &mut Cpu| {
        let addr = cpu.decode_word();
        if addr & 0xFF == 0xFF {
            let hi = (cpu.read_byte(addr & 0xFF00) as u16) << 8;
            let lo = cpu.read_byte(addr) as u16;
            (hi | lo, false)
        } else {
            (cpu.read_word(addr), false)
        }
    },
    // indirect x
    |cpu: &mut Cpu| {
        let addr = (cpu.decode_byte()).wrapping_add(cpu.r.x) as u16;
        // read 2-byte address without carry
        let hi = (cpu.read_byte((addr + 1) & 0xFF) as u16) << 8;
        let lo = cpu.read_byte(addr) as u16;
        (hi | lo, false)
    },
    // indirect y
    |cpu: &mut Cpu| {
        let addr = cpu.decode_byte() as u16;
        // read 2-byte address without carry
        let hi = (cpu.read_byte((addr + 1) & 0xFF) as u16) << 8;
        let lo = cpu.read_byte(addr) as u16;
        let addr = hi | lo;

        let ret = addr.wrapping_add(cpu.r.y as u16);
        (ret, addr & 0xFF00 != ret & 0xFF00)
    },
    // relative
    |cpu: &mut Cpu| {
        (
            (cpu.r.pc as i16 + 1 + i16::from(cpu.decode_byte() as i8)) as u16,
            false,
        )
    },
    // zero page
    |cpu: &mut Cpu| (cpu.decode_byte() as u16, false),
    // zero page x
    |cpu: &mut Cpu| (cpu.decode_byte().wrapping_add(cpu.r.x) as u16, false),
    // zero page y
    |cpu: &mut Cpu| (cpu.decode_byte().wrapping_add(cpu.r.y) as u16, false),
];
