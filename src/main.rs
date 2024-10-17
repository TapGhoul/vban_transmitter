use crate::stream::resolution::VBANResolution;
use crate::stream::stream_name::StreamName;
use crate::stream::try_parse_header;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{BufferSize, SampleRate, SizedSample, Stream, StreamConfig};
use ringbuf::traits::{Consumer, Observer, Producer, Split};
use ringbuf::HeapRb;
use std::env::args;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::process::exit;

mod stream;

fn main() {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 6980)).unwrap();
    let stream_name = StreamName::try_from("Stream1").unwrap();

    let _stream = {
        let ip: IpAddr = get_ip().parse().unwrap();
        let addr = SocketAddr::from((ip, 6980));
        println!("Sending to {addr} on {stream_name}");

        let stream_name = stream_name.clone();
        let socket = socket.try_clone().unwrap();
        let mut idx: u32 = 0;

        setup_mic(move |samples: &[i16]| {
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
                    stream_name.clone(),
                    idx,
                    VBANResolution::S16,
                    ((chunk.len() / sample_byte_len) - 1) as u8,
                );
                packet.extend(chunk);
                socket.send_to(&packet, addr).unwrap();
            }
        })
    };

    let (mut producer, mut consumer) = HeapRb::new(48000 * 256).split();

    let _speaker = {
        let mut is_warming_buffer = true;
        setup_speaker(move |data: &mut [i16]| {
            if is_warming_buffer && consumer.occupied_len() >= data.len() {
                println!("Buffer warmed!");
                is_warming_buffer = false;
            } else if !is_warming_buffer && consumer.occupied_len() < data.len() {
                println!("WARN: Buffer underrun");
                // Avoid a screech
                data.fill(0);
                is_warming_buffer = true;
            }

            if is_warming_buffer {
                return;
            }

            consumer.pop_slice(data);
        })
    };

    let mut buf = [0u8; 1436];
    loop {
        let (len, _) = socket.recv_from(&mut buf).unwrap();
        // TODO: Check for discontinuity of packets, we are using UDP here!
        let Some((sample_count, buf)) = try_parse_header(&stream_name, &buf[..len]) else {
            continue;
        };

        // TODO: check for overflow
        producer.push_iter(
            buf.chunks_exact(2)
                .take(sample_count as usize + 1)
                .map(|e| i16::from_le_bytes(e.try_into().unwrap())),
        );
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
    T: SizedSample,
{
    let host = cpal::default_host();
    let mic = host.default_input_device().unwrap();

    mic.build_input_stream(
        &StreamConfig {
            channels: 1,
            // This can probably be lower, but I haven't tested
            buffer_size: BufferSize::Fixed(512),
            sample_rate: SampleRate(22050),
        },
        move |data: &[T], _| cb(data),
        |err| panic!("{err:?}"),
        None,
    )
    .unwrap()
}

fn setup_speaker<T, D>(mut cb: D) -> Stream
where
    D: FnMut(&mut [T]) + Send + 'static,
    T: SizedSample,
{
    let host = cpal::default_host();
    let mic = host.default_input_device().unwrap();

    mic.build_output_stream(
        &StreamConfig {
            channels: 2,
            // Lowest I could get it without OS-level buffer underruns
            buffer_size: BufferSize::Fixed(512),
            sample_rate: SampleRate(48000),
        },
        move |data: &mut [T], _| cb(data),
        |err| panic!("{err:?}"),
        None,
    )
    .unwrap()
}
