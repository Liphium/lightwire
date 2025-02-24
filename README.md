# lightwire
Lightwire is an experimental audio packetizer to potentially power voice chat on Liphium in the future. It's currently being developed in Rust using these amazing libraries:
- [tokio](https://crates.io/crates/tokio) for creating threads and managing the general flow of data.
- [cpal](https://crates.io/crates/cpal) for getting microphone input.
- [rodio](https://crates.io/crates/rodio) for playing multiple audio streams to the speaker.
- [rubato](https://crates.io/crates/rubato) for resampling the audio for Opus.
- [opus-rs](https://crates.io/crates/opus) for decoding and encoding using Opus.
