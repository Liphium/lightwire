use std::sync::{Arc, Mutex};

use encoder::EncodingEngine;
use voice::VoiceInput;

mod encoder;
mod voice;

pub struct Engine {
    voice_input: VoiceInput,
    encoding_engine: Arc<Mutex<EncodingEngine>>,
}

impl Engine {
    pub fn create() -> Self {
        // Create the voice input
        let voice_input = VoiceInput::new();

        // Create the encoding engine
        let (encoding_engine, sample_sender, encoded_receiver) =
            EncodingEngine::create(voice_input.get_sample_rate());

        // Initialize the engine
        return Self {
            voice_input: VoiceInput::new(),
            encoding_engine: encoding_engine,
        };
    }
}
