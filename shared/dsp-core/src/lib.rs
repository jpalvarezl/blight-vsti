use std::f32::consts::PI;

/// Common oscillator implementations
pub mod oscillators {
    use super::*;

    #[derive(Clone)]
    pub struct SineOsc {
        phase: f32,
        frequency: f32,
        sample_rate: f32,
    }

    impl SineOsc {
        pub fn new(sample_rate: f32) -> Self {
            Self {
                phase: 0.0,
                frequency: 440.0,
                sample_rate,
            }
        }

        pub fn set_frequency(&mut self, freq: f32) {
            self.frequency = freq;
        }

        pub fn next_sample(&mut self) -> f32 {
            let sample = (self.phase * 2.0 * PI).sin();
            self.phase += self.frequency / self.sample_rate;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
            sample
        }

        pub fn reset(&mut self) {
            self.phase = 0.0;
        }
    }
}

/// Common envelope generators
pub mod envelopes {
    #[derive(Clone)]
    pub struct ADSREnvelope {
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
        stage: EnvStage,
        level: f32,
        sample_rate: f32,
    }

    #[derive(Clone, PartialEq)]
    enum EnvStage {
        Idle,
        Attack,
        Decay,
        Sustain,
        Release,
    }

    impl ADSREnvelope {
        pub fn new(sample_rate: f32) -> Self {
            Self {
                attack: 0.01,
                decay: 0.1,
                sustain: 0.7,
                release: 0.2,
                stage: EnvStage::Idle,
                level: 0.0,
                sample_rate,
            }
        }

        pub fn note_on(&mut self) {
            self.stage = EnvStage::Attack;
        }

        pub fn note_off(&mut self) {
            self.stage = EnvStage::Release;
        }

        pub fn next_sample(&mut self) -> f32 {
            match self.stage {
                EnvStage::Idle => 0.0,
                EnvStage::Attack => {
                    self.level += 1.0 / (self.attack * self.sample_rate);
                    if self.level >= 1.0 {
                        self.level = 1.0;
                        self.stage = EnvStage::Decay;
                    }
                    self.level
                }
                EnvStage::Decay => {
                    self.level -= (1.0 - self.sustain) / (self.decay * self.sample_rate);
                    if self.level <= self.sustain {
                        self.level = self.sustain;
                        self.stage = EnvStage::Sustain;
                    }
                    self.level
                }
                EnvStage::Sustain => self.sustain,
                EnvStage::Release => {
                    self.level -= self.level / (self.release * self.sample_rate);
                    if self.level <= 0.001 {
                        self.level = 0.0;
                        self.stage = EnvStage::Idle;
                    }
                    self.level
                }
            }
        }

        pub fn is_active(&self) -> bool {
            self.stage != EnvStage::Idle
        }
    }
}

/// Common utility functions
pub mod utils {
    /// Convert MIDI note number to frequency
    pub fn midi_to_freq(note: u8) -> f32 {
        440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0)
    }

    /// Linear interpolation
    pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }
}
