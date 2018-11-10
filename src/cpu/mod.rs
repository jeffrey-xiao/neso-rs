mod addressing_modes;
mod opcodes;
mod registers;

use self::registers::Registers;
use bus::Bus;
use controller::Controller;

const STACK_START: u16 = 0x100;

pub struct Cpu {
    pub cycle: u64,
    pub stall_cycle: u64,
    pub controllers: [Controller; 2],
    pub ram: [u8; 0x800],
    pub interrupt_flags: [bool; 2],
    pub r: Registers,
    pub bus: Option<Bus>,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            cycle: 0,
            stall_cycle: 0,
            controllers: [Controller::default(), Controller::default()],
            ram: [0; 0x800],
            interrupt_flags: [false; 2],
            r: Registers::default(),
            bus: None,
        }
    }

    pub fn initialize(&mut self) {
        self.r.pc = self.read_word(0xFFFC);
        self.r.sp = 0xFD;
        self.r.p = 0x24;
    }
    
    pub fn reset(&mut self) {
        self.r.pc = self.read_word(0xFFFC);
        self.r.sp -= 3;
        self.r.set_status_flag(registers::INTERRUPT_DISABLE_MASK, true);
        self.cycle = 0;
        self.stall_cycle = 0;
    }

    pub fn attach_bus(&mut self, bus: Bus) {
        self.bus = Some(bus);
        self.initialize()
    }

    fn bus(&self) -> &Bus {
        self.bus.as_ref().expect("[CPU] No bus attached.")
    }

    fn bus_mut(&mut self) -> &mut Bus {
        self.bus.as_mut().expect("[CPU] No bus attached.")
    }

    pub fn step(&mut self) {
        if self.stall_cycle > 0 {
            self.stall_cycle -= 1;
            return;
        }

        let start_cycle = self.cycle;
        // handle any interrupts
        for index in 0..self.interrupt_flags.len() {
            if self.interrupt_flags[index] {
                self.handle_interrupt(index);
                return;
            }
        }

        // print!("{:04X} ", self.r.pc);
        let opcode = self.decode_byte();
        // print!("{:02X} ", opcode);
        self.execute_opcode(opcode);
        self.stall_cycle += (self.cycle - start_cycle) - 1;
    }

    pub fn trigger_interrupt(&mut self, interrupt: Interrupt) {
        let is_disabled = self.r.get_status_flag(registers::INTERRUPT_DISABLE_MASK);
        if !is_disabled || interrupt == Interrupt::NMI {
            // println!("[CPU] Interrupt triggered: {:?}.", interrupt);
            self.interrupt_flags[interrupt as usize] = true;
        }
    }

    pub fn handle_interrupt(&mut self, interrupt: usize) {
        let val = self.r.pc;
        self.push_word(val);
        let val = self.r.p | 0x10;
        self.push_byte(val);
        self.r
            .set_status_flag(registers::INTERRUPT_DISABLE_MASK, true);
        self.r.pc = self.read_word(INTERRUPT_HANDLERS[interrupt]);
        self.interrupt_flags[interrupt] = false;
    }

    // pc related functions
    fn decode_byte(&mut self) -> u8 {
        let pc = self.r.pc;
        let ret = self.read_byte(pc);
        self.r.pc += 1;
        ret
    }

    fn decode_word(&mut self) -> u16 {
        let pc = self.r.pc;
        let ret = self.read_word(pc);
        self.r.pc += 2;
        ret
    }

    // stack related functions
    fn push_byte(&mut self, val: u8) {
        let addr = u16::from(self.r.sp) + STACK_START;
        self.write_byte(addr, val);
        self.r.sp = self.r.sp.wrapping_sub(1);
    }

    fn push_word(&mut self, word: u16) {
        self.push_byte((word >> 8) as u8);
        self.push_byte((word & 0xFF) as u8);
    }

    fn pop_byte(&mut self) -> u8 {
        self.r.sp = self.r.sp.wrapping_add(1);
        let addr = u16::from(self.r.sp) + STACK_START;
        self.read_byte(addr)
    }

    fn pop_word(&mut self) -> u16 {
        u16::from(self.pop_byte()) | (u16::from(self.pop_byte()) << 8)
    }

    // memory map related functions
    pub fn read_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr % 0x0800) as usize],
            0x2000..=0x3FFF => {
                let ppu = self.bus_mut().ppu_mut();
                let addr = (addr - 0x2000) % 8 + 0x2000;
                ppu.read_register(addr)
            },
            0x4016 => self.controllers[0].read_value(),
            0x4017 => self.controllers[1].read_value(),
            0x4000..=0x4015 => {
                let apu = self.bus_mut().apu_mut();
                apu.read_register(addr)
            },
            0x4018..=0x401F => panic!("CPU Test Mode not implemented."),
            0x4020..=0xFFFF => {
                let mapper = self.bus().mapper();
                mapper.read_byte(addr)
            },
            _ => panic!("[CPU] Invalid read with memory address: {:#06x}.", addr),
        }
    }

    pub fn read_word(&mut self, addr: u16) -> u16 {
        (u16::from(self.read_byte(addr + 1)) << 8) | u16::from(self.read_byte(addr))
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr % 0x0800) as usize] = val,
            0x2000..=0x3FFF => {
                let ppu = self.bus_mut().ppu_mut();
                let old_nmi_enabled = ppu.r.nmi_enabled;
                let addr = (addr - 0x2000) % 8 + 0x2000;
                ppu.write_register(addr, val);
                let nmi_enabled_toggled = !old_nmi_enabled && ppu.r.nmi_enabled;

                if nmi_enabled_toggled && ppu.r.v_blank_started {
                    self.trigger_interrupt(Interrupt::NMI);
                }
            },
            0x4014 => {
                // println!("[CPU] Performing OAM DMA on address {:#06x}.", val);
                let cpu_addr = u16::from(val) << 8;
                for offset in 0..=0xFF {
                    let cpu_addr = cpu_addr + offset;
                    let cpu_val = self.read_byte(cpu_addr);
                    let ppu = self.bus_mut().ppu_mut();
                    let oam_addr = ppu.r.oam_addr;
                    ppu.primary_oam[oam_addr as usize] = cpu_val;
                    ppu.r.oam_addr = oam_addr.wrapping_add(1);
                }

                if self.cycle % 2 == 1 {
                    self.stall_cycle += 514;
                } else {
                    self.stall_cycle += 513;
                }
            },
            0x4016 => {
                self.controllers[0].write_strobe(val & 0x01 != 0);
                self.controllers[1].write_strobe(val & 0x01 != 0);
            },
            0x4000..=0x4017 => {
                let apu = self.bus_mut().apu_mut();
                apu.write_register(addr, val);
            },
            0x4018..=0x401F => panic!("CPU Test Mode not implemented."),
            0x4020..=0xFFFF => {
                let mapper = self.bus_mut().mapper_mut();
                mapper.write_byte(addr, val);
            },
            _ => panic!("[CPU] Invalid write with memory address: {:#06x}.", addr),
        }
    }

    fn execute_opcode(&mut self, opcode: u8) {
        // let ppu = self.bus().ppu();
        // println!("A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:3} SL:{}", self.r.a, self.r.x, self.r.y, self.r.p, self.r.sp, (self.cycle * 3) % 341, ppu.scanline);
        let addressing_mode = opcodes::ADDRESSING_MODE_TABLE[opcode as usize];
        opcodes::INSTRUCTION_TABLE[opcode as usize](self, addressing_mode);
        self.cycle += u64::from(opcodes::CYCLE_TABLE[opcode as usize]);
    }

    fn get_operand(&mut self, addressing_mode: usize) -> opcodes::Operand {
        match addressing_mode {
            addressing_modes::ACCUMULATOR => {
                opcodes::Operand {
                    val: self.r.a,
                    addr: None,
                    page_crossing: false,
                }
            },
            _ => {
                let (addr, page_crossing) = addressing_modes::FUNCTION_TABLE[addressing_mode](self);
                opcodes::Operand {
                    val: self.read_byte(addr),
                    addr: Some(addr),
                    page_crossing,
                }
            },
        }
    }

    fn write_operand(&mut self, operand: &opcodes::Operand) {
        match operand.addr {
            Some(addr) => self.write_byte(addr, operand.val),
            None => self.r.a = operand.val,
        }
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Cpu::new()
    }
}

#[derive(Debug, PartialEq)]
pub enum Interrupt {
    NMI = 0,
    IRQ = 1,
}

const INTERRUPT_HANDLERS: [u16; 2] = [0xFFFA, 0xFFFE];
