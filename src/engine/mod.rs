use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use encoder::EncodingEngine;
use rodio::{buffer::SamplesBuffer, OutputStream, Sink};
use voice::VoiceInput;

mod encoder;
mod voice;

pub struct Engine {
    voice_input: Arc<Mutex<VoiceInput>>,
    encoding_engine: Arc<Mutex<EncodingEngine>>,
}

impl Engine {
    pub fn create() -> Self {
        // Create the voice input
        let (voice_input, receiver) = VoiceInput::create();

        // Create the encoding engine
        let (encoding_engine, mut encoded_receiver) = {
            let voice_input = voice_input.lock().unwrap();
            EncodingEngine::create(voice_input.get_sample_rate(), receiver)
        };

        // Start the voice listener
        {
            let mut input = voice_input.lock().unwrap();
            input.set_paused(false);
        }

        // Spawn a task for testing (for now)
        let sample_rate = {
            let voice_input = voice_input.lock().unwrap();
            voice_input.get_sample_rate()
        };
        tokio::task::spawn_blocking(move || {
            let mut decoder =
                opus::Decoder::new(48000, opus::Channels::Mono).expect("Couldn't create decoder");

            let (_stream, stream_handle) =
                OutputStream::try_default().expect("Failed to get default output stream");
            let sink = Sink::try_new(&stream_handle).expect("Failed to create sink");

            // Decode all the packets
            loop {
                // Listen for new packets
                let encoded_sample = encoded_receiver.blocking_recv();
                if encoded_sample.is_none() {
                    break;
                }

                // Decode the packet
                let mut output = [0f32; 2000];
                let amount = decoder
                    .decode_float(encoded_sample.unwrap().as_slice(), &mut output, false)
                    .expect("Couldn't decode");
                println!("decoded {}", amount);
                let (sample, _) = output.split_at(amount);

                let source = SamplesBuffer::new(1, sample_rate, sample);
                sink.append(source);
            }

            sink.sleep_until_end();
        });

        thread::sleep(Duration::from_secs(3));
        {
            let mut voice_input_ref = voice_input.lock().unwrap();
            voice_input_ref.stop();
        }
        thread::sleep(Duration::from_secs(6));

        // Initialize the engine
        return Self {
            voice_input: voice_input,
            encoding_engine: encoding_engine,
        };
    }
}
