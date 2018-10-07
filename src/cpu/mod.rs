use memory::Memory;
use std::cell::RefCell;
use std::rc::Rc;

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
                    println!("A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:3}", $cpu.a, $cpu.x, $cpu.y, $cpu.p, $cpu.sp, $cpu.cycle);
                    $cpu.cycle = ($cpu.cycle + $cycles * 3) % 341;
                    $cpu.$instruction_fn(AddressingMode::$addressing_mode);
                }
            )*)*
            _ => panic!("No matching instruction for: {:02x}", $opcode)
        }
    }
}

const STATUS_CARRY_MASK: u8 = 1 << 0;
const STATUS_ZERO_MASK: u8 = 1 << 1;
const STATUS_INTERRUPT_DISABLE_MASK: u8 = 1 << 2;
const STATUS_DECIMAL_MODE_MASK: u8 = 1 << 3;
const STATUS_BREAK_COMMAND_MASK: u8 = 1 << 4;
const STATUS_OVERFLOW_MASK: u8 = 1 << 6;
const STATUS_NEGATIVE_MASK: u8 = 1 << 7;

const STACK_START: u16 = 0x100;

pub struct Cpu {
    pub cycle: u32,
    pub pc: u16,
    pub sp: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
    pub interrupt_flags: [bool; 3],
    pub memory: Rc<RefCell<Memory>>,
}

impl Cpu {
    pub fn new(memory: Rc<RefCell<Memory>>) -> Self {
        Cpu {
            cycle: 0,
            // pc: 0,
            pc: 0xC000,
            sp: 0xFD,
            a: 0,
            x: 0,
            y: 0,
            p: 0x24,
            // p: 0x34,
            interrupt_flags: [false; 3],
            memory,
        }
    }

    pub fn execute_cycle(&mut self) {
        // handle any interrupts
        for index in 0..self.interrupt_flags.len() {
            self.handle_interrupt(index);
        }

        print!("{:04X} ", self.pc);
        let opcode = self.decode_byte();
        print!("{:02X} ", opcode);
        self.execute_opcode(opcode);
    }

    pub fn trigger_interrupt(&mut self, interrupt: Interrupt) {
        if !self.get_status_flag(STATUS_INTERRUPT_DISABLE_MASK) || interrupt == Interrupt::NMI {
            self.interrupt_flags[interrupt as usize] = true;
        }
    }

    pub fn handle_interrupt(&mut self, interrupt: usize) {
        if self.interrupt_flags[interrupt] {
            let val = self.pc;
            self.push_word(val);
            let val = self.p & 0x10;
            self.push_byte(val);
            self.set_status_flag(STATUS_INTERRUPT_DISABLE_MASK, true);
            self.pc = self.memory.borrow().read_word(interrupt_handlers[interrupt]);
            self.interrupt_flags[interrupt] = false;
        }
    }
}

struct Operand {
    val: u8,
    addr: Option<u16>,
    page_crossing: bool,
}

impl Cpu {
    // pc related instructions
    fn decode_byte(&mut self) -> u8 {
        let ret = self.memory.borrow().read_byte(self.pc);
        self.pc += 1;
        ret
    }

    fn decode_word(&mut self) -> u16 {
        let ret = self.memory.borrow().read_word(self.pc);
        self.pc += 2;
        ret
    }

    // stack related instructions
    fn push_byte(&mut self, val: u8) {
        self.memory
            .borrow_mut()
            .write_byte(self.sp as u16 + STACK_START, val);
        self.sp -= 1;
    }

    fn push_word(&mut self, word: u16) {
        self.push_byte((word >> 8) as u8);
        self.push_byte((word & 0xFF) as u8);
    }

    fn pop_byte(&mut self) -> u8 {
        self.sp += 1;
        self.memory.borrow().read_byte(self.sp as u16 + STACK_START)
    }

    fn pop_word(&mut self) -> u16 {
        (self.pop_byte() as u16) | ((self.pop_byte() as u16) << 8)
    }

    // status flag related instructions
    fn set_status_flag(&mut self, mask: u8, set: bool) {
        if set {
            self.p |= mask;
        } else {
            self.p &= !mask;
        }
    }

    fn get_status_flag(&mut self, mask: u8) -> bool {
        self.p & mask != 0
    }

