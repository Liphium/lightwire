use std::{collections::HashMap, sync::Arc, time::Duration};

use jittr::JitterBuffer;
use opus::Decoder;
use rodio::{buffer::SamplesBuffer, OutputStream, OutputStreamHandle, Sink};
use tokio::{
    sync::{
        mpsc::{self, UnboundedSender},
        Mutex,
    },
    time,
};

use super::AudioPacket;

pub struct PlayingEngine {
    client_map: HashMap<String, Arc<Mutex<Client>>>,
    output_handle: Option<OutputStreamHandle>,
}

struct Client {
    id: String,
    sink: Sink,
    buffer: JitterBuffer<AudioPacket, 8>,
    decoder: Option<Decoder>,
}

impl PlayingEngine {
    pub fn create() -> (Arc<Mutex<PlayingEngine>>, UnboundedSender<AudioPacket>) {
        let engine = Arc::new(Mutex::new(Self {
            client_map: HashMap::new(),
            output_handle: None,
        }));

        // Create a new channel for the packets coming in
        let (sender, mut receiver) = mpsc::unbounded_channel();

        tokio::task::spawn_blocking({
            let engine = engine.clone();
            move || {
                // Stream needs to be created on the main thread
                let (_stream, stream_handle) =
                    OutputStream::try_default().expect("Failed to get default output stream");
                {
                    let mut engine_lock = engine.blocking_lock();
                    engine_lock.output_handle = Some(stream_handle);
                }

                loop {
                    // Listen for new audio packets
                    let data: Option<AudioPacket> = receiver.blocking_recv();
                    if data.is_none() {
                        println!("playing engine has been closed.");
                        return;
                    }
                    let data = data.expect("No data found");

                    // Make sure the client with the specified id actually exists
                    let engine = engine.blocking_lock();
                    let client_id = data
                        .id
                        .as_ref()
                        .expect("No client id in packet for decoding, can't decode this");
                    if !engine.client_map.contains_key(client_id) {
                        println!("client {} hasn't been added yet", client_id);
                        continue;
                    }

                    // Add the packet to the jitter buffer of the client
                    let client = engine
                        .client_map
                        .get(client_id)
                        .expect("Not found even though key exists, wtf");
                    let mut client = client.blocking_lock();
                    client.buffer.push(data);
                }
            }
        });

        return (engine, sender);
    }

    // Add a new client to the playing engine
    pub fn add_target(&mut self, id: String) {
        // Create a sink for the thing
        let handle = self.output_handle.as_ref().unwrap();
        let sink = Sink::try_new(&handle).expect("Couldn't create sink");

        // Add the target to the playing engine
        let client = Arc::new(Mutex::new(Client {
            id: id.clone(),
            sink: sink,
            buffer: JitterBuffer::new(),
            decoder: None,
        }));
        self.client_map.insert(id, client.clone());

        // Spawn a task for playing the packets at a consistent interval
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(20));
            loop {
                interval.tick().await;

                let mut client = client.lock().await;
                let packet = client.buffer.pop();
                if packet.is_none() {
                    // TODO: Use decoder for loss concealment in case there
                    continue;
                }
                let packet = packet.unwrap();

                // Create a decoder in case there isn't one
                if client.decoder.is_none() {
                    client.decoder = Some(
                        opus::Decoder::new(packet.sample_rate, opus::Channels::Mono)
                            .expect("Couldn't create decoder"),
                    );
                }

                // Decode the packet
                let decoder = client.decoder.as_mut().expect("Decoder not found, wtf");
                let mut decoded = [0f32; 2000];
                let frame_size = decoder
                    .decode_float(&packet.packet, &mut decoded, false)
                    .expect("Couldn't decode packet");
                let (decoded, _) = decoded.split_at(frame_size);

                // Play the packet using the sink
                client
                    .sink
                    .append(SamplesBuffer::new(1, packet.sample_rate, decoded));
            }
        });
    }

    // Get a target from the decoding engine by its ID
    pub fn get_target(&self, id: &str) -> Option<&Arc<Mutex<Client>>> {
        self.client_map.get(id)
    }
}
