use std::{collections::HashMap, sync::Arc, thread, time::Duration};

use jittr::JitterBuffer;
use rodio::{buffer::SamplesBuffer, OutputStream, OutputStreamHandle, Sink};
use tokio::{
    sync::{mpsc::Receiver, Mutex},
    time,
};

use super::DecodedPacket;

pub struct PlayingEngine {
    client_map: HashMap<String, Arc<Mutex<Client>>>,
    output_handle: Option<OutputStreamHandle>,
}

struct Client {
    id: String,
    sink: Sink,
    buffer: JitterBuffer<DecodedPacket, 8>,
    last_packet: Option<DecodedPacket>,
}

impl PlayingEngine {
    pub fn create() -> Arc<Mutex<PlayingEngine>> {
        let engine = Arc::new(Mutex::new(Self {
            client_map: HashMap::new(),
            output_handle: None,
        }));

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
                    thread::sleep(Duration::from_millis(100));
                    // TODO: Handle stop and device changes here
                }
            }
        });

        return engine;
    }

    // Add a new client to the playing engine
    pub fn add_target(&mut self, id: String, mut receiver: Receiver<DecodedPacket>) {
        // Create a sink for the thing
        let handle = self.output_handle.as_ref().unwrap();
        let sink = Sink::try_new(&handle).expect("Couldn't create sink");

        // Add the target to the decoder
        let client = Arc::new(Mutex::new(Client {
            id: id.clone(),
            sink: sink,
            buffer: JitterBuffer::new(),
            last_packet: None,
        }));
        self.client_map.insert(id, client.clone());

        // Start a new task that adds all the packets to the jitter buffer
        tokio::spawn({
            let client = client.clone();
            async move {
                loop {
                    let packet = receiver.recv().await;
                    if packet.is_none() {
                        return;
                    }

                    // Add the received packet to the jitter buffer
                    let mut client = client.lock().await;
                    client.buffer.push(packet.unwrap());
                }
            }
        });

        // Spawn a task for playing the packets at a consistent interval
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(20));
            loop {
                interval.tick().await;

                let mut client = client.lock().await;
                let packet = client.buffer.pop();
                if packet.is_none() {
                    if client.last_packet.is_none() {
                        continue;
                    }

                    // Append the last packet when a packet is dropped
                    let packet = client.last_packet.as_ref().unwrap();
                    client.sink.append(SamplesBuffer::new(
                        1,
                        packet.sample_rate,
                        packet.packet.clone(),
                    ));

                    client.last_packet = None;
                    continue;
                }

                // Append the packet from the jitter buffer
                let packet = packet.unwrap();
                client.sink.append(SamplesBuffer::new(
                    1,
                    packet.sample_rate,
                    packet.packet.clone(),
                ));

                // Set as the last packet played
                client.last_packet = Some(packet);
            }
        });
    }

    // Get a target from the decoding engine by its ID
    pub fn get_target(&self, id: &str) -> Option<&Arc<Mutex<Client>>> {
        self.client_map.get(id)
    }
}