    fn update_negative_flag(&mut self, val: u8) {
        self.set_status_flag(STATUS_NEGATIVE_MASK, val & 0x80 != 0);
    }

    fn update_zero_flag(&mut self, val: u8) {
        self.set_status_flag(STATUS_ZERO_MASK, val == 0);
    }

    fn execute_opcode(&mut self, opcode: u8) {
        generate_instructions!(self, opcode, {
            aax: (
                (0x87, ZeroPage, 3),  // unofficial
                (0x97, ZeroPageY, 4), // unofficial
                (0x83, IndirectX, 6), // unofficial
                (0x8F, Absolute, 4),  // unofficial
            ),
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
            dcp: (
                (0xC7, ZeroPage, 5),  // unofficial
                (0xD7, ZeroPageX, 6), // unofficial
                (0xCF, Absolute, 6),  // unofficial
                (0xDF, AbsoluteX, 7), // unofficial
                (0xDB, AbsoluteY, 7), // unofficial
                (0xC3, IndirectX, 8), // unofficial
                (0xD3, IndirectY, 8), // unofficial
            ),
            dec: (
                (0xC6, ZeroPage, 5),
                (0xD6, ZeroPageX, 6),
                (0xCE, Absolute, 6),
                (0xDE, AbsoluteX, 7),
            ),
            dex: ((0xCA, Implied, 2)),
            dey: ((0x88, Implied, 2)),
            dop: (
                (0x04, ZeroPage, 3),  // unofficial
                (0x14, ZeroPageX, 4), // unofficial
                (0x34, ZeroPageX, 4), // unofficial
                (0x44, ZeroPage, 3),  // unofficial
                (0x54, ZeroPageX, 4), // unofficial
                (0x64, ZeroPage, 3),  // unofficial
                (0x74, ZeroPageX, 4), // unofficial
                (0x80, Immediate, 2), // unofficial
                (0x82, Immediate, 2), // unofficial
                (0x89, Immediate, 2), // unofficial
                (0xC2, Immediate, 2), // unofficial
                (0xD4, ZeroPageX, 4), // unofficial
                (0xE2, Immediate, 2), // unofficial
                (0xF4, ZeroPageX, 4), // unofficial
            ),
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
            isc: (
                (0xE7, ZeroPage, 5),
                (0xF7, ZeroPageX, 6),
                (0xEF, Absolute, 6),
                (0xFF, AbsoluteX, 7),
                (0xFB, AbsoluteY, 7),
                (0xE3, IndirectX, 8),
                (0xF3, IndirectY, 8),
            ),
            jmp: (
                (0x4C, Absolute, 3),
                (0x6C, Indirect, 5),
            ),
            jsr: ((0x20, Absolute, 6)),
            lax: (
                (0xA7, ZeroPage, 3),  // unofficial
                (0xB7, ZeroPageY, 4), // unofficial
                (0xAF, Absolute, 4),  // unofficial
                (0xBF, AbsoluteY, 4), // unofficial
                (0xA3, IndirectX, 6), // unofficial
                (0xB3, IndirectY, 5), // unofficial
            ),
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
            nop: (
                (0x1A, Implied, 2), // unofficial
                (0x3A, Implied, 2), // unofficial
                (0x5A, Implied, 2), // unofficial
                (0x7A, Implied, 2), // unofficial
                (0xDA, Implied, 2), // unofficial
                (0xEA, Implied, 2),
                (0xFA, Implied, 2), // unofficial
            ),
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
            rla: (
                (0x27, ZeroPage, 5),
                (0x37, ZeroPageX, 6),
                (0x2F, Absolute, 6),
                (0x3F, AbsoluteX, 7),
                (0x3B, AbsoluteY, 7),
                (0x23, IndirectX, 8),
                (0x33, IndirectY, 8),
            ),
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
            rra: (
                (0x67, ZeroPage, 5),
                (0x77, ZeroPageX, 6),
                (0x6F, Absolute, 6),
                (0x7F, AbsoluteX, 7),
                (0x7B, AbsoluteY, 7),
                (0x63, IndirectX, 8),
                (0x73, IndirectY, 8),
            ),
            sbc: (
                (0xE1, IndirectX, 6),
                (0xE5, ZeroPage, 3),
                (0xE9, Immediate, 2),
                (0xEB, Immediate, 2), // unofficial
                (0xED, Absolute, 4),
                (0xF1, IndirectY, 5),
                (0xF5, ZeroPageX, 4),
                (0xF9, AbsoluteY, 4),
                (0xFD, AbsoluteX, 4),
            ),
            sec: ((0x38, Implied, 2)),
            sed: ((0xF8, Implied, 2)),
            sei: ((0x78, Implied, 2)),
            slo: (
                (0x07, ZeroPage, 5),
                (0x17, ZeroPageX, 6),
                (0x0F, Absolute, 6),
                (0x1F, AbsoluteX, 7),
                (0x1B, AbsoluteY, 7),
                (0x03, IndirectX, 8),
                (0x13, IndirectY, 8),
            ),
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
            sre: (
                (0x47, ZeroPage, 5),
                (0x57, ZeroPageX, 6),
                (0x4F, Absolute, 6),
                (0x5F, AbsoluteX, 7),
                (0x5B, AbsoluteY, 7),
                (0x43, IndirectX, 8),
                (0x53, IndirectY, 8),
            ),
            tax: ((0xAA, Implied, 2)),
            tay: ((0xA8, Implied, 2)),
            top: (
                (0x0C, Absolute, 4),  // unofficial
                (0x1C, AbsoluteX, 4), // unofficial
                (0x3C, AbsoluteX, 4), // unofficial
                (0x5C, AbsoluteX, 4), // unofficial
                (0x7C, AbsoluteX, 4), // unofficial
                (0xDC, AbsoluteX, 4), // unofficial
                (0xFC, AbsoluteX, 4), // unofficial
            ),
            tsx: ((0xBA, Implied, 2)),
            txa: ((0x8A, Implied, 2)),
            txs: ((0x9A, Implied, 2)),
            tya: ((0x98, Implied, 2)),
        })
    }

