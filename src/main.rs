use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    println!("Hello, world!");

    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();

    println!(
        "using {} with device {}",
        host.id().name(),
        device.name().unwrap()
    );

    let config = device
        .default_input_config()
        .expect("Failed to get default input config");

    let mic_config = cpal::StreamConfig {
        buffer_size: cpal::BufferSize::Default,
        channels: config.channels(),
        sample_rate: config.sample_rate(),
    };

    let err_fn = move |err| eprintln!("error in cpal: {}", err);

    let stream = device
        .build_input_stream(&mic_config, move |data, _| handle_data(data), err_fn, None)
        .expect("Couldn't start input stream");

    stream.play().expect("Couldn't play stream");
    std::thread::sleep(Duration::from_secs(3));
}

fn handle_data(samples: &[f32]) {
    println!("received with length: {}", samples.len())
}
