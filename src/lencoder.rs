//! ### Variable-Length Integer Encoding
//!
//! This encoding ensures that smaller integer values need fewer bytes to encode. Support types are `L2` and `L3`.
//!
//! By default, `L2` (u15) is used to encode length (integer) for record. But you override it by setting `L3` (u22) in features flag.
//!  
//! Encoding algorithm is very straightforward, reserving one or two most significant bits of the first byte to encode rest of the length.
//!
//! #### L2
//!
//! |  MSB  | Length | Usable Bits | Range    |
//! | :---: | :----: | :---------: | :------- |
//! |   0   |   1    |      7      | 0..127   |
//! |   1   |   2    |     15      | 0..32767 |
//!
//!
//! #### L3
//!
//! |  MSB  | Length | Usable Bits | Range      |
//! | :---: | :----: | :---------: | :--------- |
//! |   0   |   1    |      7      | 0..127     |
//! |  10   |   2    |     14      | 0..16383   |
//! |  11   |   3    |     22      | 0..4194303 |
//!
//!  
//! For example, Binary representation of `0x_C0DE` is `0x_11_00000011_011110`
//!  
//! `L3(0x_C0DE)` is encoded in 3 bytes:
//!  
//! ```text
//! 1st byte: 11_011110      # MSB is 11, so read next 2 bytes
//! 2nd byte:        11
//! 3rd byte:        11
//! ```
//!
//! Another example, `L3(107)` is encoded in just 1 byte:
//!
//! ```text
//! 1st byte: 0_1101011      # MSB is 0, So we don't have to read another bytes.
//! ```
use crate::*;

#[cfg(not(feature = "L3"))]
pub use L2 as Lencoder;
#[cfg(feature = "L3")]
pub use L3 as Lencoder;


macro_rules! def {
    [$name:ident($ty:ty), LenSize: $size:literal, MAX: $MAX:literal, $encoder:item, $decoder:item] => {
        #[derive(Default, Debug, Clone, Copy)]
        pub struct $name(pub $ty);
        impl $name { pub const MAX: $ty = $MAX; }

        impl Encoder for $name {
            const SIZE: usize = $size;
            #[inline] $encoder
        }
        impl<E: Error> Decoder<'_, E> for $name { #[inline] $decoder }
        impl From<$ty> for $name { fn from(num: $ty) -> Self { Self(num) } }
        impl core::ops::Deref for $name {
            type Target = $ty;
            fn deref(&self) -> &Self::Target { &self.0 }
        }
        impl core::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
        }
    };
}   

def!(
    L2(u16),
    LenSize: 2,
    MAX: 0x7FFF,
    fn encoder(self, c: &mut impl Array<u8>) {
        let num = self.0;
        let b1 = num as u8;
        if num < 128 {
            b1.encoder(c); // No MSB is set, Bcs `num` is less then `128`
        } else {
            debug_assert!(num <= Self::MAX);
            let b1 = 0x80 | b1; // 7 bits with MSB is set.
            let b2 = (num >> 7) as u8; // next 8 bits
            c.extend_from_slice([b1, b2]);
        }
    },
    fn decoder(c: &mut Cursor<&[u8]>) -> Result<Self, E> {
        let mut num = u8::decoder(c)? as u16;
        // if MSB is set, read another byte.
        if num >> 7 == 1 {
            let snd = u8::decoder(c)? as u16;
            num = (num & 0x7F) | snd << 7; // num <- add 8 bits
        }
        Ok(Self(num))
    }
);
def!(
    L3(u32),
    LenSize: 3,
    MAX: 0x3FFFFF,
    fn encoder(self, c: &mut impl Array<u8>) {
        let num = self.0;
        let b1 = num as u8;
        if num < 128 {
            b1.encoder(c);
        }
        else {
            let b1 = b1 & 0x3F; // read last 6 bits
            let b2 = (num >> 6) as u8; // next 8 bits
            if num < 0x4000 {
                // set first 2 bits  of `b1` to `10`
                c.extend_from_slice([0x80 | b1, b2]);
            }
            else {
                debug_assert!(num <= Self::MAX);
                let b3 = (num >> 14) as u8; // next 8 bits
                // set first 2 bits  of `b1` to `11`
                c.extend_from_slice([0xC0 | b1, b2, b3]);
            }
        }
    },
    fn decoder(c: &mut Cursor<& [u8]>) -> Result<Self, E> {
        let num = u8::decoder(c)? as u32;
        // if 1st bit is `0`
        let num = if num >> 7 == 0 { num }
        // and 2nd bit is `0`
        else if num >> 6 == 2 {
            let b2 = u8::decoder(c)? as u32;
            (num & 0x3F) | b2 << 6
        } else  {
            // At this point, only possible first 2 bits are `11`
            let b2 = *c.data.get(c.offset).ok_or_else(E::insufficient_bytes)? as u32;
            let b3 = *c.data.get(c.offset + 1).ok_or_else(E::insufficient_bytes)? as u32;
            c.offset += 2;

            (num & 0x3F)  // get last 6 bits
            | b2 << 6     // add 8 bits from 2nd byte
            | b3 << 14    // add 8 bits from 3rd byte
        };
        Ok(Self(num))
    }
);
