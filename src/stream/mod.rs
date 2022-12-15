use crate::stream::resolution::VBANResolution;
use crate::stream::sample_rate::VBANSampleRate;
use crate::stream::stream_name::StreamName;
use deku::{DekuContainerWrite, DekuRead, DekuUpdate, DekuWrite};
use std::net::{ToSocketAddrs, UdpSocket};

pub mod resolution;
pub mod sample_rate;
pub mod stream_name;

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(magic = b"VBAN")]
pub struct VBANHeader {
    #[deku(bits = 3)]
    sub_proto: u8,
    #[deku(bits = 5)]
    rate: VBANSampleRate,
    sample_count: u8,
    channel_count: u8,
    #[deku(bits = 4)]
    codec: u8,
    #[deku(pad_bits_before = "1", bits = 3)]
    format_bit: VBANResolution,
    stream_name: StreamName,
    frame: u32,
}

pub fn setup_socket(ip: impl ToSocketAddrs) -> UdpSocket {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.connect(ip).unwrap();
    socket
}

pub fn generate_header(frame: u32, format_bit: VBANResolution, sample_count: u8) -> Vec<u8> {
    VBANHeader {
        rate: VBANSampleRate::Rate22050,
        sub_proto: 0,
        sample_count,
        channel_count: 0,
        codec: 0,
        format_bit,
        stream_name: StreamName::try_from("Stream1").unwrap(),
        frame,
    }
    .to_bytes()
    .unwrap()
}

pub fn generate_sin(frame_start: u32, buf: &mut Vec<u8>) {
    buf.reserve_exact(256);
    for idx in 0..256 {
        let p = (frame_start * 256 + idx) as f64 * 440. / 44100. * std::f64::consts::TAU;
        let e = ((p.sin() + 1.) * (u8::MAX / 2) as f64) as u8;
        buf.push(e);
    }
}
