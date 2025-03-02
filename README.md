# lightwire

Lightwire is an experimental audio packetizer to potentially power voice chat on Liphium in the future. It's currently being developed in Rust using these amazing libraries:

- [tokio](https://crates.io/crates/tokio) for creating threads and managing the general flow of data.
- [cpal](https://crates.io/crates/cpal) for getting microphone input.
- [rodio](https://crates.io/crates/rodio) for playing multiple audio streams to the speaker.
- [rubato](https://crates.io/crates/rubato) for resampling the audio for Opus.
- [opus-rs](https://crates.io/crates/opus) for decoding and encoding using Opus.
- [aec-rs](https://crates.io/crates/aec-rs) for echo cancellation and basic noise suppression.

Just a note here: Evaluate https://crates.io/crates/jittr for use in the player.
ANOTHER NOTE: You absolute dumbass need to decode the packets after the jitter buffer has done its job (opus needs to decode the packets in the correct order, it also has automatic loss concealment, use that instead of what you currently have maybe).
