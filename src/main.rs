use std::{thread, time::Duration};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    StreamConfig,
};
use opusic_sys::{opus_encoder_create, OpusEncoder, OPUS_APPLICATION_VOIP};

fn main() {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("Default device not found");

    let default_config = device
        .default_input_config()
        .expect("Couldn't use the microphone");

    let config = StreamConfig {
        buffer_size: cpal::BufferSize::Fixed(4096),
        channels: default_config.channels(),
        sample_rate: default_config.sample_rate(),
    };

    let stream = match device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &_| println!("got data: {}", data.len()),
        move |err| {
            println!("error while listening: {}", err);
        },
        None,
    ) {
        Ok(stream) => stream,
        Err(err) => {
            println!("error while creating stream: {}", err);
            return;
        }
    };

    stream.play().unwrap();
    thread::sleep(Duration::from_millis(100));

    println!("begin recording");
}
