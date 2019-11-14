use crate::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub trait BitstreamReader {
    fn read_bit(&mut self) -> Result<bool, Error>;
    fn read_bytes(&mut self, num_bytes: usize) -> Result<Box<[u8]>, Error>;
    fn read_bits(&mut self, num_bits: usize) -> Result<Box<[bool]>, Error>;
    fn read_unary(&mut self, start: bool) -> Result<u32, Error>;
    fn read_utf8(&mut self, max_length: isize) -> Result<Box<str>, Error>;
    fn read_sized(&mut self, num_bits: u8) -> Result<u128, Error>;
    fn get_total_position(&self) -> usize;
}

pub struct BufferedBitstreamReader<T: Read> {
    reader: BufReader<T>,
    total_position: usize,
    curr_byte: u8,
    bit_idx: u8,
}

impl BufferedBitstreamReader<File> {
    pub fn open(filename: &Path) -> Result<Self, Error> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        Ok(BufferedBitstreamReader {
            reader,
            total_position: 0,
            curr_byte: 0,
            bit_idx: 8,
        })
    }
}

impl<T: Read> BufferedBitstreamReader<T> {
    #[inline(always)]
    fn refill_if_necessary(&mut self) -> Result<(), Error> {
        if self.bit_idx == 8 {
            let mut buf = [0u8; 1];
            self.reader.read_exact(&mut buf)?;
            self.curr_byte = buf[0];
            self.bit_idx = 0;
        }
        Ok(())
    }
}

impl<T: Read> BitstreamReader for BufferedBitstreamReader<T> {
    fn read_bit(&mut self) -> Result<bool, Error> {
        self.refill_if_necessary()?;
        let result = (self.curr_byte >> (7 - self.bit_idx)) & 1 == 1;
        self.bit_idx += 1;
        self.total_position += 1;
        Ok(result)
    }

    fn read_bytes(&mut self, num_bytes: usize) -> Result<Box<[u8]>, Error> {
        self.refill_if_necessary()?;

        self.total_position += 8 * num_bytes;
        // fast aligned path
        if self.bit_idx == 0 {
            let mut buf = vec![0u8; num_bytes];
            self.reader.read_exact(&mut buf[1..])?;
            buf[0] = self.curr_byte;
            self.bit_idx = 8;
            Ok(buf.into_boxed_slice())
        } else {
            unimplemented!()
        }
    }

    fn read_bits(&mut self, num_bits: usize) -> Result<Box<[bool]>, Error> {
        unimplemented!("{}", num_bits)
    }

    fn read_unary(&mut self, start: bool) -> Result<u32, Error> {
        let mut data = 0u32;

        loop {
            self.refill_if_necessary()?;
            let bit = (self.curr_byte >> (7 - self.bit_idx)) & 1 == 1;
            self.bit_idx += 1;
            self.total_position += 1;

            if bit != start {
                break;
            }
            data += 1;
        }

        Ok(data)
    }

    fn read_utf8(&mut self, max_length: isize) -> Result<Box<str>, Error> {
        let mut raw_data = Vec::new();

        let mut cur_byte;

        loop {
            cur_byte = self.read_bytes(1)?[0];

            println!("Reading utf8, got byte {}", cur_byte);

            if cur_byte == 0 {
                break;
            }
            raw_data.push(cur_byte);

            if raw_data.len() as isize > max_length {
                return Err(Error::TooLong);
            }
        }

        Ok(String::from_utf8(raw_data)?.into_boxed_str())
    }

    fn read_sized(&mut self, num_bits: u8) -> Result<u128, Error> {
        debug_assert!(num_bits <= 128, "Cannot read {} bits into u128", num_bits);

        let mut data = 0u128;
        let mut bits_left = num_bits;

        // TODO: make this more efficient
        while bits_left > 0 {
            self.refill_if_necessary()?;

            let next_bit = (self.curr_byte >> (7 - self.bit_idx)) & 1;
            //println!("total bits: {}, bits left: {}, next bit: {}, data: {}", num_bits, bits_left, next_bit, data);
            data = data << 1 | (next_bit as u128);
            self.bit_idx += 1;
            self.total_position += 1;
            bits_left -= 1;
        }

        Ok(data)
    }

    fn get_total_position(&self) -> usize {
        self.total_position
    }
}
