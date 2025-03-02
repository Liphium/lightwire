use std::{sync::Arc, thread, time::Duration};

use cpal::traits::{HostTrait, StreamTrait};
use rodio::DeviceTrait;
use tokio::sync::{
    mpsc::{self, Receiver},
    Mutex,
};

pub struct VoiceInput {
    device: cpal::Device,
    channels: u16,
    sample_rate: u32,
    frame_size: u32,
    stop: bool,
    paused: bool,
}

impl VoiceInput {
    pub fn create() -> (Arc<Mutex<Self>>, Receiver<Vec<f32>>) {
        // Get the default microphone and stream config
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .expect("No default device found");
        let device_config: cpal::SupportedStreamConfig = device
            .default_input_config()
            .expect("Failed to get default input config");

        let input = Arc::new(Mutex::new(Self {
            device: device,
            channels: device_config.channels(),
            sample_rate: device_config.sample_rate().0,
            frame_size: device_config.sample_rate().0 / 50,
            stop: false,
            paused: true,
        }));

        // Create a new channel for sending the packets
        let (sender, receiver) = mpsc::channel(4);

        // Create a new task for handling all of the sending
        tokio::task::spawn_blocking({
            let input = input.clone(); // Clone for use in the task
            move || {
                // Create stream config for the device based on the channels
                let stream_config = {
                    let input = input.blocking_lock();
                    if input.channels == 1 {
                        println!(
                            "using mono, sample_rate={} frame_size={}",
                            input.sample_rate, input.frame_size
                        );
                        cpal::StreamConfig {
                            channels: 1,
                            sample_rate: cpal::SampleRate(input.sample_rate),
                            buffer_size: cpal::BufferSize::Fixed(input.frame_size),
                        }
                    } else {
                        cpal::StreamConfig {
                            channels: 2,
                            sample_rate: cpal::SampleRate(input.sample_rate),
                            buffer_size: cpal::BufferSize::Fixed(input.frame_size * 2), // Use double here because it will be turned into mono
                        }
                    }
                };

                // Error function for printing errors that happen during voice handling
                let err_fn = move |err| eprintln!("error in cpal: {}", err);

                // Process data if needed (e.g. convert stereo to mono)
                let channels = {
                    let input = input.blocking_lock();
                    input.channels
                };
                let callback = {
                    let input = input.clone();
                    let mut overflow_buffer = Vec::<f32>::new();
                    let frame_size = {
                        let input = input.blocking_lock();
                        input.frame_size
                    };
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        // Check if paused
                        {
                            let input = input.blocking_lock();
                            if input.paused {
                                return;
                            }
                        }

                        // Accumulate data
                        overflow_buffer.extend_from_slice(data);

                        // Dispatch complete frames
                        while overflow_buffer.len() >= frame_size as usize {
                            let packet: Vec<f32> =
                                overflow_buffer.drain(0..frame_size as usize).collect();
                            if channels == 1 {
                                sender.blocking_send(packet).ok();
                            } else {
                                let mono = packet.chunks(2).map(|c| (c[0] + c[1]) * 0.5).collect();
                                sender.blocking_send(mono).ok();
                            }
                        }
                    }
                };

                // Start the audio stream
                let stream = {
                    let input = input.blocking_lock();
                    input
                        .device
                        .build_input_stream(&stream_config, callback, err_fn, None)
                        .expect("Couldn't build stream")
                };

                stream.play().expect("Couldn't start stream");

                loop {
                    {
                        let input = input.blocking_lock();
                        if input.stop {
                            break;
                        }
                    }

                    thread::sleep(Duration::from_millis(100));
                }
            }
        });

        return (input, receiver);
    }

    pub fn get_sample_rate(&self) -> u32 {
        return self.sample_rate;
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn is_paused(&self) -> bool {
        return self.paused;
    }

    pub fn stop(&mut self) {
        self.stop = true;
    }
}
