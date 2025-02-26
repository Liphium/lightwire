use voice::VoiceInput;

mod encoder;
mod voice;

pub struct Engine {
    voice_input: VoiceInput,
}

impl Engine {
    pub fn create() -> Self {
        let engine = Self {
            voice_input: VoiceInput::new(),
        };

        engine.voice_input.start(move |_| {});

        return engine;
    }
}

pub struct AudioStreamConfig {
    sample_rate: u32,
    frame_size: u32,
}
