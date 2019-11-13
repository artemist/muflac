use std::env::{args_os};
use std::path::Path;
use muflac::bitstream::BufferedBitstreamReader;
use muflac::header::{read_magic, read_metadata_block};
use muflac::metadata_types::MetadataBlock;
use muflac::error::Error;
use std::fs::File;

fn main() {
    let mut args_iter = args_os();
    args_iter.next();
    let filename = args_iter.next().unwrap();
    let file = Path::new(&filename);
    let mut stream = BufferedBitstreamReader::<File>::new(file)
        .unwrap_or_else(|e| panic!("Unable to create reader {:?}", e));

    let magic_result = read_magic(&mut stream);
    println!("magic: {:?}", magic_result);


    let mut block = read_metadata_block(&mut stream);
    println!("first block: {:?}", block);
    while !is_last(block) {
        block = read_metadata_block(&mut stream);
        println!("block: {:?}", block)
    }
}

fn is_last(block: Result<MetadataBlock, Error>) -> bool {
    if let Ok(blk) = block {
        blk.is_last
    } else {
        true
    }
}