    fn get_operand(&mut self, addressing_mode: AddressingMode) -> Operand {
        match addressing_mode {
            AddressingMode::Accumulator => {
                Operand {
                    val: self.a,
                    addr: None,
                    page_crossing: false,
                }
            },
            _ => {
                let (addr, page_crossing) = ADDRESSING_MODE_TABLE[addressing_mode as usize](self);
                Operand {
                    val: self.memory.borrow().read_byte(addr),
                    addr: Some(addr),
                    page_crossing,
                }
            },
        }
    }

    fn write_operand(&mut self, operand: &mut Operand) {
        match operand.addr {
            Some(addr) => self.memory.borrow_mut().write_byte(addr, operand.val),
            None => self.a = operand.val,
        }
    }

    fn aax(&mut self, addressing_mode: AddressingMode) {
        let (addr, _page_break) = ADDRESSING_MODE_TABLE[addressing_mode as usize](self);

        let res = self.x & self.a;
        // self.update_negative_flag(res);
        // self.update_zero_flag(res);
        self.memory.borrow_mut().write_byte(addr, res);
    }

    fn adc_impl(&mut self, operand: &Operand) {
        let carry = if self.p & STATUS_CARRY_MASK == 0 {
            0
        } else {
            1
        };
        let (res, is_overflow_1) = self.a.overflowing_add(operand.val);
        let (res, is_overflow_2) = res.overflowing_add(carry);
        let overflow = !(operand.val ^ self.a) & (res ^ self.a) & 0x80 != 0;
        self.update_negative_flag(res);
        self.update_zero_flag(res);
        self.set_status_flag(STATUS_CARRY_MASK, is_overflow_1 | is_overflow_2);
        self.set_status_flag(STATUS_OVERFLOW_MASK, overflow);
        self.a = res;
    }

    fn adc(&mut self, addressing_mode: AddressingMode) {
        let operand = self.get_operand(addressing_mode);
        if operand.page_crossing {
            self.cycle = (self.cycle + 1 * 3) % 341;
        }

        self.adc_impl(&operand);
    }

    fn and_impl(&mut self, operand: &Operand) {
        self.a &= operand.val;
        let res = self.a;
        self.update_negative_flag(res);
        self.update_zero_flag(res);
    }

