# lightwire

The repository for lightwire has been moved to the [main Liphium app repository](https://github.com/Liphium/chat_interface) where it'll be shipped as part of the chat app. You can find it in the lightwire folder over there.

Lightwire is an experimental audio packetizer to potentially power voice chat on Liphium in the future. It's currently being developed in Rust using these amazing libraries:

- [tokio](https://crates.io/crates/tokio) for creating threads and managing the general flow of data.
- [cpal](https://crates.io/crates/cpal) for getting microphone input.
- [rodio](https://crates.io/crates/rodio) for playing multiple audio streams to the speaker.
- [rubato](https://crates.io/crates/rubato) for resampling the audio for Opus.
- [opus-rs](https://crates.io/crates/opus) for decoding and encoding using Opus.
- [aec-rs](https://crates.io/crates/aec-rs) for echo cancellation and basic noise suppression.

## To-Do

- [x] Fix the bug with decoding and playing being separate and that causing Opus to not be decoded properly
- [x] Evaluate https://crates.io/crates/jittr for use in the player (already part of the implementation c:).
- [ ] Write a decoder and encoder for a lightwire packet format
- [ ] Test the audio engine with an actual stream of audio
- [ ] Add device switching
- [ ] Add resampling to the nearest Opus-compatible sample rate (using rubato)
- [ ] Add noise cancelling using RNNoise bindings (that I'll probably have to write myself or look at https://github.com/jneem/nnnoiseless)
