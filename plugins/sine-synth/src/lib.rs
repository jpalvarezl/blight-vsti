use dsp_core::{envelopes::ADSREnvelope, oscillators::SineOsc, utils::midi_to_freq};
use nih_plug::prelude::*;
use std::sync::Arc;

const MAX_VOICES: usize = 16;

struct SineSynth {
    params: Arc<SynthParams>,
    voices: [Voice; MAX_VOICES],
    next_voice: usize,
}

#[derive(Clone)]
struct Voice {
    osc: SineOsc,
    env: ADSREnvelope,
    note: Option<u8>,
    velocity: f32,
}

#[derive(Params)]
struct SynthParams {
    #[id = "gain"]
    pub gain: FloatParam,

    #[id = "attack"]
    pub attack: FloatParam,

    #[id = "decay"]
    pub decay: FloatParam,

    #[id = "sustain"]
    pub sustain: FloatParam,

    #[id = "release"]
    pub release: FloatParam,
}

impl Default for SineSynth {
    fn default() -> Self {
        Self {
            params: Arc::new(SynthParams::default()),
            voices: std::array::from_fn(|_| Voice {
                osc: SineOsc::new(44100.0),
                env: ADSREnvelope::new(44100.0),
                note: None,
                velocity: 0.0,
            }),
            next_voice: 0,
        }
    }
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(-12.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            attack: FloatParam::new(
                "Attack",
                0.01,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 5.0,
                    factor: 0.25,
                },
            )
            .with_unit(" s")
            .with_value_to_string(formatters::v2s_f32_rounded(3)),

            decay: FloatParam::new(
                "Decay",
                0.1,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 5.0,
                    factor: 0.25,
                },
            )
            .with_unit(" s")
            .with_value_to_string(formatters::v2s_f32_rounded(3)),

            sustain: FloatParam::new("Sustain", 0.7, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_value_to_string(formatters::v2s_f32_percentage(1)),

            release: FloatParam::new(
                "Release",
                0.2,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 5.0,
                    factor: 0.25,
                },
            )
            .with_unit(" s")
            .with_value_to_string(formatters::v2s_f32_rounded(3)),
        }
    }
}

impl Plugin for SineSynth {
    const NAME: &'static str = "Sine Synth";
    const VENDOR: &'static str = "Your Studio";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "contact@yourstudio.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Initialize all voices with correct sample rate
        for voice in &mut self.voices {
            voice.osc = SineOsc::new(buffer_config.sample_rate);
            voice.env = ADSREnvelope::new(buffer_config.sample_rate);
        }
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let mut next_event = context.next_event();
        let gain = self.params.gain.smoothed.next();

        for (sample_id, channel_samples) in buffer.iter_samples().enumerate() {
            // Process MIDI events for this sample
            while let Some(event) = next_event {
                if event.timing() != sample_id as u32 {
                    break;
                }

                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        // Find available voice or steal oldest
                        let voice_idx = self.find_free_voice().unwrap_or_else(|| {
                            let idx = self.next_voice;
                            self.next_voice = (self.next_voice + 1) % MAX_VOICES;
                            idx
                        });

                        let voice = &mut self.voices[voice_idx];
                        voice.note = Some(note);
                        voice.velocity = velocity;
                        voice.osc.set_frequency(midi_to_freq(note));
                        voice.osc.reset();
                        voice.env.note_on();
                    }
                    NoteEvent::NoteOff { note, .. } => {
                        // Find and release the voice playing this note
                        for voice in &mut self.voices {
                            if voice.note == Some(note) {
                                voice.env.note_off();
                            }
                        }
                    }
                    _ => {}
                }

                next_event = context.next_event();
            }

            // Generate audio from active voices
            let mut sample_l = 0.0;
            let mut sample_r = 0.0;

            for voice in &mut self.voices {
                if voice.env.is_active() {
                    let osc_sample = voice.osc.next_sample();
                    let env_sample = voice.env.next_sample();
                    let voice_sample = osc_sample * env_sample * voice.velocity * gain;

                    sample_l += voice_sample;
                    sample_r += voice_sample;
                }
            }

            // Apply to all channels
            for (channel_idx, sample) in channel_samples.into_iter().enumerate() {
                *sample = if channel_idx % 2 == 0 {
                    sample_l
                } else {
                    sample_r
                };
            }
        }

        ProcessStatus::Normal
    }
}

impl SineSynth {
    fn find_free_voice(&self) -> Option<usize> {
        self.voices.iter().position(|v| !v.env.is_active())
    }
}

impl ClapPlugin for SineSynth {
    const CLAP_ID: &'static str = "com.yourstudio.sine-synth";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A polyphonic sine wave synthesizer");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for SineSynth {
    const VST3_CLASS_ID: [u8; 16] = *b"SineSynth0000000";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth];
}

nih_export_clap!(SineSynth);
nih_export_vst3!(SineSynth);
