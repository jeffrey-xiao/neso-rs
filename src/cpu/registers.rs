pub const CARRY_MASK: u8 = 0x01;
pub const ZERO_MASK: u8 = 0x02;
pub const INTERRUPT_DISABLE_MASK: u8 = 0x04;
pub const DECIMAL_MODE_MASK: u8 = 0x08;
pub const BREAK_COMMAND_MASK: u8 = 0x10;
pub const OVERFLOW_MASK: u8 = 0x40;
pub const NEGATIVE_MASK: u8 = 0x80;

pub struct Registers {
    pub pc: u16,
    pub sp: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            // pc: 0xC000,
            pc: 0,
            sp: 0xFD,
            a: 0,
            x: 0,
            y: 0,
            p: 0x24,
        }
    }

    // status flag related instructions
    pub fn set_status_flag(&mut self, mask: u8, set: bool) {
        if set {
            self.p |= mask;
        } else {
            self.p &= !mask;
        }
    }

    pub fn get_status_flag(&mut self, mask: u8) -> bool {
        self.p & mask != 0
    }

    pub fn update_negative_flag(&mut self, val: u8) {
        self.set_status_flag(NEGATIVE_MASK, val & 0x80 != 0);
    }

    pub fn update_zero_flag(&mut self, val: u8) {
        self.set_status_flag(ZERO_MASK, val == 0);
    }

    pub fn update_nz_flags(&mut self, val: u8) {
        self.update_negative_flag(val);
        self.update_zero_flag(val);
    }
}