    fn and(&mut self, addressing_mode: AddressingMode) {
        let operand = self.get_operand(addressing_mode);
        if operand.page_crossing {
            self.cycle = (self.cycle + 1 * 3) % 341;
        }

        self.and_impl(&operand);
    }

    fn asl_impl(&mut self, operand: &mut Operand) {
        let res = operand.val << 1;
        self.update_negative_flag(res);
        self.update_zero_flag(res);
        self.set_status_flag(STATUS_CARRY_MASK, operand.val & 0x80 != 0);

        operand.val = res;
        self.write_operand(operand);
    }

    fn asl(&mut self, addressing_mode: AddressingMode) {
        let mut operand = self.get_operand(addressing_mode);
        if operand.page_crossing {
            self.cycle = (self.cycle + 1 * 3) % 341;
        }

        self.asl_impl(&mut operand);
    }

    fn branch_impl(&mut self, cond: bool, addressing_mode: AddressingMode) {
        let (addr, _page_break) = ADDRESSING_MODE_TABLE[addressing_mode as usize](self);
        if cond {
            self.cycle = (self.cycle + 1 * 3) % 341;
            if self.pc & 0xFF00 != addr & 0xFF00 {
                self.cycle = (self.cycle + 1 * 3) % 341;
            }
            self.pc = addr;
        }
    }

    fn bcc(&mut self, addressing_mode: AddressingMode) {
        let cond = !self.get_status_flag(STATUS_CARRY_MASK);
        self.branch_impl(cond, addressing_mode);
    }

    fn bcs(&mut self, addressing_mode: AddressingMode) {
        let cond = self.get_status_flag(STATUS_CARRY_MASK);
        self.branch_impl(cond, addressing_mode);
    }

    fn beq(&mut self, addressing_mode: AddressingMode) {
        let cond = self.get_status_flag(STATUS_ZERO_MASK);
        self.branch_impl(cond, addressing_mode);
    }

    fn bit(&mut self, addressing_mode: AddressingMode) {
        let operand = self.get_operand(addressing_mode);
        self.set_status_flag(
            STATUS_NEGATIVE_MASK,
            operand.val & STATUS_NEGATIVE_MASK != 0,
        );
        self.set_status_flag(
            STATUS_OVERFLOW_MASK,
            operand.val & STATUS_OVERFLOW_MASK != 0,
        );

        let res = operand.val & self.a;
        self.update_zero_flag(res);
    }

    fn bmi(&mut self, addressing_mode: AddressingMode) {
        let cond = self.get_status_flag(STATUS_NEGATIVE_MASK);
        self.branch_impl(cond, addressing_mode);
    }

    fn bne(&mut self, addressing_mode: AddressingMode) {
        let cond = !self.get_status_flag(STATUS_ZERO_MASK);
        self.branch_impl(cond, addressing_mode);
    }

    fn bpl(&mut self, addressing_mode: AddressingMode) {
        let cond = !self.get_status_flag(STATUS_NEGATIVE_MASK);
        self.branch_impl(cond, addressing_mode);
    }

    fn brk(&mut self, _addressing_mode: AddressingMode) {
        self.interrupt_flags[Interrupt::IRQ as usize] = true;
        self.handle_interrupt(Interrupt::IRQ as usize);
    }

    fn bvc(&mut self, addressing_mode: AddressingMode) {
        let cond = !self.get_status_flag(STATUS_OVERFLOW_MASK);
        self.branch_impl(cond, addressing_mode);
    }

    fn bvs(&mut self, addressing_mode: AddressingMode) {
        let cond = self.get_status_flag(STATUS_OVERFLOW_MASK);
        self.branch_impl(cond, addressing_mode);
    }

    fn clc(&mut self, _addressing_mode: AddressingMode) {
        self.set_status_flag(STATUS_CARRY_MASK, false);
    }

    fn cld(&mut self, _addressing_mode: AddressingMode) {
        self.set_status_flag(STATUS_DECIMAL_MODE_MASK, false);
    }

    fn cli(&mut self, _addressing_mode: AddressingMode) {
        self.set_status_flag(STATUS_INTERRUPT_DISABLE_MASK, false);
    }

    fn clv(&mut self, _addressing_mode: AddressingMode) {
        self.set_status_flag(STATUS_OVERFLOW_MASK, false);
    }

