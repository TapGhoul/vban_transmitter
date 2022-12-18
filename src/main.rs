use crate::stream::resolution::VBANResolution;
use crate::stream::setup_socket;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{BufferSize, Sample, SampleRate, Stream, StreamConfig};
use std::env::args;
use std::mem::size_of_val;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

mod stream;

fn main() {
    let ip = get_ip();

    let socket = setup_socket(format!("{ip}:6980"));
    println!("Sending to {} on Stream1", socket.peer_addr().unwrap());
    let mut idx: u32 = 0;

    let _stream = setup_mic(move |samples: &[i16]| {
        let bytes: Vec<u8> = samples.iter().flat_map(|e| e.to_le_bytes()).collect();
        let sample_byte_len: usize = size_of_val(&samples[0]);
        // Max bytes: 1436
        // Max samples: 256
        let chunk_size = 1436.min(256 * sample_byte_len);

        for chunk in bytes.chunks(chunk_size) {
            let sample_count = chunk.len();
            if sample_count == 0 {
                return;
            }

            idx = idx.wrapping_add(1);
            let mut packet = stream::generate_header(
                idx,
                VBANResolution::S16,
                ((chunk.len() / sample_byte_len) - 1) as u8,
            );
            packet.extend(chunk);
            socket.send(&packet).unwrap();
        }
    });

    loop {
        sleep(Duration::from_secs(30));
    }
}

fn get_ip() -> String {
    let mut args = args();
    let bin = args.next().unwrap();
    let ip = args.next();
    match ip {
        Some(e) => e,
        None => {
            println!("Usage: {bin} <ip>");
            exit(-1);
        }
    }
}

fn setup_mic<T, D>(mut cb: D) -> Stream
where
    D: FnMut(&[T]) + Send + 'static,
    T: Sample,
{
    let host = cpal::default_host();
    let mic = host.default_input_device().unwrap();
    let speaker = host.default_output_device().unwrap();

    mic.build_input_stream(
        &StreamConfig {
            channels: 1,
            buffer_size: BufferSize::Default,
            sample_rate: SampleRate(22050),
        },
        move |data: &[T], _| cb(data),
        |err| panic!("{err:?}"),
    )
    .unwrap()
}
