use crate::error::Error;
use crate::bitstream::BitstreamReader;

pub(crate) fn read_rice(
    reader: &mut dyn BitstreamReader,
    encoding_parameter: u8
) -> Result<i32, Error> {
    let quotient = reader.read_unary(false)?;
    let remainder = reader.read_unsigned(encoding_parameter)? as u32;
    let raw = (quotient << encoding_parameter as u32) | remainder;
    let is_negative = raw % 2 == 1;
    Ok(if is_negative {
        -((raw / 2) as i32) - 1
    } else {
        (raw / 2) as i32
    })
}