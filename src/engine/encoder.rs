use std::sync::{Arc, Mutex};

use opus::Encoder;
use tokio::sync::mpsc::{self, Receiver};

pub struct EncodingEngine {
    encoder: Option<Mutex<Encoder>>,
}

impl EncodingEngine {
    // Start a new encoding engine
    pub fn create(
        sample_rate: u32,
        mut sample_receiver: Receiver<Vec<f32>>,
    ) -> (Arc<Mutex<Self>>, Receiver<Vec<u8>>) {
        // TODO: Use rubato to resample in case of sample rate not supported by Opus
        // Create a new Opus encoder for this encoding engine
        let encoder =
            opus::Encoder::new(sample_rate, opus::Channels::Mono, opus::Application::Voip)
                .expect("Couldn't create opus encoder");

        let engine = Arc::new(Mutex::new(Self {
            encoder: Some(Mutex::new(encoder)),
        }));

        // Create a channels for receiving the data and also sending back the encoded data
        let (encoded_sender, encoded_receiver) = mpsc::channel(4);

        // Spawn the encoding task
        tokio::task::spawn_blocking({
            let engine = engine.clone();
            move || loop {
                let sample: Option<Vec<f32>> = sample_receiver.blocking_recv();
                if sample.is_none() {
                    break;
                }

                // Get the encoder from the engine
                let engine = engine.lock().expect("Couldn't lock encoding engine mutex");
                if engine.encoder.is_none() {
                    break;
                }
                let encoder = engine.encoder.as_ref().unwrap();
                let mut coder = encoder.lock().unwrap();

                // Encode using Opus
                let mut output = [0u8; 2000];
                let output_size = coder
                    .encode_float(sample.unwrap().as_slice(), &mut output)
                    .expect("Couldn't encode");

                let (packet, _) = output.split_at(output_size);
                encoded_sender.blocking_send(packet.to_vec()).ok();
            }
        });

        return (engine.clone(), encoded_receiver);
    }
}
