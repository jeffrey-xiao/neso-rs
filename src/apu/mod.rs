mod filter;
mod mixer;

use self::filter::{FirstOrderFilter, HighPassFilter, LowPassFilter};
use self::mixer::Mixer;
use cpu::Interrupt;
use bus::Bus;

// https://wiki.nesdev.com/w/index.php/APU_Length_Counter
#[cfg_attr(rustfmt, rustfmt_skip)]
const LENGTH_COUNTER_TABLE: [u8; 32] = [
    10, 254, 20,  2, 40,  4, 80,  6, 160,  8, 60, 10, 14, 12, 26, 14,
    12,  16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];

// https://wiki.nesdev.com/w/index.php/APU_Pulse
#[cfg_attr(rustfmt, rustfmt_skip)]
const DUTY_CYCLE_TABLE: [u8; 32] = [
  0, 1, 0, 0, 0, 0, 0, 0,
  0, 1, 1, 0, 0, 0, 0, 0,
  0, 1, 1, 1, 1, 0, 0, 0,
  1, 0, 0, 1, 1, 1, 1, 1,
];

const CLOCK_FREQ: u64 = 1_789_773;
const SAMPLE_FREQ: u64 = 44_100;

pub enum FrameCounterMode {
    FourStep,
    FiveStep,
}

pub struct Pulse {
    enabled: bool,
    duty_cycle: u8,
    duty_val: u8,
    length_counter_enabled: bool,
    length_counter: u8,
    timer_period: u16,
    timer_val: u16,
    envelope_enabled: bool,
    envelope_loop: bool,
    envelope_period: u8,
    envelope_decay_val: u8,
    envelope_val: u8,
    envelope_reset: bool,
    sweep_enabled: bool,
    sweep_period: u8,
    sweep_val: u8,
    sweep_negated: bool,
    sweep_shift: u8,
    sweep_reset: bool,
    constant_volume: u8,
}

impl Pulse {
    pub fn new() -> Self {
        Pulse {
            enabled: false,
            duty_cycle: 0,
            duty_val: 0,
            length_counter_enabled: false,
            length_counter: 0,
            timer_period: 0,
            timer_val: 0,
            envelope_enabled: false,
            envelope_loop: false,
            envelope_period: 0,
            envelope_decay_val: 0,
            envelope_val: 0,
            envelope_reset: false,
            sweep_enabled: false,
            sweep_period: 0,
            sweep_val: 0,
            sweep_negated: false,
            sweep_shift: 0,
            sweep_reset: false,
            constant_volume: 0,
        }
    }

    pub fn step(&mut self) {
        if self.timer_val == 0 {
            self.timer_val = self.timer_period;
        } else {
            self.timer_val -= 1;
            if self.timer_val == 0 {
                self.duty_cycle = (self.duty_cycle + 1) % 8;
            }
        }
    }

    pub fn output(&self) -> u8 {
        let ret = DUTY_CYCLE_TABLE[self.duty_cycle as usize * 8 + self.duty_val as usize];
        // TODO: Handle envelope
        if self.enabled || ret == 0 {
            return 0;
        }
        ret
    }
}

impl Default for Pulse {
    fn default() -> Self {
        Pulse::new()
    }
}

pub struct Apu {
    pub cycle: u64,
    pub bus: Option<Bus>,
    pub pulses: [Pulse; 2],
    pub filters: [Box<FirstOrderFilter>; 3],
    pub mixer: Mixer,
    pub frame_counter_mode: FrameCounterMode,
    pub frame_counter_val: u64,
    pub inhibit_irq: bool,
}

impl Apu {
    pub fn new() -> Self {
        Apu {
            cycle: 0,
            bus: None,
            pulses: [Pulse::default(), Pulse::default()],
            filters: [
                Box::new(HighPassFilter::new(90, 1)),
                Box::new(HighPassFilter::new(440, 1)),
                Box::new(LowPassFilter::new(14000, 1)),
            ],
            mixer: Mixer::new(),
            frame_counter_mode: FrameCounterMode::FourStep,
            frame_counter_val: 0,
            inhibit_irq: false,
        }
    }

    pub fn attach_bus(&mut self, bus: Bus) {
        self.bus = Some(bus);
    }

