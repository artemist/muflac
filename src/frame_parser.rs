use crate::bitstream::BitstreamReader;
use crate::error::Error;
use crate::frame_types::{
    ChannelAssignment, ConstantSubframe, FixedSubframe, Frame, LPCSubframe, RICEPartition,
    Residual, Subframe, SubframeData, VerbatimSubframe,
};
use crate::metadata_types::MetadataBlockStreamInfo;
use std::str::FromStr;
use crate::rice::read_rice;

pub fn read_frame(
    reader: &mut dyn BitstreamReader,
    stream_info: &MetadataBlockStreamInfo,
) -> Result<Frame, Error> {
    let sync_code = reader.read_unsigned(14)?;
    if sync_code != 0b11_1111_1111_1110 {
        return Err(Error::Content);
    }

    if reader.read_bit()? {
        return Err(Error::Reserved);
    }

    let is_variable = reader.read_bit()?;
    let block_size_raw = reader.read_unsigned(4)? as u8;
    let sample_rate_raw = reader.read_unsigned(4)? as u8;
    let channel_assignment_raw: u8 = reader.read_unsigned(4)? as u8;
    let sample_depth_raw = reader.read_unsigned(3)? as u8;

    if reader.read_bit()? {
        return Err(Error::Reserved);
    }

    let frame_or_sample_number =
        u64::from_str(&*reader.read_utf8(if is_variable { 5 } else { 4 })?).ok();

    let block_size = match block_size_raw {
        0b0000 => Err(Error::Reserved),
        0b0001 => Ok(192),
        0b0110 => Ok(reader.read_unsigned(8)? as u32 + 1),
        0b0111 => Ok(reader.read_unsigned(16)? as u32 + 1),
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
        0b1100 => Ok(reader.read_unsigned(8)? as u32),
        0b1101 => Ok(reader.read_unsigned(16)? as u32),
        0b1110 => Ok(reader.read_unsigned(16)? as u32 * 10),
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
        _ => unreachable!(),
    }?;

    let header_crc = reader.read_unsigned(8)? as u8;

    let mut subframes = Vec::new();
    for i in 0..num_channels {
        subframes.push(read_subframe(reader, sample_depth, block_size)?);
    }

    let overall_crc = reader.read_unsigned(16)? as u16;

    Ok(Frame {
        is_variable,
        block_size,
        sample_rate,
        num_channels,
        channel_assignment,
        sample_depth,
        frame_or_sample_number,
        header_crc,
        subframes: subframes.into_boxed_slice(),
        overall_crc,
    })
}

fn read_subframe(
    reader: &mut dyn BitstreamReader,
    sample_depth: u8,
    block_size: u32,
) -> Result<Subframe, Error> {
    if reader.read_bit()? {
        return Err(Error::Content);
    }

    let subframe_type = reader.read_unsigned(6)? as u8;

    let wasted_bits = if reader.read_bit()? {
        reader.read_unary(false)? as u8 + 1
    } else {
        0
    };

    let data = match subframe_type {
        0b00000 => Ok(SubframeData::Constant(read_constant_subframe(
            reader,
            sample_depth,
        )?)),
        0b00001 => Ok(SubframeData::Verbatim(read_verbatim_subframe(
            reader,
            sample_depth,
            block_size,
        )?)),
        x if x >= 0b00_0010 && x <= 0b00_0011 => Err(Error::Reserved),
        x if x >= 0b00_0100 && x <= 0b00_0111 => Err(Error::Reserved),
        x if x >= 0b00_1000 && x <= 0b00_1100 => Ok(SubframeData::Fixed(read_fixed_subframe(
            reader,
            sample_depth,
            block_size,
            wasted_bits,
            x & 0b111,
        )?)),
        x if x >= 0b00_1101 && x <= 0b00_1111 => Err(Error::Reserved),
        x if x >= 0b01_0000 && x <= 0b01_1111 => Err(Error::Reserved),
        x if x >= 0b10_0000 && x <= 0b11_1111 => Ok(SubframeData::LPC(read_lpc_subframe(
            reader,
            sample_depth,
            block_size,
            wasted_bits,
            dbg!(x & 0b01_1111 + 1),
        )?)),
        _ => unreachable!(),
    }?;

    Ok(Subframe { wasted_bits, data })
}

