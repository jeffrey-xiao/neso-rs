use bus::Bus;
use std::f64::consts;

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

pub trait FirstOrderFilter {
    fn filter(&mut self, input_sample: f64) -> f64;
}

struct FirstOrderFilterParams {
    pub frequency: u64,
    pub sample_rate: u64,
    pub rc: f64,
    pub dt: f64,
    pub alpha: f64,
}

impl FirstOrderFilterParams {
    pub fn new(frequency: u64, sample_rate: u64) -> Self {
        let rc = 1.0 / (2.0 * consts::PI * frequency as f64);
        let dt = 1.0 / sample_rate as f64;
        FirstOrderFilterParams {
            frequency,
            sample_rate,
            rc,
            dt,
            alpha: dt / (rc + dt),
        }
    }
}

struct LowPassFilter {
    prev_input_sample: f64,
    prev_output_sample: f64,
    params: FirstOrderFilterParams,
}

impl LowPassFilter {
    pub fn new(frequency: u64, sample_rate: u64) -> Self {
        LowPassFilter {
            prev_input_sample: 0.0,
            prev_output_sample: 0.0,
            params: FirstOrderFilterParams::new(frequency, sample_rate),
        }
    }
}

impl FirstOrderFilter for LowPassFilter {
    fn filter(&mut self, input_sample: f64) -> f64 {
        let output_sample = self.prev_output_sample + self.params.alpha * (input_sample - self.prev_input_sample);
        self.prev_input_sample = input_sample;
        self.prev_output_sample = output_sample;
        output_sample
    }
}

struct HighPassFilter {
    prev_input_sample: f64,
    prev_output_sample: f64,
    params: FirstOrderFilterParams,
}

impl HighPassFilter {
    pub fn new(frequency: u64, sample_rate: u64) -> Self {
        HighPassFilter {
            prev_input_sample: 0.0,
            prev_output_sample: 0.0,
            params: FirstOrderFilterParams::new(frequency, sample_rate),
        }
    }
}

impl FirstOrderFilter for HighPassFilter {
    fn filter(&mut self, input_sample: f64) -> f64 {
        let output_sample = self.params.alpha * (self.prev_output_sample + input_sample - self.prev_input_sample);
        self.prev_input_sample = input_sample;
        self.prev_output_sample = output_sample;
        output_sample
    }
}

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
    envelope_reset: bool,
    sweep_enabled: bool,
    sweep_period: u8,
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
            envelope_reset: false,
            sweep_enabled: false,
            sweep_period: 0,
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
    pub frame_counter_mode: FrameCounterMode,
}

impl Apu {
    pub fn new() -> Self {
        Apu {
            cycle: 0,
            bus: None,
            pulses: [Pulse::default(), Pulse::default()],
            frame_counter_mode: FrameCounterMode::FourStep,
        }
    }

    pub fn attach_bus(&mut self, bus: Bus) {
        self.bus = Some(bus);
    }

    fn _bus(&self) -> &Bus {
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
                self.pulses[index].envelope_reset = true;
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
            0x4017 => {},
            _ => {},
        }
    }

    pub fn step(&mut self) {
        self.cycle += 1;

        if self.cycle % 2 == 0 {
            self.pulses[0].step();
            self.pulses[1].step();
        }
        // TODO: Handle other waves and frame counter
    }
}
