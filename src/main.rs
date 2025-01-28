use cpal::traits::{DeviceTrait, HostTrait};

fn main() {
    println!("Hello, world!");

    println!("Supported hosts: {:?}", cpal::ALL_HOSTS);
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    println!("using device: {:?}", device.name().unwrap());
}
