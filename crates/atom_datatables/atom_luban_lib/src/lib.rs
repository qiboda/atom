pub mod table;

pub mod prelude {
    pub use crate::table::*;
    pub use crate::*;
}

use std::cell::UnsafeCell;

#[derive(Debug)]
pub struct ByteBuf {
    pub reader_index: usize,
    pub writer_index: usize,
    pub bytes: Vec<u8>,
}

impl ByteBuf {
    #[allow(dead_code)]
    const MIN_CAPACITY: usize = 16;

    pub fn new(bytes: Vec<u8>) -> Self {
        ByteBuf {
            reader_index: 0,
            writer_index: bytes.len(),
            bytes,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        ByteBuf {
            reader_index: 0,
            writer_index: 0,
            bytes: Vec::with_capacity(capacity),
        }
    }

    pub fn replace(&mut self, bytes: Vec<u8>, begin_pos: usize, end_pos: usize) {
        self.bytes = bytes;
        self.reader_index = begin_pos;
        self.writer_index = end_pos;
    }

    pub fn capacity(&self) -> usize {
        self.bytes.capacity()
    }

    pub fn size(&self) -> usize {
        self.writer_index - self.reader_index
    }

    pub fn copy_data(&self) -> Vec<u8> {
        self.bytes[self.reader_index..self.writer_index].to_vec()
    }

    pub fn read_bool(&mut self) -> bool {
        self.ensure_read(1);
        let result = self.bytes[self.reader_index] != 0;
        self.reader_index += 1;
        result
    }

    pub fn read_byte(&mut self) -> u8 {
        self.ensure_read(1);
        let result = self.bytes[self.reader_index];
        self.reader_index += 1;
        result
    }

    pub fn read_short(&mut self) -> i16 {
        self.ensure_read(1);
        let h = self.bytes[self.reader_index];
        if h < 0x80 {
            self.reader_index += 1;
            return h as i16;
        }

        if h < 0xc0 {
            self.ensure_read(2);
            let x = (((h & 0x3f) as i16) << 8) | (self.bytes[self.reader_index + 1] as i16);
            self.reader_index += 2;
            return x;
        }

        if h < 0xff {
            self.ensure_read(3);
            let x = ((self.bytes[self.reader_index + 1] as i16) << 8)
                | (self.bytes[self.reader_index + 2] as i16);
            self.reader_index += 3;
            return x;
        }

        panic!("Invalid data")
    }

    pub fn read_uint(&mut self) -> u32 {
        self.ensure_read(1);
        let h = self.bytes[self.reader_index] as u32;
        if h < 0x80 {
            self.reader_index += 1;
            return h;
        }
        if h < 0xc0 {
            self.ensure_read(2);
            let x = ((h & 0x3f) << 8) | (self.bytes[self.reader_index + 1] as u32);
            self.reader_index += 2;
            return x;
        }
        if h < 0xe0 {
            self.ensure_read(3);
            let x = ((h & 0x1f) << 16)
                | ((self.bytes[self.reader_index + 1] as u32) << 8)
                | (self.bytes[self.reader_index + 2] as u32);
            self.reader_index += 3;
            return x;
        }
        if h < 0xf0 {
            self.ensure_read(4);
            let x = ((h & 0x0f) << 24)
                | ((self.bytes[self.reader_index + 1] as u32) << 16)
                | ((self.bytes[self.reader_index + 2] as u32) << 8)
                | (self.bytes[self.reader_index + 3] as u32);
            self.reader_index += 4;
            x
        } else {
            self.ensure_read(5);
            let x = ((self.bytes[self.reader_index + 1] as u32) << 24)
                | ((self.bytes[self.reader_index + 2] as u32) << 16)
                | ((self.bytes[self.reader_index + 3] as u32) << 8)
                | (self.bytes[self.reader_index + 4] as u32);
            self.reader_index += 5;
            x
        }
    }

    pub fn read_int(&mut self) -> i32 {
        self.read_uint() as i32
    }

    pub fn read_ulong(&mut self) -> u64 {
        self.ensure_read(1);
        let h = self.bytes[self.reader_index];
        if h < 0x80 {
            self.reader_index += 1;
            return h as u64;
        }
        if h < 0xc0 {
            self.ensure_read(2);
            let x = (((h & 0x3f) as u64) << 8) | (self.bytes[self.reader_index + 1] as u64);
            self.reader_index += 2;
            return x;
        }
        if h < 0xe0 {
            self.ensure_read(3);
            let x = (((h & 0x1f) as u64) << 16)
                | ((self.bytes[self.reader_index + 1] as u64) << 8)
                | (self.bytes[self.reader_index + 2] as u64);
            self.reader_index += 3;
            return x;
        }
        if h < 0xf0 {
            self.ensure_read(4);
            let x = (((h & 0x0f) as u64) << 24)
                | ((self.bytes[self.reader_index + 1] as u64) << 16)
                | ((self.bytes[self.reader_index + 2] as u64) << 8)
                | (self.bytes[self.reader_index + 3] as u64);
            self.reader_index += 4;
            return x;
        }
        if h < 0xf8 {
            self.ensure_read(5);
            let xl = ((self.bytes[self.reader_index + 1] as u64) << 24)
                | ((self.bytes[self.reader_index + 2] as u64) << 16)
                | ((self.bytes[self.reader_index + 3] as u64) << 8)
                | (self.bytes[self.reader_index + 4] as u64);
            let xh = (h & 0x07) as u64;
            self.reader_index += 5;
            return (xh << 32) | xl;
        }
        if h < 0xfc {
            self.ensure_read(6);
            let xl = ((self.bytes[self.reader_index + 2] as u64) << 24)
                | ((self.bytes[self.reader_index + 3] as u64) << 16)
                | ((self.bytes[self.reader_index + 4] as u64) << 8)
                | (self.bytes[self.reader_index + 5] as u64);
            let xh = (((h & 0x03) as u64) << 8) | (self.bytes[self.reader_index + 1] as u64);
            self.reader_index += 6;
            return (xh << 32) | xl;
        }
        if h < 0xfe {
            self.ensure_read(7);
            let xl = ((self.bytes[self.reader_index + 3] as u64) << 24)
                | ((self.bytes[self.reader_index + 4] as u64) << 16)
                | ((self.bytes[self.reader_index + 5] as u64) << 8)
                | (self.bytes[self.reader_index + 6] as u64);
            let xh = (((h & 0x01) as u64) << 16)
                | ((self.bytes[self.reader_index + 1] as u64) << 8)
                | (self.bytes[self.reader_index + 1] as u64);
            self.reader_index += 7;
            return (xh << 32) | xl;
        }
        if h < 0xff {
            self.ensure_read(8);
            let xl = ((self.bytes[self.reader_index + 4] as u64) << 24)
                | ((self.bytes[self.reader_index + 5] as u64) << 16)
                | ((self.bytes[self.reader_index + 6] as u64) << 8)
                | (self.bytes[self.reader_index + 7] as u64);
            let xh = ((self.bytes[self.reader_index + 1] as u64) << 16)
                | ((self.bytes[self.reader_index + 2] as u64) << 8)
                | (self.bytes[self.reader_index + 3] as u64);
            self.reader_index += 8;
            (xh << 32) | xl
        } else {
            self.ensure_read(9);
            let xl = ((self.bytes[self.reader_index + 5] as u64) << 24)
                | ((self.bytes[self.reader_index + 6] as u64) << 16)
                | ((self.bytes[self.reader_index + 7] as u64) << 8)
                | (self.bytes[self.reader_index + 8] as u64);
            let xh = ((self.bytes[self.reader_index + 1] as u64) << 24)
                | ((self.bytes[self.reader_index + 2] as u64) << 16)
                | ((self.bytes[self.reader_index + 3] as u64) << 8)
                | (self.bytes[self.reader_index + 4] as u64);
            self.reader_index += 9;
            (xh << 32) | xl
        }
    }

    pub fn read_long(&mut self) -> i64 {
        self.read_ulong() as i64
    }

    pub fn read_float(&mut self) -> f32 {
        self.ensure_read(4);
        let b = &self.bytes[self.reader_index] as *const u8;
        let mut x = 0_f32;
        unsafe {
            if (b as u64).is_multiple_of(8) {
                x = *(b as *const f32)
            } else {
                let c = UnsafeCell::new(x);
                *(c.get() as *mut u32) = (*b.offset(0) as u32)
                    | ((*b.offset(1) as u32) << 8)
                    | ((*b.offset(2) as u32) << 16)
                    | ((*b.offset(3) as u32) << 24);
            }
        }

        self.reader_index += 4;
        x
    }

    pub fn read_double(&mut self) -> f64 {
        self.ensure_read(8);
        let b = &self.bytes[self.reader_index] as *const u8;
        let mut x = 0_f64;
        unsafe {
            if (b as u64).is_multiple_of(8) {
                x = *(b as *const f64)
            } else {
                let low = (*b.offset(0) as u64)
                    | ((*b.offset(1) as u64) << 8)
                    | ((*b.offset(2) as u64) << 16)
                    | ((*b.offset(3) as u64) << 24);
                let high = (*b.offset(4) as u64)
                    | ((*b.offset(5) as u64) << 8)
                    | ((*b.offset(6) as u64) << 16)
                    | ((*b.offset(7) as u64) << 24);
                let c = UnsafeCell::new(x);
                *(c.get() as *mut u64) = ((high) << 32) | (low)
            }
        }

        self.reader_index += 8;
        x
    }

    pub fn read_size(&mut self) -> usize {
        self.read_uint() as usize
    }

    pub fn read_string(&mut self) -> String {
        let n = self.read_size();
        if n > 0 {
            self.ensure_read(n);
            let s = String::from_utf8_lossy(&self.bytes[self.reader_index..self.reader_index + n]);
            self.reader_index += n;
            return s.to_string();
        }

        "".to_string()
    }
    //region internal

    #[allow(dead_code)]
    fn prop_size(init_size: usize, need_size: usize) -> usize {
        let mut i = usize::max(init_size, Self::MIN_CAPACITY);
        loop {
            if i >= need_size {
                return i;
            }

            i <<= 1;
        }
    }

    #[inline]
    fn ensure_read(&self, size: usize) {
        if self.reader_index + size > self.writer_index {
            panic!("Not enough data")
        }
    }

    #[allow(dead_code)]
    #[inline]
    fn can_read(&self, size: usize) -> bool {
        self.reader_index + size <= self.writer_index
    }

    //endregion
}

impl PartialEq<Self> for ByteBuf {
    fn eq(&self, other: &Self) -> bool {
        if self.size() != other.size() {
            return false;
        }

        for i in 0..self.size() {
            if self.bytes[self.reader_index + i] != other.bytes[other.reader_index + i] {
                return false;
            }
        }

        true
    }
}

impl Eq for ByteBuf {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_size() {
        let buf = ByteBuf::new(vec![1, 2, 3]);
        assert_eq!(buf.size(), 3);
        assert_eq!(buf.reader_index, 0);
        assert_eq!(buf.writer_index, 3);
    }

