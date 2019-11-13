use std::fs::File;
use std::io;
use std::io::{BufReader, Read};
use std::path::Path;

pub trait BitstreamReader {
    fn read_bit(&mut self) -> io::Result<bool>;
    fn read_bytes(&mut self, num_bytes: usize) -> io::Result<Box<[u8]>>;
    fn read_bits(&mut self, num_bits: usize) -> io::Result<Box<[bool]>>;
    fn read_sized(&mut self, num_bits: u8) -> io::Result<u128>;
    fn get_total_position(&self) -> usize;
}

pub struct BufferedBitstreamReader<T: Read> {
    reader: BufReader<T>,
    total_position: usize,
    curr_byte: u8,
    bit_idx: u8,
}

impl<T: Read> BufferedBitstreamReader<T> {
    pub fn new(filename: &Path) -> io::Result<BufferedBitstreamReader<File>> {
        let mut file = File::open(filename)?;
        let mut reader = BufReader::new(file);

        Ok(BufferedBitstreamReader {
            reader,
            total_position: 0,
            curr_byte: 0,
            bit_idx: 8,
        })
    }

    #[inline(always)]
    fn refill_if_necessary(&mut self) {
        if self.bit_idx == 8 {
            let mut buf = [0u8; 1];
            self.reader.read_exact(&mut buf);
            self.curr_byte = buf[0];
            self.bit_idx = 0;
        }
    }
}

impl<T: Read> BitstreamReader for BufferedBitstreamReader<T> {
    fn read_bit(&mut self) -> io::Result<bool> {
        self.refill_if_necessary();
        let result = (self.curr_byte >> (7 - self.bit_idx)) & 1 == 1;
        self.bit_idx += 1;
        self.total_position += 1;
        Ok(result)
    }

    fn read_bytes(&mut self, num_bytes: usize) -> io::Result<Box<[u8]>> {
        self.refill_if_necessary();

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

    fn read_bits(&mut self, num_bits: usize) -> io::Result<Box<[bool]>> {
        unimplemented!()
    }

    fn read_sized(&mut self, num_bits: u8) -> io::Result<u128> {
        debug_assert!(num_bits <= 128, "Cannot read {} bits into u128", num_bits);

        let mut data = 0u128;
        let mut bits_left = num_bits;

        // TODO: make this more efficfient
        while bits_left > 0 {
            self.refill_if_necessary();

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