    fn cmp_impl(&mut self, operand: &Operand) {
        let (diff, underflow) = self.a.overflowing_sub(operand.val);
        self.set_status_flag(STATUS_CARRY_MASK, !underflow);
        self.update_zero_flag(diff);
        self.update_negative_flag(diff);
    }

    fn cmp(&mut self, addressing_mode: AddressingMode) {
        let operand = self.get_operand(addressing_mode);
        if operand.page_crossing {
            self.cycle = (self.cycle + 1 * 3) % 341;
        }

        self.cmp_impl(&operand);
    }

    fn cpx(&mut self, addressing_mode: AddressingMode) {
        let operand = self.get_operand(addressing_mode);
        let (diff, underflow) = self.x.overflowing_sub(operand.val);
        self.set_status_flag(STATUS_CARRY_MASK, !underflow);
        self.update_zero_flag(diff);
        self.update_negative_flag(diff);
    }

    fn cpy(&mut self, addressing_mode: AddressingMode) {
        let operand = self.get_operand(addressing_mode);
        let (diff, underflow) = self.y.overflowing_sub(operand.val);
        self.set_status_flag(STATUS_CARRY_MASK, !underflow);
        self.update_zero_flag(diff);
        self.update_negative_flag(diff);
    }

    fn dcp(&mut self, addressing_mode: AddressingMode) {
        let mut operand = self.get_operand(addressing_mode);

        self.dec_impl(&mut operand);
        self.cmp_impl(&mut operand);
    }

    fn dec_impl(&mut self, operand: &mut Operand) {
        let res = operand.val.wrapping_sub(1);
        self.update_zero_flag(res);
        self.update_negative_flag(res);

        operand.val = res;
        self.write_operand(operand);
    }

    fn dec(&mut self, addressing_mode: AddressingMode) {
        let mut operand = self.get_operand(addressing_mode);

        self.dec_impl(&mut operand);
    }

    fn dex(&mut self, _addressing_mode: AddressingMode) {
        let res = self.x.wrapping_sub(1);
        self.update_zero_flag(res);
        self.update_negative_flag(res);
        self.x = res;
    }

    fn dey(&mut self, _addressing_mode: AddressingMode) {
        let res = self.y.wrapping_sub(1);
        self.update_zero_flag(res);
        self.update_negative_flag(res);
        self.y = res;
    }

    fn dop(&mut self, addressing_mode: AddressingMode) {
        ADDRESSING_MODE_TABLE[addressing_mode as usize](self);
    }

    fn eor_impl(&mut self, operand: &Operand) {
        self.a ^= operand.val;
        let res = self.a;
        self.update_negative_flag(res);
        self.update_zero_flag(res);
    }

    fn eor(&mut self, addressing_mode: AddressingMode) {
        let operand = self.get_operand(addressing_mode);
        if operand.page_crossing {
            self.cycle = (self.cycle + 1 * 3) % 341;
        }

        self.eor_impl(&operand);
    }

    fn inc_impl(&mut self, operand: &mut Operand) {
        let res = operand.val.wrapping_add(1);
        self.update_zero_flag(res);
        self.update_negative_flag(res);

        operand.val = res;
        self.write_operand(operand);
    }

    fn inc(&mut self, addressing_mode: AddressingMode) {
        let mut operand = self.get_operand(addressing_mode);

        self.inc_impl(&mut operand);
    }

    fn inx(&mut self, _addressing_mode: AddressingMode) {
        let res = self.x.wrapping_add(1);
        self.update_zero_flag(res);
        self.update_negative_flag(res);
        self.x = res;
    }

    fn iny(&mut self, _addressing_mode: AddressingMode) {
        let res = self.y.wrapping_add(1);
        self.update_zero_flag(res);
        self.update_negative_flag(res);
        self.y = res;
    }

    fn isc(&mut self, addressing_mode: AddressingMode) {
        let mut operand = self.get_operand(addressing_mode);

        self.inc_impl(&mut operand);
        self.sbc_impl(&operand);
    }

    fn jmp(&mut self, addressing_mode: AddressingMode) {
        let (addr, _page_break) = ADDRESSING_MODE_TABLE[addressing_mode as usize](self);
        self.pc = addr;
    }

