use crate::bitstream::BitstreamReader;
use crate::error::Error;
use crate::frame_types::{ChannelAssignment, Frame, Subframe};
use crate::metadata_types::MetadataBlockStreamInfo;
use std::str::FromStr;

pub fn read_frame(
    reader: &mut dyn BitstreamReader,
    stream_info: &MetadataBlockStreamInfo,
) -> Result<Frame, Error> {
    let sync_code = reader.read_sized(14)?;
    if sync_code != 0b11_1111_1111_1110 {
        return Err(Error::Content);
    }

    if reader.read_bit()? {
        return Err(Error::Reserved);
    }

    let is_variable = reader.read_bit()?;
    let block_size_raw = reader.read_sized(4)? as u8;
    let sample_rate_raw = reader.read_sized(4)? as u8;
    let channel_assignment_raw: u8 = reader.read_sized(4)? as u8;
    let sample_depth_raw = reader.read_sized(3)? as u8;

    if reader.read_bit()? {
        return Err(Error::Reserved);
    }

    let frame_or_sample_number =
        u64::from_str(&*reader.read_utf8(if is_variable { 5 } else { 4 })?).ok();

    let block_size = match block_size_raw {
        0b0000 => Err(Error::Reserved),
        0b0001 => Ok(192),
        0b0110 => Ok(reader.read_sized(8)? as u32 + 1),
        0b0111 => Ok(reader.read_sized(16)? as u32 + 1),
        x if x >= 0b0010 && x <= 0b0101 => Ok(576 << (x - 2)),
        x if x >= 0b1000 && x <= 0b1111 => Ok(256 << (x - 8)),
        _ => unreachable!(),
    }?;

    let sample_rate = match sample_rate_raw {
        0b0000 => Ok(stream_info.sample_rate),
        0b0001 => Ok(88_200),
        0b0010 => Ok(176_400),
        0b0011 => Ok(192_000),
        0b0100 => Ok(8_000),
        0b0101 => Ok(16_000),
        0b0110 => Ok(22_050),
        0b0111 => Ok(24_000),
        0b1000 => Ok(32_000),
        0b1001 => Ok(44_100),
        0b1010 => Ok(48_000),
        0b1011 => Ok(96_000),
        0b1100 => Ok(reader.read_sized(8)? as u32),
        0b1101 => Ok(reader.read_sized(16)? as u32),
        0b1110 => Ok(reader.read_sized(16)? as u32 * 10),
        0b1111 => Err(Error::Reserved),
        _ => unreachable!(),
    }?;

    let channel_assignment = match channel_assignment_raw {
        0b1000 => Ok(ChannelAssignment::LeftSide),
        0b1001 => Ok(ChannelAssignment::RightSide),
        0b1010 => Ok(ChannelAssignment::MidSide),
        x if x <= 0b0111 => Ok(ChannelAssignment::Direct),
        x if x >= 0b1011 && x <= 0b1111 => Err(Error::Reserved),
        _ => unreachable!(),
    }?;


    let num_channels = if channel_assignment_raw <= 0b111 {
        channel_assignment_raw + 1
    } else {
        2
    };

    let sample_depth = match sample_depth_raw {
        0b000 => Ok(stream_info.sample_depth),
        0b001 => Ok(8),
        0b010 => Ok(12),
        0b011 => Err(Error::Reserved),
        0b100 => Ok(16),
        0b101 => Ok(24),
        0b110 => Ok(32),
        0b111 => Err(Error::Reserved),
        _ => unreachable!()
    }?;

    let header_crc = reader.read_sized(8)? as u8;

    Ok(Frame {
        is_variable,
        block_size,
        sample_rate,
        num_channels,
        channel_assignment,
        sample_depth,
        frame_or_sample_number,
        header_crc,
        subframes: Box::new([])
    })
}

fn read_subframe(reader: &mut dyn BitstreamReader, sample_depth: u8, block_size: u32) -> Result<Subframe, Error> {
    unimplemented!()
}