use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use cpal::traits::{HostTrait, StreamTrait};
use rodio::DeviceTrait;
use tokio::{
    sync::mpsc::{self, Receiver},
    time,
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
                    let input = input.lock().unwrap();
                    if input.channels == 1 {
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
                    let input = input.lock().unwrap();
                    input.channels
                };
                let callback = {
                    let input = input.clone();
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        // Make sure the output is not paused
                        {
                            let input = input.lock().unwrap();
                            if input.paused {
                                return;
                            }
                        }

                        if channels == 1 {
                            // Directly forward it when it's mono
                            sender.blocking_send(data.to_vec()).ok();
                        } else {
                            // Convert to mono audio when we're using 2 channels
                            let mono = data
                                .chunks(2)
                                .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
                                .collect();
                            sender.blocking_send(mono).ok();
                        }
                    }
                };

                // Start the audio stream
                let stream = {
                    let input = input.lock().unwrap();
                    input
                        .device
                        .build_input_stream(&stream_config, callback, err_fn, None)
                        .expect("Couldn't build stream")
                };

                stream.play().expect("Couldn't start stream");

                loop {
                    {
                        let input = input.lock().unwrap();
                        if input.stop {
                            break;
                        }
                    }

                    time::sleep(Duration::from_millis(100));
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
