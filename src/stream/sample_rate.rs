use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::ctx::BitSize;
use deku::error::DekuError;
use deku::{DekuRead, DekuWrite};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryFrom;

#[derive(Debug, Clone, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum VBANSampleRate {
    Rate6000,
    Rate12000,
    Rate24000,
    Rate48000,
    Rate96000,
    Rate192000,
    Rate384000,

    Rate8000,
    Rate16000,
    Rate32000,
    Rate64000,
    Rate128000,
    Rate256000,
    Rate512000,

    Rate11025,
    Rate22050,
    Rate44100,
    Rate88200,
    Rate176400,
    Rate352800,
    Rate705600,
}

impl DekuRead<'_, BitSize> for VBANSampleRate {
    fn read(
        input: &BitSlice<u8, Msb0>,
        ctx: BitSize,
    ) -> Result<(&BitSlice<u8, Msb0>, Self), DekuError> {
        let (rest, value) = u8::read(input, ctx)?;
        let parsed = VBANSampleRate::try_from(value).map_err(|_| DekuError::IdVariantNotFound)?;

        Ok((rest, parsed))
    }
}

impl DekuWrite<BitSize> for VBANSampleRate {
    fn write(&self, output: &mut BitVec<u8, Msb0>, ctx: BitSize) -> Result<(), DekuError> {
        let v = self.clone() as u8;
        v.write(output, ctx)
    }
}
