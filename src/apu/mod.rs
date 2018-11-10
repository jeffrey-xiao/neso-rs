mod filter;
mod mixer;

use self::filter::{FirstOrderFilter, HighPassFilter, LowPassFilter};
use self::mixer::Mixer;
use bus::Bus;
use cpu::Interrupt;

// https://wiki.nesdev.com/w/index.php/APU_Length_Counter
#[rustfmt::skip]
const LENGTH_COUNTER_TABLE: [u8; 32] = [
    10, 254, 20,  2, 40,  4, 80,  6, 160,  8, 60, 10, 14, 12, 26, 14,
    12,  16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];

// https://wiki.nesdev.com/w/index.php/APU_Pulse
#[rustfmt::skip]
const PULSE_TABLE: [u8; 32] = [
  0, 1, 0, 0, 0, 0, 0, 0,
  0, 1, 1, 0, 0, 0, 0, 0,
  0, 1, 1, 1, 1, 0, 0, 0,
  1, 0, 0, 1, 1, 1, 1, 1,
];

// https://wiki.nesdev.com/w/index.php/APU_Triangle
#[rustfmt::skip]
const TRIANGLE_TABLE: [u8; 32] = [
    15, 14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0,
     0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15,
];

// https://wiki.nesdev.com/w/index.php/APU_Noise
#[rustfmt::skip]
const NOISE_PERIOD_TABLE: [u16; 16] = [
      4,   8,  16,  32,  64,   96,  128,  160,
    202, 254, 380, 508, 762, 1016, 2034, 4068,
];

// https://wiki.nesdev.com/w/index.php/APU_DMC
#[rustfmt::skip]
const DMC_PERIOD_TABLE: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214,
    190, 160, 142, 128, 106,  84,  72,  54,
];

const FOUR_STEP_FRAME_COUNTER_CYCLES: [u16; 4] = [7456, 7458, 7458, 7458];
const FIVE_STEP_FRAME_COUNTER_CYCLES: [u16; 5] = [7458, 7456, 7458, 7458, 7452];

const CLOCK_FREQ: u64 = 1_789_773;
const SAMPLE_FREQ: u64 = 44_100;
const SAMPLE_CYCLES: f64 = CLOCK_FREQ as f64 / SAMPLE_FREQ as f64;
const BUFFER_SIZE: usize = 735;

#[derive(Debug)]
pub enum FrameCounterMode {
    FourStep,
    FiveStep,
}

#[derive(Default)]
pub struct LengthCounter {
    pub enabled: bool,
    pub val: u8,
}

impl LengthCounter {
    pub fn step(&mut self) {
        if self.enabled && self.val > 0 {
            self.val -= 1;
        }
    }

    pub fn reload(&mut self, index: usize) {
        self.val = LENGTH_COUNTER_TABLE[index];
    }
}

#[derive(Default)]
pub struct Envelope {
    pub enabled: bool,
    pub looped: bool,
    pub reset: bool,
    pub period: u8,
    pub val: u8,
    pub volume: u8,
}

impl Envelope {
    pub fn step(&mut self) {
        if self.reset {
            self.reset = false;
            self.volume = 15;
            self.val = self.period;
            return;
        }

        if self.val > 0 {
            self.val -= 1;
            return;
        }

        self.val = self.period;
        if self.volume > 0 {
            self.volume -= 1;
        } else if self.looped {
            self.volume = 15;
        }
    }

    pub fn volume(&self) -> u8 {
        if self.enabled {
            self.volume
        } else {
            self.period
        }
    }
}

#[derive(Default)]
pub struct Pulse {
    enabled: bool,
    duty_cycle: u8,
    duty_val: u8,
    length_counter: LengthCounter,
    timer_period: u16,
    timer_val: u16,
    envelope: Envelope,
    sweep_enabled: bool,
    sweep_period: u8,
    sweep_val: u8,
    sweep_negated: bool,
    sweep_shift: u8,
    sweep_reset: bool,
}

impl Pulse {
    pub fn step(&mut self) {
        if self.timer_val > 0 {
            self.timer_val -= 1;
            return;
        }
        self.timer_val = self.timer_period;
        self.duty_val = (self.duty_val + 1) % 8;
    }

    pub fn output(&self) -> u8 {
        let val = PULSE_TABLE[self.duty_cycle as usize * 8 + self.duty_val as usize];
        let is_muted = self.timer_period < 0x0008 || self.timer_period >= 0x07FF;
        if !self.enabled || val == 0 || self.length_counter.val == 0 || is_muted {
            return 0;
        }

        self.envelope.volume()
    }
}

