use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use opus::Decoder;
use tokio::sync::mpsc::{self, Receiver, Sender, UnboundedSender};

struct DecodingEngine {
    client_map: HashMap<String, DecodingTarget>,
}

struct DecodingTarget {
    id: String,
    decoder: Mutex<Decoder>,
    sender: Sender<Vec<f32>>,
}

struct DecodeData {
    id: String,
    packet: Vec<u8>,
}

impl DecodingEngine {
    // Create a new decoding engine (starts a blocking task for it as well)
    pub fn create() -> (Arc<Mutex<Self>>, UnboundedSender<DecodeData>) {
        // Create a new decoding engine
        let engine = Arc::new(Mutex::new(Self {
            client_map: HashMap::new(),
        }));

        // Create a new channel for the packets coming in
        let (sender, mut receiver) = mpsc::unbounded_channel();

        // Spawn a new task for decoding
        tokio::task::spawn_blocking({
            let engine = engine.clone();
            move || loop {
                let data: Option<DecodeData> = receiver.blocking_recv();
                if data.is_none() {
                    println!("decoder has been closed.");
                    return;
                }
                let data = data.expect("No data found");

                // Make sure the client with the specified id actually exists
                let engine = engine.lock().unwrap();
                if !engine.client_map.contains_key(&data.id) {
                    println!("client {} hasn't been added yet", data.id);
                    continue;
                }

                // Get the decoder from the client
                let client = engine.client_map.get(&data.id).unwrap();
                let mut decoder = client.decoder.lock().unwrap();

                // Decode the packet
                let mut output = [0f32; 2000];
                let size = decoder
                    .decode_float(&data.packet, &mut output, false)
                    .expect("Couldn't decode");

                // Pass the output on to the next step
                let (decoded, _) = output.split_at(size);
                client.sender.blocking_send(decoded.to_vec()).ok();
            }
        });

        return (engine, sender);
    }

    // Add a new target to the decoding engine
    pub fn add_target(&mut self, id: String, sample_rate: u32) -> Receiver<Vec<f32>> {
        // Create a new decoder for the sample rate
        let decoder =
            opus::Decoder::new(sample_rate, opus::Channels::Mono).expect("Couldn't create decoder");

        // Create a channel for the samples
        let (sender, receiver) = mpsc::channel(4);

        // Add the target to the decoder
        self.client_map.insert(
            id.clone(),
            DecodingTarget {
                id: id,
                decoder: Mutex::new(decoder),
                sender: sender,
            },
        );

        return receiver;
    }

    // Get a target from the decoding engine by its ID
    pub fn get_target(&self, id: &str) -> Option<&DecodingTarget> {
        self.client_map.get(id)
    }
}
