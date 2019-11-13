use crate::bitstream::BitstreamReader;
use crate::error::Error;
use crate::metadata_types::{MetadataBlock, MetadataBlockData, MetadataBlockStreamInfo};

pub fn read_magic<T: BitstreamReader>(reader: &mut T) -> Result<(), Error> {
    let magic = &*reader.read_bytes(4)?;
    if magic == &b"fLaC"[..] {
        Ok(())
    } else {
        Err(Error::Content)
    }
}

pub fn read_metadata_block<T: BitstreamReader>(
    reader: &mut T,
) -> Result<MetadataBlock, Error> {
    let is_last = reader.read_bit()?;
    let block_type = reader.read_sized(7)? as u8;
    let length = reader.read_sized(24)? as u32;
    let data = match block_type {
        0 => MetadataBlockData::StreamInfo(read_stream_info_block(reader)?),
        1 => {
            reader.read_bytes(length as usize)?;
            MetadataBlockData::Padding
        }
        2 => MetadataBlockData::Application(reader.read_bytes(length as usize)?),
        3 => unimplemented!(),
        4 => unimplemented!(),
        5 => unimplemented!(),
        6 => unimplemented!(),
        127 => MetadataBlockData::Invalid,
        n => MetadataBlockData::Reserved(n),
    };

    Ok(MetadataBlock {
        is_last,
        content: data,
    })
}

pub fn read_stream_info_block<T: BitstreamReader>(
    reader: &mut T,
) -> Result<MetadataBlockStreamInfo, Error> {
    let min_block_size = reader.read_sized(16)? as u16;
    let max_block_size = reader.read_sized(16)? as u16;
    let min_frame_size = reader.read_sized(24)? as u32;
    let max_frame_size = reader.read_sized(24)? as u32;
    let sample_rate = reader.read_sized(20)? as u32;
    let num_channels = reader.read_sized(3)? as u8 + 1;
    let sample_depth = reader.read_sized(5)? as u8 + 1;
    let num_samples = reader.read_sized(36)? as u64;
    let decoded_checksum = reader.read_sized(128)?;

    Ok(MetadataBlockStreamInfo {
        min_block_size,
        max_block_size,
        min_frame_size,
        max_frame_size,
        sample_rate,
        num_channels,
        sample_depth,
        num_samples,
        decoded_checksum,
    })
}