    fn jsr(&mut self, addressing_mode: AddressingMode) {
        let (addr, _page_break) = ADDRESSING_MODE_TABLE[addressing_mode as usize](self);
        let ret = self.pc - 1;
        self.pc = addr;
        self.push_word(ret);
    }

    fn lax(&mut self, addressing_mode: AddressingMode) {
        let operand = self.get_operand(addressing_mode);
        if operand.page_crossing {
            self.cycle = (self.cycle + 1 * 3) % 341;
        }

        self.lda_impl(&operand);
        self.ldx_impl(&operand);
    }

    fn lda_impl(&mut self, operand: &Operand) {
        self.a = operand.val;
        self.update_zero_flag(operand.val);
        self.update_negative_flag(operand.val);
    }

    fn lda(&mut self, addressing_mode: AddressingMode) {
        let operand = self.get_operand(addressing_mode);
        if operand.page_crossing {
            self.cycle = (self.cycle + 1 * 3) % 341;
        }

        self.lda_impl(&operand);
    }

    fn ldx_impl(&mut self, operand: &Operand) {
        self.x = operand.val;
        self.update_zero_flag(operand.val);
        self.update_negative_flag(operand.val);
    }

    fn ldx(&mut self, addressing_mode: AddressingMode) {
        let operand = self.get_operand(addressing_mode);
        if operand.page_crossing {
            self.cycle = (self.cycle + 1 * 3) % 341;
        }

        self.ldx_impl(&operand);
    }

    fn ldy(&mut self, addressing_mode: AddressingMode) {
        let operand = self.get_operand(addressing_mode);
        if operand.page_crossing {
            self.cycle = (self.cycle + 1 * 3) % 341;
        }

        self.y = operand.val;
        self.update_zero_flag(operand.val);
        self.update_negative_flag(operand.val);
    }

    fn lsr_impl(&mut self, operand: &mut Operand) {
        let res = operand.val >> 1;
        self.update_negative_flag(res);
        self.update_zero_flag(res);
        self.set_status_flag(STATUS_CARRY_MASK, operand.val & 0x01 != 0);

        operand.val = res;
        self.write_operand(operand);
    }

    fn lsr(&mut self, addressing_mode: AddressingMode) {
        let mut operand = self.get_operand(addressing_mode);
        if operand.page_crossing {
            self.cycle = (self.cycle + 1 * 3) % 341;
        }

        self.lsr_impl(&mut operand);
    }

    fn nop(&mut self, _addressing_mode: AddressingMode) {}

    fn ora_impl(&mut self, operand: &Operand) {
        self.a |= operand.val;
        let res = self.a;
        self.update_negative_flag(res);
        self.update_zero_flag(res);
    }

    fn ora(&mut self, addressing_mode: AddressingMode) {
        let operand = self.get_operand(addressing_mode);
        if operand.page_crossing {
            self.cycle = (self.cycle + 1 * 3) % 341;
        }

        self.ora_impl(&operand);
    }

    fn pha(&mut self, _addressing_mode: AddressingMode) {
        let res = self.a;
        self.push_byte(res);
    }

    fn php(&mut self, _addressing_mode: AddressingMode) {
        let res = self.p | 0x10;
        self.push_byte(res);
    }

    fn pla(&mut self, _addressing_mode: AddressingMode) {
        let res = self.pop_byte();
        self.a = res;
        self.update_negative_flag(res);
        self.update_zero_flag(res);
    }

    fn plp(&mut self, _addressing_mode: AddressingMode) {
        let res = (self.pop_byte() & !0x30) | (self.p & 0x30);
        self.p = res;
    }

    fn rla(&mut self, addressing_mode: AddressingMode) {
        let mut operand = self.get_operand(addressing_mode);

        self.rol_impl(&mut operand);
        self.and_impl(&operand);
    }

    fn rol_impl(&mut self, operand: &mut Operand) {
        let mut res = operand.val << 1;
        res |= if self.get_status_flag(STATUS_CARRY_MASK) {
            1
        } else {
            0
        };
        self.update_negative_flag(res);
        self.update_zero_flag(res);
        self.set_status_flag(STATUS_CARRY_MASK, operand.val & 0x80 != 0);

        operand.val = res;
        self.write_operand(operand);
    }