#[derive(Default)]
pub struct Triangle {
    enabled: bool,
    duty_val: u8,
    length_counter: LengthCounter,
    timer_period: u16,
    timer_val: u16,
    linear_counter_enabled: bool,
    linear_counter: u8,
    linear_counter_period: u8,
    linear_counter_reset: bool,
}

impl Triangle {
    pub fn step(&mut self) {
        if self.timer_val > 0 {
            self.timer_val -= 1;
            return;
        }
        self.timer_val = self.timer_period;
        self.duty_val = (self.duty_val + 1) % 32;
    }

    pub fn step_linear_counter(&mut self) {
        if self.linear_counter_reset {
            self.linear_counter = self.linear_counter_period;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }

        if self.linear_counter_enabled {
            self.linear_counter_reset = false;
        }
    }

    pub fn output(&self) -> u8 {
        if !self.enabled || self.linear_counter == 0 || self.length_counter.val == 0 {
            return 0;
        }
        TRIANGLE_TABLE[self.duty_val as usize]
    }
}

pub struct Noise {
    enabled: bool,
    mode: bool,
    timer_period: u16,
    timer_val: u16,
    shift_register: u16,
    length_counter: LengthCounter,
    envelope: Envelope,
}

impl Noise {
    pub fn new() -> Self {
        Noise {
            enabled: false,
            mode: false,
            timer_period: 0,
            timer_val: 0,
            shift_register: 1,
            length_counter: LengthCounter::default(),
            envelope: Envelope::default(),
        }
    }

    pub fn step(&mut self) {
        if self.timer_val > 0 {
            self.timer_val -= 1;
            return;
        }
        self.timer_val = self.timer_period;
        let feedback = ((self.shift_register >> (if self.mode { 6 } else { 1 })) & 0x01)
            ^ (self.shift_register & 0x01);
        self.shift_register = (self.shift_register >> 1) | (feedback << 14);
    }

    pub fn output(&self) -> u8 {
        if !self.enabled || self.shift_register & 0x01 != 0 || self.length_counter.val == 0 {
            return 0;
        }
        self.envelope.volume()
    }
}

impl Default for Noise {
    fn default() -> Self {
        Noise::new()
    }
}

#[derive(Default)]
pub struct Dmc {
    enabled: bool,
    silenced: bool,
    looped: bool,
    irq_enabled: bool,
    irq_pending: bool,
    timer_period: u16,
    timer_val: u16,
    shift_register: u8,
    bits_remaining: u8,
    volume: u8,
    curr_addr: u16,
    sample_addr: u16,
    curr_len: u16,
    sample_len: u16,
}

impl Dmc {
    pub fn restart_sample(&mut self) {
        self.curr_addr = self.sample_addr;
        self.curr_len = self.sample_len;
    }

    pub fn step(&mut self) {
        if self.timer_val > 0 {
            self.timer_val -= 1;
            return;
        }
        self.timer_val = self.timer_period;

        if self.bits_remaining == 0 {
            return;
        }

        if !self.silenced {
            if self.shift_register & 0x01 == 0 {
                if self.volume >= 2 {
                    self.volume -= 2;
                }
            } else if self.volume <= 125 {
                self.volume += 2;
            }
            self.shift_register >>= 1;
        }

        self.bits_remaining -= 1;
    }

    pub fn output(&self) -> u8 {
        self.volume
    }
}

pub struct Apu {
    pub pulses: [Pulse; 2],
    pub triangle: Triangle,
    pub noise: Noise,
    pub dmc: Dmc,
    pub filters: [Box<FirstOrderFilter>; 3],
    pub mixer: Mixer,
    pub buffer_index: usize,
    pub buffer: [f32; BUFFER_SIZE],
    pub frame_counter_mode: FrameCounterMode,
    pub frame_counter_val: u16,
    pub frame_counter_phase: u8,
    pub irq_enabled: bool,
    pub irq_pending: bool,
    pub last_written_byte: u8,
    pub cycle: u64,
    pub bus: Option<Bus>,
}

