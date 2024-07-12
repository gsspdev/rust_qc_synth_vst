#[macro_use]
extern crate vst;

use vst::prelude::*;
use vst::util::AtomicFloat;

use std::sync::Arc;
use std::f32::consts::PI;

struct SineWaveSynth {
    sample_rate: f32,
    time: f32,
    params: Arc<SineWaveSynthParameters>,
    current_note: Option<u8>,
    envelope_level: f32,
    envelope_stage: EnvelopeStage,
}

struct SineWaveSynthParameters {
    volume: AtomicFloat,
    attack: AtomicFloat,
    decay: AtomicFloat,
    sustain: AtomicFloat,
    release: AtomicFloat,
}

enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

impl Default for SineWaveSynth {
    fn default() -> Self {
        SineWaveSynth {
            sample_rate: 44100.0,
            time: 0.0,
            params: Arc::new(SineWaveSynthParameters::default()),
            current_note: None,
            envelope_level: 0.0,
            envelope_stage: EnvelopeStage::Idle,
        }
    }
}

impl Default for SineWaveSynthParameters {
    fn default() -> Self {
        SineWaveSynthParameters {
            volume: AtomicFloat::new(0.5),
            attack: AtomicFloat::new(0.01),
            decay: AtomicFloat::new(0.1),
            sustain: AtomicFloat::new(0.5),
            release: AtomicFloat::new(0.2),
        }
    }
}

impl Plugin for SineWaveSynth {
    fn new(_host: HostCallback) -> Self {
        Default::default()
    }

    fn get_info(&self) -> Info {
        Info {
            name: "Rust Sine Wave Synth".to_string(),
            vendor: "RustAudio".to_string(),
            unique_id: 1357924680,
            category: Category::Synth,
            inputs: 0,
            outputs: 2,
            parameters: 5,
            initial_delay: 0,
            ..Default::default()
        }
    }

    fn init(&mut self) {
        self.sample_rate = 44100.0;
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let samples = buffer.samples();
        let (_, mut outputs) = buffer.split();
        let output_count = outputs.len();
        let per_sample = self.time_per_sample();

        for sample_idx in 0..samples {
            self.update_envelope(per_sample);

            let output = if let Some(note) = self.current_note {
                let frequency = midi_note_to_freq(note);
                (self.time * frequency * 2.0 * PI).sin() * self.envelope_level
            } else {
                0.0
            };

            let output = output * self.params.volume.get();

            for buf_idx in 0..output_count {
                outputs[buf_idx][sample_idx] = output;
            }

            self.time += per_sample;
        }
    }

    fn process_events(&mut self, events: &Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => {
                    match ev.data[0] {
                        128 => self.note_off(ev.data[1]),  // Note off
                        144 => self.note_on(ev.data[1]),   // Note on
                        _ => (),
                    }
                }
                _ => (),
            }
        }
    }

    fn can_do(&self, can_do: CanDo) -> Supported {
        match can_do {
            CanDo::ReceiveMidiEvent => Supported::Yes,
            _ => Supported::Maybe,
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

impl SineWaveSynth {
    fn note_on(&mut self, note: u8) {
        self.current_note = Some(note);
        self.envelope_stage = EnvelopeStage::Attack;
        self.time = 0.0;
    }

    fn note_off(&mut self, note: u8) {
        if self.current_note == Some(note) {
            self.envelope_stage = EnvelopeStage::Release;
        }
    }

    fn update_envelope(&mut self, delta_time: f32) {
        match self.envelope_stage {
            EnvelopeStage::Idle => (),
            EnvelopeStage::Attack => {
                self.envelope_level += delta_time / self.params.attack.get();
                if self.envelope_level >= 1.0 {
                    self.envelope_level = 1.0;
                    self.envelope_stage = EnvelopeStage::Decay;
                }
            }
            EnvelopeStage::Decay => {
                let sustain = self.params.sustain.get();
                self.envelope_level -= (1.0 - sustain) * delta_time / self.params.decay.get();
                if self.envelope_level <= sustain {
                    self.envelope_level = sustain;
                    self.envelope_stage = EnvelopeStage::Sustain;
                }
            }
            EnvelopeStage::Sustain => (),
            EnvelopeStage::Release => {
                self.envelope_level -= self.envelope_level * delta_time / self.params.release.get();
                if self.envelope_level < 0.001 {
                    self.envelope_level = 0.0;
                    self.envelope_stage = EnvelopeStage::Idle;
                    self.current_note = None;
                }
            }
        }
    }

    fn time_per_sample(&self) -> f32 {
        1.0 / self.sample_rate
    }
}

impl PluginParameters for SineWaveSynthParameters {
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.volume.get(),
            1 => self.attack.get(),
            2 => self.decay.get(),
            3 => self.sustain.get(),
            4 => self.release.get(),
            _ => 0.0,
        }
    }

    fn set_parameter(&self, index: i32, value: f32) {
        match index {
            0 => self.volume.set(value),
            1 => self.attack.set(value),
            2 => self.decay.set(value),
            3 => self.sustain.set(value),
            4 => self.release.set(value),
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Volume".to_string(),
            1 => "Attack".to_string(),
            2 => "Decay".to_string(),
            3 => "Sustain".to_string(),
            4 => "Release".to_string(),
            _ => "".to_string(),
        }
    }

    fn get_parameter_label(&self, index: i32) -> String {
        match index {
            0 => "%".to_string(),
            1 | 2 | 4 => "s".to_string(),
            3 => "".to_string(),
            _ => "".to_string(),
        }
    }
}

fn midi_note_to_freq(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

plugin_main!(SineWaveSynth);