    fn rol(&mut self, addressing_mode: AddressingMode) {
        let mut operand = self.get_operand(addressing_mode);

        self.rol_impl(&mut operand);
    }

    fn ror_impl(&mut self, operand: &mut Operand) {
        let mut res = operand.val >> 1;
        res |= if self.get_status_flag(STATUS_CARRY_MASK) {
            0x80
        } else {
            0
        };
        self.update_negative_flag(res);
        self.update_zero_flag(res);
        self.set_status_flag(STATUS_CARRY_MASK, operand.val & 0x01 != 0);

        operand.val = res;
        self.write_operand(operand);
    }

    fn ror(&mut self, addressing_mode: AddressingMode) {
        let mut operand = self.get_operand(addressing_mode);

        self.ror_impl(&mut operand);
    }

    fn rti(&mut self, _addressing_mode: AddressingMode) {
        self.plp(AddressingMode::Implied);
        self.pc = self.pop_word();
    }

    fn rts(&mut self, _addressing_mode: AddressingMode) {
        self.pc = self.pop_word() + 1;
    }

    fn rra(&mut self, addressing_mode: AddressingMode) {
        let mut operand = self.get_operand(addressing_mode);

        self.ror_impl(&mut operand);
        self.adc_impl(&operand);
    }

    fn sbc_impl(&mut self, operand: &Operand) {
        let carry = if self.p & STATUS_CARRY_MASK == 0 {
            1
        } else {
            0
        };
        let (res, is_underflow_1) = self.a.overflowing_sub(operand.val);
        let (res, is_underflow_2) = res.overflowing_sub(carry);
        let underflow = (operand.val ^ self.a) & (res ^ self.a) & 0x80 != 0;
        self.update_negative_flag(res);
        self.update_zero_flag(res);
        self.set_status_flag(STATUS_CARRY_MASK, !is_underflow_1 && !is_underflow_2);
        self.set_status_flag(STATUS_OVERFLOW_MASK, underflow);
        self.a = res;
    }

    fn sbc(&mut self, addressing_mode: AddressingMode) {
        let operand = self.get_operand(addressing_mode);
        if operand.page_crossing {
            self.cycle = (self.cycle + 1 * 3) % 341;
        }

        self.sbc_impl(&operand);
    }

    fn sec(&mut self, _addressing_mode: AddressingMode) {
        self.set_status_flag(STATUS_CARRY_MASK, true);
    }

    fn sed(&mut self, _addressing_mode: AddressingMode) {
        self.set_status_flag(STATUS_DECIMAL_MODE_MASK, true);
    }

    fn sei(&mut self, _addressing_mode: AddressingMode) {
        self.set_status_flag(STATUS_INTERRUPT_DISABLE_MASK, true);
    }

    fn slo(&mut self, addressing_mode: AddressingMode) {
        let mut operand = self.get_operand(addressing_mode);

        self.asl_impl(&mut operand);
        self.ora_impl(&operand);
    }

    fn sta(&mut self, addressing_mode: AddressingMode) {
        let (addr, _page_break) = ADDRESSING_MODE_TABLE[addressing_mode as usize](self);
        self.memory.borrow_mut().write_byte(addr, self.a);
    }

    fn stx(&mut self, addressing_mode: AddressingMode) {
        let (addr, _page_break) = ADDRESSING_MODE_TABLE[addressing_mode as usize](self);
        self.memory.borrow_mut().write_byte(addr, self.x);
    }

    fn sty(&mut self, addressing_mode: AddressingMode) {
        let (addr, _page_break) = ADDRESSING_MODE_TABLE[addressing_mode as usize](self);
        self.memory.borrow_mut().write_byte(addr, self.y);
    }

    fn sre(&mut self, addressing_mode: AddressingMode) {
        let mut operand = self.get_operand(addressing_mode);

        self.lsr_impl(&mut operand);
        self.eor_impl(&operand);
    }

    fn tax(&mut self, _addressing_mode: AddressingMode) {
        let res = self.a;
        self.update_negative_flag(res);
        self.update_zero_flag(res);
        self.x = res;
    }

    fn tay(&mut self, _addressing_mode: AddressingMode) {
        let res = self.a;
        self.update_negative_flag(res);
        self.update_zero_flag(res);
        self.y = res;
    }

