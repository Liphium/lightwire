use cpal::traits::{HostTrait, StreamTrait};
use rodio::DeviceTrait;

pub struct VoiceInput {
    device: cpal::Device,
    channels: u16,
    sample_rate: u32,
    frame_size: u32,
}

impl VoiceInput {
    // Create a new voice input, initializes with the default device
    pub fn new() -> Self {
        // Get the default microphone and stream config
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .expect("No default device found");
        let device_config: cpal::SupportedStreamConfig = device
            .default_input_config()
            .expect("Failed to get default input config");

        // Return a new voice input with the sample rate of the device
        return Self {
            device: device,
            channels: device_config.channels(),
            sample_rate: device_config.sample_rate().0,
            frame_size: device_config.sample_rate().0 / 50,
        };
    }

    // Start sending samples through the channel
    pub fn start<F>(&self, mut receiver: F)
    where
        F: FnMut(&[f32]) + Send + 'static,
    {
        // Create stream config for the device based on the channels
        let stream_config = if self.channels == 1 {
            cpal::StreamConfig {
                channels: 1,
                sample_rate: cpal::SampleRate(self.sample_rate),
                buffer_size: cpal::BufferSize::Fixed(self.frame_size),
            }
        } else {
            cpal::StreamConfig {
                channels: 2,
                sample_rate: cpal::SampleRate(self.sample_rate),
                buffer_size: cpal::BufferSize::Fixed(self.frame_size * 2), // Use double here because it will be turned into mono
            }
        };

        // Error function for printing errors that happen during voice handling
        let err_fn = move |err| eprintln!("error in cpal: {}", err);

        // Process data if needed (e.g. convert stereo to mono)
        let channels = self.channels;
        let callback = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            if channels == 1 {
                // Directly forward it when it's mono
                receiver(data);
            } else {
                // Convert to mono audio when we're using 2 channels
                let mono: Vec<f32> = data
                    .chunks(2)
                    .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
                    .collect();
                receiver(&mono);
            }
        };

        // Start the audio stream
        let stream = self
            .device
            .build_input_stream(&stream_config, callback, err_fn, None)
            .expect("Couldn't build stream");

        stream.play().expect("Couldn't start stream");
    }

    pub fn get_sample_rate(&self) -> u32 {
        return self.sample_rate;
    }

    pub fn get_frame_size(&self) -> u32 {
        return self.frame_size;
    }
}