impl Apu {
    pub fn new() -> Self {
        Apu {
            pulses: [Pulse::default(), Pulse::default()],
            triangle: Triangle::default(),
            noise: Noise::default(),
            dmc: Dmc::default(),
            filters: [
                Box::new(HighPassFilter::new(90, SAMPLE_FREQ)),
                Box::new(HighPassFilter::new(440, SAMPLE_FREQ)),
                Box::new(LowPassFilter::new(14000, SAMPLE_FREQ)),
            ],
            mixer: Mixer::new(),
            buffer_index: 0,
            buffer: [0.0; BUFFER_SIZE],
            frame_counter_mode: FrameCounterMode::FourStep,
            frame_counter_val: FOUR_STEP_FRAME_COUNTER_CYCLES[0],
            frame_counter_phase: 0,
            irq_enabled: false,
            irq_pending: false,
            last_written_byte: 0,
            cycle: 0,
            bus: None,
        }
    }

    pub fn initialize(&mut self) {
        self.write_register(0x4015, 0);
        self.write_register(0x4017, 0);
        for addr in 0x4000..=0x400F {
            self.write_register(addr, 0);
        }
        // Delay after power on.
        for _ in 0..12 {
            self.step();
        }
    }

    pub fn reset(&mut self) {
        self.write_register(0x4015, 0);
        self.irq_pending = false;
        self.write_register(0x4017, self.last_written_byte);
        // Delay after reset.
        for _ in 0..12 {
            self.step();
        }
    }

    pub fn attach_bus(&mut self, bus: Bus) {
        self.bus = Some(bus);
        self.initialize();
    }

    fn bus(&self) -> &Bus {
        self.bus.as_ref().expect("[APU] No bus attached.")
    }