    fn top(&mut self, addressing_mode: AddressingMode) {
        let (_, page_crossing) = ADDRESSING_MODE_TABLE[addressing_mode as usize](self);
        if page_crossing {
            self.cycle = (self.cycle + 1 * 3) % 341;
        }
    }

    fn tsx(&mut self, _addressing_mode: AddressingMode) {
        let res = self.sp;
        self.update_negative_flag(res);
        self.update_zero_flag(res);
        self.x = res;
    }

    fn txa(&mut self, _addressing_mode: AddressingMode) {
        let res = self.x;
        self.update_negative_flag(res);
        self.update_zero_flag(res);
        self.a = res;
    }

    fn txs(&mut self, _addressing_mode: AddressingMode) {
        let res = self.x;
        self.sp = res;
    }

    fn tya(&mut self, _addressing_mode: AddressingMode) {
        let res = self.y;
        self.update_negative_flag(res);
        self.update_zero_flag(res);
        self.a = res;
    }
}

#[derive(PartialEq)]
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

const ADDRESSING_MODE_TABLE: [fn(&mut Cpu) -> (u16, bool); 13] = [
    |cpu: &mut Cpu| (cpu.decode_word(), false),
    |cpu: &mut Cpu| {
        let addr = cpu.decode_word();
        let ret = addr + cpu.x as u16;
        let mut page_crossing = false;
        if addr & 0xFF00 != ret & 0xFF00 {
            page_crossing = true;
        }
        (ret, page_crossing)
    },
    |cpu: &mut Cpu| {
        let addr = cpu.decode_word();
        let ret = addr.wrapping_add(cpu.y as u16);
        let mut page_crossing = false;
        if addr & 0xFF00 != ret & 0xFF00 {
            page_crossing = true;
        }
        (ret, page_crossing)
    },
    |_: &mut Cpu| panic!("No address associated with accumulator mode."),
    |cpu: &mut Cpu| {
        let ret = cpu.pc;
        cpu.pc += 1;
        (ret, false)
    },
    |_: &mut Cpu| panic!("No address associated with implied mode."),
    |cpu: &mut Cpu| {
        let addr = cpu.decode_word();
        if addr & 0xFF == 0xFF {
            let hi = (cpu.memory.borrow().read_byte(addr & 0xFF00) as u16) << 8;
            let lo = cpu.memory.borrow().read_byte(addr) as u16;
            (hi | lo, false)
        } else {
            (cpu.memory.borrow().read_word(addr), false)
        }
    },
    |cpu: &mut Cpu| {
        let addr = (cpu.decode_byte()).wrapping_add(cpu.x) as u16;
        // read 2-byte address without carry
        let hi = (cpu.memory.borrow().read_byte((addr + 1) & 0xFF) as u16) << 8;
        let lo = cpu.memory.borrow().read_byte(addr) as u16;
        (hi | lo, false)
    },
    |cpu: &mut Cpu| {
        let addr = cpu.decode_byte() as u16;
        // read 2-byte address without carry
        let hi = (cpu.memory.borrow().read_byte((addr + 1) & 0xFF) as u16) << 8;
        let lo = cpu.memory.borrow().read_byte(addr) as u16;
        let addr = hi | lo;

        let ret = addr.wrapping_add(cpu.y as u16);
        let mut page_crossing = false;
        if addr & 0xFF00 != ret & 0xFF00 {
            page_crossing = true;
        }
        (ret, page_crossing)
    },
    |cpu: &mut Cpu| {
        (
            (cpu.pc as i16 + 1 + i16::from(cpu.decode_byte() as i8)) as u16,
            false,
        )
    },
    |cpu: &mut Cpu| (cpu.decode_byte() as u16, false),
    |cpu: &mut Cpu| (cpu.decode_byte().wrapping_add(cpu.x) as u16, false),
    |cpu: &mut Cpu| (cpu.decode_byte().wrapping_add(cpu.y) as u16, false),
];

#[derive(PartialEq)]
enum Interrupt {
    NMI = 0,
    IRQ = 1,
    RESET = 2,
}

const interrupt_handlers: [u16; 3] = [0xFFFA, 0xFFFE, 0xFFFC];
