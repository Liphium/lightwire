use std::{sync::Arc, time::Duration, u16};

use engine::Engine;
use jittr::JitterBuffer;
use tokio::{
    sync::Mutex,
    time::{self, interval},
};

pub mod engine;

#[derive(Clone)]
struct OpusPacket {
    pub seq: u16,
}

impl jittr::Packet for OpusPacket {
    fn sequence_number(&self) -> u16 {
        return self.seq;
    }
}

#[tokio::main]
async fn main() {
    Engine::create();

    /*
    let buffer = Arc::new(Mutex::new(JitterBuffer::<OpusPacket, 8>::new()));

    let mut playback = interval(Duration::from_millis(500));

    tokio::spawn({
        let buffer = buffer.clone();
        async move {
            {
                let mut buffer = buffer.lock().await;
                buffer.push(OpusPacket { seq: u16::MAX });
                buffer.push(OpusPacket { seq: 0 });
                buffer.push(OpusPacket { seq: 1 });
                buffer.push(OpusPacket { seq: 2 });
                buffer.push(OpusPacket { seq: 4 });
            }
        }
    });

    time::sleep(Duration::from_millis(10)).await;

    loop {
        playback.tick().await;

        let mut buffer = buffer.lock().await;
        let packet = buffer.pop();
        if packet.is_none() {
            println!("failure");
        } else {
            println!("Now playing: {}", packet.unwrap().seq);
        }
    }

    */
}

/*
#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();

    let mut encoder = opus::Encoder::new(48000, opus::Channels::Mono, opus::Application::Voip)
        .expect("Couldn't create encoder");
    encoder
        .set_bitrate(opus::Bitrate::Bits(64000))
        .expect("Couldn't set bitrate");

    println!(
        "using {} with device {}",
        host.id().name(),
        device.name().unwrap()
    );

    let config: cpal::SupportedStreamConfig = device
        .default_input_config()
        .expect("Failed to get default input config");

    let frame_size = (config.sample_rate() / 50).0;
    println!("using frame size: {}", frame_size);
    let mic_config = cpal::StreamConfig {
        buffer_size: cpal::BufferSize::Fixed(frame_size),
        channels: config.channels(),
        sample_rate: config.sample_rate(),
    };

    let err_fn = move |err| eprintln!("error in cpal: {}", err);

    let buffer = Arc::new(Mutex::new(Vec::<Vec<u8>>::new()));

    let stream = device
        .build_input_stream(
            &mic_config,
            {
                let buffer = buffer.clone();
                move |data, _| {
                    buffer.clear_poison();
                    let mut buffer = buffer.lock().unwrap();
                    handle_data(data, &mut buffer, &mut encoder);
                }
            },
            err_fn,
            None,
        )
        .expect("Couldn't start input stream");

    stream.play().expect("Couldn't play stream");
    std::thread::sleep(Duration::from_secs(3));

    // Create a new decoder
    let mut decoder =
        opus::Decoder::new(48000, opus::Channels::Mono).expect("Couldn't create decoder");

    // Decode all the packets again
    buffer.clear_poison();
    let mut samples = Vec::<f32>::new();
    let buf = buffer.lock().unwrap().clone();
    for packet in buf {
        let mut output = [0f32; 10000];
        let amount = decoder
            .decode_float(packet.as_slice(), &mut output, false)
            .expect("Couldn't decode");
        println!("decoded {}", amount);
        let (sample, _) = output.split_at(amount);
        samples.append(&mut sample.to_vec());
    }

    {
        let (_stream, stream_handle) =
            OutputStream::try_default().expect("Failed to get default output stream");
        let sink = Sink::try_new(&stream_handle).expect("Failed to create sink");

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        let source = SamplesBuffer::new(channels, sample_rate, samples.clone());
        sink.append(source);
        sink.sleep_until_end();
    }
}

fn handle_data(samples: &[f32], buffer: &mut Vec<Vec<u8>>, encoder: &mut opus::Encoder) {
    // Encode the samples using Opus
    let encoded = encoder.encode_vec_float(samples, 10000).unwrap();

    // Append the samples to the buffer
    buffer.push(encoded);
}
*/