    #[test]
    fn test_with_capacity() {
        let buf = ByteBuf::with_capacity(32);
        assert_eq!(buf.size(), 0);
        assert!(buf.capacity() >= 32);
    }

    #[test]
    fn test_copy_data() {
        let buf = ByteBuf::new(vec![10, 20, 30]);
        assert_eq!(buf.copy_data(), vec![10, 20, 30]);
    }

    #[test]
    fn test_replace() {
        let mut buf = ByteBuf::new(vec![1, 2, 3]);
        buf.replace(vec![4, 5, 6, 7], 1, 3);
        assert_eq!(buf.size(), 2);
        assert_eq!(buf.copy_data(), vec![5, 6]);
    }

    #[test]
    fn test_read_bool() {
        let mut buf = ByteBuf::new(vec![0, 1, 255]);
        assert!(!buf.read_bool());
        assert!(buf.read_bool());
        assert!(buf.read_bool());
    }

    #[test]
    fn test_read_byte() {
        let mut buf = ByteBuf::new(vec![0, 127, 255]);
        assert_eq!(buf.read_byte(), 0);
        assert_eq!(buf.read_byte(), 127);
        assert_eq!(buf.read_byte(), 255);
    }

    #[test]
    fn test_read_short_one_byte() {
        // h < 0x80: single byte
        let mut buf = ByteBuf::new(vec![0x7F]);
        assert_eq!(buf.read_short(), 0x7F);
    }