    fn bus_mut(&mut self) -> &mut Bus {
        self.bus.as_mut().expect("[APU] No bus attached.")
    }

    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            0x4015 => {
                let mut ret = 0;
                for (index, pulse) in self.pulses.iter().enumerate() {
                    if pulse.length_counter.val > 0 {
                        ret |= 1 << index;
                    }
                }

                if self.triangle.length_counter.val > 0 {
                    ret |= 0x04;
                }

                if self.noise.length_counter.val > 0 {
                    ret |= 0x08;
                }

                if self.dmc.curr_len > 0 {
                    ret |= 0x10;
                }

                if self.irq_pending {
                    ret |= 0x40;
                }
                self.irq_pending = false;

                if self.dmc.irq_pending {
                    ret |= 0x80;
                }

                ret
            },
            _ => 0,
        }
    }

    pub fn write_register(&mut self, addr: u16, val: u8) {
        match addr {
            // Pulse
            0x4000 | 0x4004 => {
                let index = ((addr - 0x4000) / 4) as usize;
                self.pulses[index].duty_cycle = val >> 6;
                self.pulses[index].length_counter.enabled = val & 0x20 == 0;
                self.pulses[index].envelope.looped = val & 0x20 != 0;
                self.pulses[index].envelope.enabled = val & 0x10 == 0;
                self.pulses[index].envelope.period = val & 0x0F;
            },
            0x4001 | 0x4005 => {
                let index = ((addr - 0x4000) / 4) as usize;
                self.pulses[index].sweep_period = ((val >> 4) & 0x07) + 1;
                self.pulses[index].sweep_negated = val & 0x08 != 0;
                self.pulses[index].sweep_shift = val & 0x07;
                self.pulses[index].sweep_reset = true;
                self.pulses[index].sweep_enabled =
                    val & 0x80 != 0 && self.pulses[index].sweep_shift != 0;
            },
            0x4002 | 0x4006 => {
                let index = ((addr - 0x4000) / 4) as usize;
                let timer_period_low = u16::from(val);
                self.pulses[index].timer_period &= 0xFF00;
                self.pulses[index].timer_period |= timer_period_low;
            },
            0x4003 | 0x4007 => {
                let index = ((addr - 0x4000) / 4) as usize;
                let timer_period_high = (u16::from(val) & 0x07) << 8;
                self.pulses[index].timer_period &= 0x00FF;
                self.pulses[index].timer_period |= timer_period_high;
                if self.pulses[index].enabled {
                    self.pulses[index].length_counter.reload(val as usize >> 3);
                }
                // Timer should _not_ be reset according to the APU Phase Reset Test ROM.
                self.pulses[index].duty_val = 0;
                self.pulses[index].envelope.reset = true;
            },
            // Triangle
            0x4008 => {
                self.triangle.length_counter.enabled = val & 0x80 == 0;
                self.triangle.linear_counter_enabled = val & 0x80 == 0;
                self.triangle.linear_counter_period = val & 0x7F;
            },
            0x400A => {
                let timer_period_low = u16::from(val);
                self.triangle.timer_period &= 0xFF00;
                self.triangle.timer_period |= timer_period_low;
            },
            0x400B => {
                let timer_period_high = (u16::from(val) & 0x07) << 8;
                self.triangle.timer_period &= 0x00FF;
                self.triangle.timer_period |= timer_period_high;
                if self.triangle.enabled {
                    self.triangle.length_counter.reload(val as usize >> 3);
                }
                self.triangle.linear_counter_reset = true;
            },
            // Noise
            0x400C => {
                self.noise.length_counter.enabled = val & 0x20 == 0;
                self.noise.envelope.looped = val & 0x20 != 0;
                self.noise.envelope.enabled = val & 0x10 == 0;
                self.noise.envelope.period = val & 0x0F;
            },
            0x400E => {
                self.noise.mode = val & 0x80 != 0;
                self.noise.timer_period = NOISE_PERIOD_TABLE[(val & 0x0F) as usize];
            },
            0x400F => {
                if self.noise.enabled {
                    self.noise.length_counter.reload(val as usize >> 3);
                }
                self.noise.envelope.reset = true;
            },
            // DMC
            0x4010 => {
                self.dmc.irq_enabled = val & 0x80 != 0;
                if !self.dmc.irq_enabled {
                    self.dmc.irq_pending = false;
                }
                self.dmc.looped = val & 0x40 != 0;
                self.dmc.timer_period = DMC_PERIOD_TABLE[(val & 0x0F) as usize];
            },
            0x4011 => self.dmc.volume = val & 0x7F,
            0x4012 => self.dmc.sample_addr = 0xC000 | (u16::from(val) << 6),
            0x4013 => self.dmc.sample_len = 1 | (u16::from(val) << 4),
            // All
            0x4015 => {
                self.pulses[0].enabled = val & 0x01 != 0;
                self.pulses[1].enabled = val & 0x02 != 0;
                self.triangle.enabled = val & 0x04 != 0;
                self.noise.enabled = val & 0x08 != 0;
                self.dmc.enabled = val & 0x10 != 0;
                self.dmc.irq_pending = false;

                for pulse in &mut self.pulses {
                    if !pulse.enabled {
                        pulse.length_counter.val = 0;
                    }
                }

                if !self.triangle.enabled {
                    self.triangle.length_counter.val = 0;
                }

                if !self.noise.enabled {
                    self.noise.length_counter.val = 0;
                }

                if !self.dmc.enabled {
                    self.dmc.curr_len = 0;
                } else if self.dmc.curr_len == 0 {
                    self.dmc.restart_sample();
                }
            },
            0x4017 => {
                self.last_written_byte = val;
                // Special timings for writing to 0x4017.
                self.frame_counter_mode = if val >> 7 == 0 {
                    // TODO: This is not exactly right.
                    self.frame_counter_val = 7458;
                    FrameCounterMode::FourStep
                } else {
                    // TODO: Handle jitter.
                    self.frame_counter_val = 0;
                    FrameCounterMode::FiveStep
                };
                self.irq_enabled = (val >> 6) & 0x01 == 0;
                if !self.irq_enabled {
                    self.irq_pending = false;
                }
                self.frame_counter_phase = 0;
            },
            _ => {},
        }
    }

    fn step_envelope(&mut self) {
        for pulse in &mut self.pulses {
            pulse.envelope.step();
        }
        self.noise.envelope.step();
    }

    fn step_length_counter(&mut self) {
        for pulse in &mut self.pulses {
            pulse.length_counter.step();
        }
        self.triangle.length_counter.step();
        self.noise.length_counter.step();
    }

    fn step_sweep(&mut self) {
        // TODO: Modularize
        for (index, pulse) in self.pulses.iter_mut().enumerate() {
            if pulse.sweep_reset {
                pulse.sweep_reset = false;
                pulse.sweep_val = pulse.sweep_period;
            } else if pulse.sweep_val > 0 {
                pulse.sweep_val -= 1;
            } else {
                pulse.sweep_val = pulse.sweep_period;
                if pulse.sweep_enabled {
                    let change_amount = pulse.timer_period >> pulse.sweep_shift;
                    let target_timer_period = if pulse.sweep_negated {
                        pulse.timer_period - change_amount + index as u16 - 1
                    } else {
                        pulse.timer_period + change_amount
                    };

                    // TODO: `target_timer_period` should be continuously computed which affects if
                    // the sweep unit mutes the channel.
                    if 0x08 <= target_timer_period && target_timer_period < 0x07FF {
                        pulse.timer_period = target_timer_period;
                    }
                }
            }
        }
    }

    fn step_dmc_read(&mut self) {
        // TODO: Ideally, this should be in Dmc, but we need to find a way to pass bus to Dmc.
        if self.dmc.bits_remaining != 0 || self.dmc.curr_len == 0 {
            return;
        }
        self.dmc.bits_remaining = 8;
        // TODO: Pause for 2 cycles if OAM DMA is in progress.
        let addr = self.dmc.curr_addr;
        let val = {
            let cpu = self.bus_mut().cpu_mut();
            cpu.stall_cycle += 4;
            cpu.read_byte(addr)
        };
        self.dmc.shift_register = val;
        let (next_addr, overflow) = self.dmc.curr_addr.overflowing_add(1);
        self.dmc.curr_addr = if overflow { 0x8000 } else { next_addr };
        self.dmc.curr_len -= 1;

        if self.dmc.curr_len == 0 {
            if self.dmc.looped {
                self.dmc.restart_sample();
            } else if self.dmc.irq_enabled {
                self.dmc.irq_pending = true;
                self.bus_mut().cpu_mut().trigger_interrupt(Interrupt::IRQ);
            }
        }
    }

    pub fn step(&mut self) {
        self.cycle += 1;
        let curr_cycle = self.cycle as f64;
        let next_cycle = (self.cycle + 1) as f64;

        self.triangle.step();
        if self.dmc.enabled {
            self.dmc.step();
            self.step_dmc_read();
        }
        if self.cycle % 2 == 0 {
            self.pulses[0].step();
            self.pulses[1].step();
            self.noise.step();
        }

        if self.frame_counter_val > 0 {
            self.frame_counter_val -= 1;
        } else {
            match self.frame_counter_mode {
                FrameCounterMode::FourStep => {
                    let index = self.frame_counter_phase as usize;
                    self.frame_counter_val = FOUR_STEP_FRAME_COUNTER_CYCLES[index] - 1;
                    match self.frame_counter_phase {
                        0 | 2 => {
                            // envelope
                            self.step_envelope();
                            self.triangle.step_linear_counter();
                            self.frame_counter_phase += 1;
                        },
                        1 => {
                            // envelope
                            self.step_envelope();
                            self.triangle.step_linear_counter();
                            // length counter and sweep
                            self.step_length_counter();
                            self.step_sweep();
                            self.frame_counter_phase += 1;
                        },
                        3 => {
                            // envelope
                            self.step_envelope();
                            self.triangle.step_linear_counter();
                            // length counter and sweep
                            self.step_length_counter();
                            self.step_sweep();
                            // irq
                            if self.irq_enabled {
                                self.irq_pending = true;
                                let cpu = self.bus_mut().cpu_mut();
                                cpu.trigger_interrupt(Interrupt::IRQ);
                            }
                            self.frame_counter_phase = 0;
                        },
                        _ => panic!("[APU] Invalid frame counter phase."),
                    }
                },
                FrameCounterMode::FiveStep => {
                    let index = self.frame_counter_phase as usize;
                    self.frame_counter_val = FIVE_STEP_FRAME_COUNTER_CYCLES[index] - 1;
                    match self.frame_counter_phase {
                        0 | 2 => {
                            // envelope
                            self.step_envelope();
                            self.triangle.step_linear_counter();
                            // length counter
                            self.step_length_counter();
                            self.step_sweep();
                            self.frame_counter_phase += 1;
                        },
                        1 => {
                            // envelope
                            self.step_envelope();
                            self.triangle.step_linear_counter();
                            self.frame_counter_phase += 1;
                        },
                        3 => {
                            // envelope
                            self.step_envelope();
                            self.triangle.step_linear_counter();
                            self.frame_counter_phase = 0;
                        },
                        _ => panic!("[APU] Invalid frame counter phase."),
                    }
                },
            }
        }

        // Output sample device is 44.1 kHz
        let curr_sample = f64::floor(curr_cycle / SAMPLE_CYCLES) as u64;
        let next_sample = f64::floor(next_cycle as f64 / SAMPLE_CYCLES) as u64;
        if curr_sample != next_sample {
            let mut sample = self.mixer.sample(
                self.pulses[0].output(),
                self.pulses[1].output(),
                self.triangle.output(),
                self.noise.output(),
                self.dmc.output(),
            );

            for filter in &mut self.filters {
                sample = filter.process(sample);
            }

            self.buffer[self.buffer_index] = sample;
            self.buffer_index += 1;
        }
    }
}
