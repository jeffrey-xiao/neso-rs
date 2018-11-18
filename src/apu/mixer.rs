// https://wiki.nesdev.com/w/index.php/APU_Mixer#Emulation
pub struct Mixer {
    pulse_table: [f32; 31],
    tnd_table: [f32; 203],
}

impl Mixer {
    pub fn new() -> Self {
        let mut pulse_table = [0.0; 31];
        for (index, val) in pulse_table.iter_mut().enumerate() {
            *val = 95.52 / (8128.0 / index as f32 + 100.0);
        }

        let mut tnd_table = [0.0; 203];
        for (index, val) in tnd_table.iter_mut().enumerate() {
            *val = 163.67 / (24329.0 / index as f32 + 100.0)
        }

        Mixer {
            pulse_table,
            tnd_table,
        }
    }

    pub fn sample(
        &self,
        pulse_1_output: u8,
        pulse_2_output: u8,
        triangle_output: u8,
        noise_output: u8,
        dmc_output: u8,
    ) -> f32 {
        let pulse_table_index = (pulse_1_output + pulse_2_output) as usize;
        let pulse_out = self.pulse_table[pulse_table_index];
        let tnd_table_index = (3 * triangle_output + 2 * noise_output + dmc_output) as usize;
        let tnd_out = self.tnd_table[tnd_table_index];
        pulse_out + tnd_out
    }
}

impl Default for Mixer {
    fn default() -> Self {
        Mixer::new()
    }
}
