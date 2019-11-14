use crate::error::Error;
use crate::bitstream::BitstreamReader;

pub(crate) fn read_rice(
    reader: &mut dyn BitstreamReader,
    encoding_parameter: u8
) -> Result<u32, Error> {
    let quotient = reader.read_unary(false)?;
    let remainder = reader.read_sized(encoding_parameter)? as u32;
    Ok((quotient << encoding_parameter as u32) | remainder)
}