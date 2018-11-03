use std::f32::consts;

pub trait FirstOrderFilter {
    fn process(&mut self, input_sample: f32) -> f32;
}

struct FirstOrderFilterParams {
    pub frequency: u64,
    pub sample_rate: u64,
    pub rc: f32,
    pub dt: f32,
    pub alpha: f32,
}

impl FirstOrderFilterParams {
    pub fn new(frequency: u64, sample_rate: u64) -> Self {
        let rc = 1.0 / (2.0 * consts::PI * frequency as f32);
        let dt = 1.0 / sample_rate as f32;
        FirstOrderFilterParams {
            frequency,
            sample_rate,
            rc,
            dt,
            alpha: dt / (rc + dt),
        }
    }
}

// https://en.wikipedia.org/wiki/Low-pass_filter
pub struct LowPassFilter {
    prev_input_sample: f32,
    prev_output_sample: f32,
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
    fn process(&mut self, input_sample: f32) -> f32 {
        let output_sample =
            self.prev_output_sample + self.params.alpha * (input_sample - self.prev_input_sample);
        self.prev_input_sample = input_sample;
        self.prev_output_sample = output_sample;
        output_sample
    }
}

// https://en.wikipedia.org/wiki/High-pass_filter
pub struct HighPassFilter {
    prev_input_sample: f32,
    prev_output_sample: f32,
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
    fn process(&mut self, input_sample: f32) -> f32 {
        let output_sample =
            self.params.alpha * (self.prev_output_sample + input_sample - self.prev_input_sample);
        self.prev_input_sample = input_sample;
        self.prev_output_sample = output_sample;
        output_sample
    }
}