    #[test]
    fn test_read_short_two_bytes() {
        // h in [0x80, 0xc0): two bytes, value = ((h & 0x3f) << 8) | byte2
        // h=0x80, byte2=0x01 => ((0x80 & 0x3f) << 8) | 0x01 = 0x0001 = 1
        let mut buf = ByteBuf::new(vec![0x80, 0x01]);
        assert_eq!(buf.read_short(), 1);
    }

    #[test]
    fn test_read_short_three_bytes() {
        // h in [0xC0, 0xFF): three bytes, value = (byte2 << 8) | byte3
        // h=0xC0, byte2=0x01, byte3=0x00 => (0x01 << 8) | 0x00 = 256
        let mut buf = ByteBuf::new(vec![0xC0, 0x01, 0x00]);
        assert_eq!(buf.read_short(), 256);
    }

    #[test]
    fn test_read_uint_one_byte() {
        let mut buf = ByteBuf::new(vec![0x00]);
        assert_eq!(buf.read_uint(), 0);
        let mut buf = ByteBuf::new(vec![0x7F]);
        assert_eq!(buf.read_uint(), 127);
    }

    #[test]
    fn test_read_uint_two_bytes() {
        // h=0x80, byte2=0x00 => ((0x80 & 0x3F) << 8) | 0x00 = 0
        let mut buf = ByteBuf::new(vec![0x80, 0x00]);
        assert_eq!(buf.read_uint(), 0);
        // h=0x80, byte2=0xFF => ((0x80 & 0x3F) << 8) | 0xFF = 255
        let mut buf = ByteBuf::new(vec![0x80, 0xFF]);
        assert_eq!(buf.read_uint(), 255);
    }