    fn bus(&self) -> &Bus {
        self.bus.as_ref().expect("[APU] No bus attached.")
    }

    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            0x4015 => {
                let mut ret = 0;
                for (index, pulse) in self.pulses.iter().enumerate() {
                    if pulse.enabled {
                        ret |= 1 << index;
                    }
                }
                // TODO: Handle other waves
                ret
            },
            _ => 0,
        }
    }

    pub fn write_register(&mut self, addr: u16, val: u8) {
        match addr {
            0x4000 | 0x4004 => {
                let index = ((addr - 0x4000) / 4) as usize;
                self.pulses[index].duty_cycle = val >> 6;
                self.pulses[index].length_counter_enabled = val & 0x20 == 0;
                self.pulses[index].envelope_loop = val & 0x20 != 0;
                self.pulses[index].envelope_enabled = val & 0x10 == 0;
                self.pulses[index].envelope_period = val & 0x0F;
                self.pulses[index].constant_volume = val & 0x0F;
            },
            0x4001 | 0x4005 => {
                let index = ((addr - 0x4000) / 4) as usize;
                self.pulses[index].sweep_enabled = val & 0x80 != 0;
                self.pulses[index].sweep_period = (val >> 4) & 0x07 + 1;
                self.pulses[index].sweep_negated = val & 0x08 != 0;
                self.pulses[index].sweep_shift = val & 0x07;
                self.pulses[index].sweep_reset = true;
            },
            0x4002 | 0x4006 => {
                let index = ((addr - 0x4000) / 4) as usize;
                let val = val as u16;
                self.pulses[index].timer_period = (self.pulses[index].timer_period & 0xFF00) | val;
            },
            0x4003 | 0x4007 => {
                let index = ((addr - 0x4000) / 4) as usize;
                let val = ((val as u16) & 0x07) << 8;
                self.pulses[index].timer_period = (self.pulses[index].timer_period & 0x00FF) | val;
                self.pulses[index].length_counter =
                    LENGTH_COUNTER_TABLE[(val as usize & 0xF8) >> 3];
                self.pulses[index].duty_val = 0;
                self.pulses[index].envelope_reset = true;
            },
            0x4015 => {
                self.pulses[0].enabled = val & 0x01 != 0;
                self.pulses[1].enabled = val & 0x02 != 0;

                for pulse in &mut self.pulses {
                    if !pulse.enabled {
                        pulse.length_counter = 0;
                    }
                }
                // TODO: Handle other waves
            },
            0x4017 => {
                self.frame_counter_mode = if val >> 7 == 0 {
                    FrameCounterMode::FourStep
                } else {
                    FrameCounterMode::FiveStep
                };

                self.inhibit_irq = (val >> 6) & 0x01 != 0;
            },
            _ => {},
        }
    }

    fn step_envelope(&mut self) {
        // TODO: Modularize
        for pulse in &mut self.pulses {
            if pulse.envelope_reset {
                pulse.envelope_reset = false;
                pulse.envelope_decay_val = 15;
                pulse.envelope_val = pulse.envelope_period;
            } else if pulse.envelope_val > 0 {
                pulse.envelope_val -= 1;
            } else {
                pulse.envelope_val = pulse.envelope_period;
                if pulse.envelope_decay_val > 0 {
                    pulse.envelope_decay_val -= 1;
                } else if pulse.envelope_loop {
                    pulse.envelope_decay_val = 15;
                }
            }
        }
    }

    fn step_length_counter(&mut self) {
        // TODO: Modularize
        for pulse in &mut self.pulses {
            if pulse.length_counter_enabled && pulse.length_counter > 0 {
                pulse.length_counter -= 1;
            }
        }
    }

    fn step_sweep(&mut self) {
        // TODO: Modularize
        for (index, pulse) in self.pulses.iter_mut().enumerate() {
            if pulse.sweep_val == 0 && pulse.sweep_enabled {
                let mut change_amount = (pulse.timer_period >> pulse.sweep_shift) as i16;
                if pulse.sweep_negated {
                    change_amount = -change_amount + (index as i16) - 1;
                }
                // Cannot be negative
                let target_timer_period = (pulse.timer_period as i16 + change_amount) as u16;

                // Change period if not muted
                if 0x0008 <= target_timer_period && target_timer_period < 0x07FF {
                    pulse.timer_period = target_timer_period;
                }
            }

            if pulse.sweep_val == 0 || pulse.sweep_reset {
                pulse.sweep_reset = false;
                pulse.sweep_val = pulse.sweep_period;
            } else {
                pulse.sweep_val -= 1;
            }
        }
    }

    pub fn step(&mut self) {
        self.cycle += 1;

        if self.cycle % 2 == 0 {
            self.pulses[0].step();
            self.pulses[1].step();
        }

        // Frame counter ticks at 240 Hz
        if self.cycle % (CLOCK_FREQ / 240) == 0 {
            match self.frame_counter_mode {
                FrameCounterMode::FourStep => match self.frame_counter_val % 4 {
                    0 | 2 => {
                        // envelope
                        self.step_envelope();
                    },
                    1 => {
                        // envelope
                        self.step_envelope();
                        // length counter and sweep
                        self.step_length_counter();
                        self.step_sweep();
                    },
                    3 => {
                        // envelope
                        self.step_envelope();
                        // length counter and sweep
                        self.step_length_counter();
                        self.step_sweep();
                        // irq
                        if !self.inhibit_irq {
                            let cpu = self.bus().cpu();
                            cpu.borrow_mut().trigger_interrupt(Interrupt::IRQ);
                        }
                    },
                    _ => {},
                },
                FrameCounterMode::FiveStep => match self.frame_counter_val % 5 {
                    0 | 2 => {
                        // envelope
                        self.step_envelope();
                        // length counter
                        self.step_length_counter();
                        self.step_sweep();
                    },
                    1 | 3 => {
                        // envelope
                        self.step_envelope();
                    },
                    _ => {},
                },
            }
        }

        // Output sample device is 44.1 kHz
        if self.cycle % (CLOCK_FREQ / SAMPLE_FREQ) == 0 {

        }
    }
}
