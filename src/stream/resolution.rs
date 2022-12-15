use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::ctx::BitSize;
use deku::error::DekuError;
use deku::{DekuRead, DekuWrite};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryFrom;

#[derive(Debug, Clone, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum VBANResolution {
    U8,
    S16,
    S24,
    S32,
    F32,
    F64,
    S12,
    S10,
}

impl DekuRead<'_, BitSize> for VBANResolution {
    fn read(
        input: &BitSlice<u8, Msb0>,
        ctx: BitSize,
    ) -> Result<(&BitSlice<u8, Msb0>, Self), DekuError> {
        let (rest, value) = u8::read(input, ctx)?;
        let parsed = VBANResolution::try_from(value).map_err(|_| DekuError::IdVariantNotFound)?;

        Ok((rest, parsed))
    }
}

impl DekuWrite<BitSize> for VBANResolution {
    fn write(&self, output: &mut BitVec<u8, Msb0>, ctx: BitSize) -> Result<(), DekuError> {
        let v = self.clone() as u8;
        v.write(output, ctx)
    }
}