    #[test]
    fn test_read_uint_five_bytes() {
        // h >= 0xF0: 5 bytes
        // h=0xF0, then 4 bytes for the value
        let mut buf = ByteBuf::new(vec![0xF0, 0x00, 0x00, 0x01, 0x00]);
        assert_eq!(buf.read_uint(), 256);
    }

    #[test]
    fn test_read_int() {
        let mut buf = ByteBuf::new(vec![0x01]);
        assert_eq!(buf.read_int(), 1);
    }

    #[test]
    fn test_read_ulong_one_byte() {
        let mut buf = ByteBuf::new(vec![0x7F]);
        assert_eq!(buf.read_ulong(), 127);
    }

    #[test]
    fn test_read_ulong_two_bytes() {
        let mut buf = ByteBuf::new(vec![0x80, 0x01]);
        assert_eq!(buf.read_ulong(), 1);
    }

    #[test]
    fn test_read_float() {
        let val: f32 = 1.0;
        let bytes = val.to_le_bytes();
        let mut buf = ByteBuf::new(bytes.to_vec());
        let read_val = buf.read_float();
        assert_eq!(read_val, 1.0);
    }

    #[test]
    fn test_read_float_negative() {
        let val: f32 = -3.25;
        let bytes = val.to_le_bytes();
        let mut buf = ByteBuf::new(bytes.to_vec());
        let read_val = buf.read_float();
        assert!((read_val - (-3.25)).abs() < 1e-6);
    }

    #[test]
    fn test_read_double() {
        let val: f64 = 1.23456789;
        let bytes = val.to_le_bytes();
        let mut buf = ByteBuf::new(bytes.to_vec());
        let read_val = buf.read_double();
        assert!((read_val - 1.23456789).abs() < 1e-9);
    }

    #[test]
    fn test_read_string_empty() {
        // size=0 => empty string
        let mut buf = ByteBuf::new(vec![0x00]);
        assert_eq!(buf.read_string(), "");
    }

    #[test]
    fn test_read_string_nonempty() {
        // size=5, then "hello"
        let mut bytes = vec![0x05];
        bytes.extend_from_slice(b"hello");
        let mut buf = ByteBuf::new(bytes);
        assert_eq!(buf.read_string(), "hello");
    }

    #[test]
    fn test_partial_eq() {
        let buf1 = ByteBuf::new(vec![1, 2, 3]);
        let buf2 = ByteBuf::new(vec![1, 2, 3]);
        assert_eq!(buf1, buf2);

        let buf3 = ByteBuf::new(vec![1, 2, 4]);
        assert_ne!(buf1, buf3);

        let buf4 = ByteBuf::new(vec![1, 2]);
        assert_ne!(buf1, buf4);
    }

    #[test]
    #[should_panic(expected = "Not enough data")]
    fn test_ensure_read_panic() {
        let mut buf = ByteBuf::new(vec![1]);
        buf.read_byte();
        buf.read_byte(); // should panic
    }

    #[test]
    fn test_sequential_reads() {
        let bytes = vec![
            1u8,  // read_bool -> true
            42u8, // read_byte -> 42
            0x05, // read_uint -> 5
        ];

        let mut buf = ByteBuf::new(bytes);
        assert!(buf.read_bool());
        assert_eq!(buf.read_byte(), 42);
        assert_eq!(buf.read_uint(), 5);
        assert_eq!(buf.size(), 0);
    }
}
