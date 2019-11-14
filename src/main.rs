use muflac::bitstream::BufferedBitstreamReader;
use muflac::block_parser::{read_magic, read_metadata_block};
use muflac::error::Error;
use muflac::metadata_types::{MetadataBlock, MetadataBlockData};
use std::env::args_os;
use std::fs::File;
use std::path::Path;
use muflac::frame_parser::read_frame;

fn main() {
    let mut args_iter = args_os();
    args_iter.next();
    let filename = args_iter.next().unwrap();
    let file = Path::new(&filename);
    let mut stream = BufferedBitstreamReader::<File>::open(file)
        .unwrap_or_else(|e| panic!("Unable to create reader {:?}", e));

    let magic_result = read_magic(&mut stream);
    println!("magic: {:?}", magic_result);

    let block = &read_metadata_block(&mut stream);
    println!("streaminfo block: {:?}", block);
    let stream_info = match &block.as_ref().unwrap().content {
        MetadataBlockData::StreamInfo(si) => Some(si),
        _ => None
    }.unwrap();

    let mut finished = is_last(block);
    while !finished {
        let block = &read_metadata_block(&mut stream);
        println!("block: {:?}", block);
        finished = is_last(block);
    }

    let frame = read_frame(&mut stream, stream_info);
    println!("first frame: {:?}", frame)
}

fn is_last(block: &Result<MetadataBlock, Error>) -> bool {
    if let Ok(blk) = block.as_ref() {
        blk.is_last
    } else {
        true
    }
}