fn read_constant_subframe(
    reader: &mut dyn BitstreamReader,
    sample_depth: u8,
) -> Result<ConstantSubframe, Error> {
    Ok(ConstantSubframe {
        content: reader.read_signed(sample_depth)? as i32,
    })
}

fn read_verbatim_subframe(
    reader: &mut dyn BitstreamReader,
    sample_depth: u8,
    block_size: u32,
) -> Result<VerbatimSubframe, Error> {
    let mut data = Vec::new();

    for _ in 0..block_size {
        data.push(reader.read_signed(sample_depth)? as i32);
    }

    Ok(VerbatimSubframe {
        content: data.into_boxed_slice(),
    })
}

fn read_fixed_subframe(
    reader: &mut dyn BitstreamReader,
    sample_depth: u8,
    block_size: u32,
    wasted_bits: u8,
    order: u8,
) -> Result<FixedSubframe, Error> {
    let mut warmup = Vec::new();

    for _ in 0..order {
        warmup.push(reader.read_signed(sample_depth)? as i32)
    }

    let residual = read_residual(reader, block_size, order)?;

    Ok(FixedSubframe {
        order,
        warmup: warmup.into_boxed_slice(),
        residual,
    })
}

fn read_lpc_subframe(
    reader: &mut dyn BitstreamReader,
    sample_depth: u8,
    block_size: u32,
    wasted_bits: u8,
    predictor_order: u8,
) -> Result<LPCSubframe, Error> {
    let mut warmup = Vec::new();

    for _ in 0..predictor_order {
        warmup.push(reader.read_signed(sample_depth)? as i32)
    }

    dbg!(&warmup);

    let coefficient_precision = reader.read_unsigned(4)? as u8 + 1;
    if coefficient_precision == 16 {
        return Err(Error::Reserved)
    }

    let shift = reader.read_signed(5)? as i8;

    let mut coefficients = Vec::new();

    let required_shift = 16 - coefficient_precision;
    for _ in 0..predictor_order {
        let coefficient = reader.read_signed(coefficient_precision)? as i16;
        coefficients.push(coefficient);
    }

    let residual = read_residual(reader, block_size, predictor_order)?;

    Err(Error::TooLong)
//    Ok(LPCSubframe {
//        order: predictor_order,
//        warmup: warmup.into_boxed_slice(),
//        coefficient_precision,
//        shift,
//        coefficients: coefficients.into_boxed_slice(),
//        residual
//    })
}

fn read_residual(reader: &mut dyn BitstreamReader, block_size: u32, predictor_order: u8) -> Result<Residual, Error> {
    let rice_type = reader.read_unsigned(2)? as u8;
    if rice_type >= 0b10 {
        return Err(Error::Reserved);
    }
    let parameter_size = 4 + rice_type;
    let partition_order = reader.read_unsigned(4)? as u8;

    let mut partitions = Vec::new();

    for idx in 0..(1u16 << partition_order as u16) {
        partitions.push(read_rice_partition(
            reader,
            parameter_size,
            block_size,
            partition_order,
            predictor_order,
            idx
        )?);
    }

    Ok(Residual {
        parameter_size,
        order: partition_order,
        partitions: partitions.into_boxed_slice(),
    })
}

fn read_rice_partition(
    reader: &mut dyn BitstreamReader,
    parameter_size: u8,
    block_size: u32,
    partition_order: u8,
    predictor_order: u8,
    idx: u16,
) -> Result<RICEPartition, Error> {
    let num_samples = if idx == 0 {
        (block_size >> partition_order as u32) - predictor_order as u32
    } else {
        block_size >> partition_order as u32
    };

    let encoding_parameter = reader.read_unsigned(parameter_size)? as u8;

    let mut residual = Vec::new();
    if encoding_parameter == 1u8 << parameter_size - 1 {
        let residual_size = reader.read_unsigned(5)? as u8;
        // raw encoding
        for _ in 0..num_samples {
            residual.push(reader.read_signed(residual_size)? as i32);
        }
    } else {
        for _ in 0..num_samples {
            residual.push(read_rice(reader, encoding_parameter)?);
        }
    }

    Ok(RICEPartition {
        encoding_parameter,
        residual: residual.into_boxed_slice()
    })
}
