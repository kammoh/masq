use std::fmt::{Debug, Display, Formatter};
use std::ops::{Index, Range};
use std::str::FromStr;
use funty::{Fundamental, Integral};
use bitvec::prelude::*;
use bitvec::slice::BitSliceIndex;
use itertools::Itertools;
use nom::InputIter;
use num_enum::{IntoPrimitive, FromPrimitive};
use simple_error::SimpleError;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid logic value:`{0}`")]
    InvalidLogicValue(String),
    #[error(transparent)]
    LogicParseError(#[from] strum::ParseError),
    #[error("encountered logic value:`{0}` when expecting 0 or 1")]
    NotZeroOne(Logic),
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
#[derive(strum::Display, strum::EnumString, strum::EnumIter, strum::FromRepr)]
#[derive(IntoPrimitive, FromPrimitive)]
#[repr(u8)]
#[strum(ascii_case_insensitive)]
pub enum Logic {
    #[default]
    #[strum(serialize = "z")]
    Z = 0,
    #[strum(serialize = "x")]
    X = 1,
    #[strum(serialize = "0")]
    Zero = 2,
    #[strum(serialize = "1")]
    One = 3,
}

impl TryFrom<&Logic> for bool {
    type Error = Error;

    fn try_from(value: &Logic) -> Result<Self, Self::Error> {
        match value {
            Logic::Zero => Ok(false),
            Logic::One => Ok(true),
            _ => Err(Error::NotZeroOne(value.clone())),
        }
    }
}

impl TryFrom<char> for Logic {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        // TODO optimize
        Logic::from_str(&value.to_string()).map_err(|e| Error::LogicParseError(e))
    }
}

// or:
// impl From<&Logic> for bool {
//     fn from(value: &Logic) -> Self {
//         match value {
//             Logic::One => true,
//             _ => false,
//         }
//     }
// }

static LOGIC_VALUES: [Logic; 4] = [Logic::Z, Logic::X, Logic::Zero, Logic::One];

/*
4-valued logic

___|___|___
 0 | 0 | X
 0 | 1 | z
 1 | 0 | 0
 1 | 1 | 1
-----------
*/
#[derive(Clone, Debug, Default, PartialEq, Hash)]
pub struct LogicVec1 {
    bv: BitVec,
}

impl LogicVec1 {
    #[inline(always)]
    fn resize(&mut self, new_len: usize) {
        self.bv.resize(2 * new_len, false);
    }
    #[inline(always)]
    fn bit_range(index: usize) -> Range<usize> {
        2 * index..2 * index + 2
    }
}


#[derive(Clone, Debug, Default, PartialEq, Hash)]
pub struct LogicVec {
    bv_zx: BitVec,
    bv_01: BitVec,
}

impl LogicVec {
    #[inline(always)]
    fn resize(&mut self, new_len: usize) {
        self.bv_zx.resize(new_len, false);
        self.bv_01.resize(new_len, false);
    }

    fn get_i<'a, I>(&'a self, index: I) where I: BitSliceIndex<'a, usize, Lsb0> + Clone {
        todo!()
        // let zx = self.bv_zx.get(index.clone()).iter().flatten();
        // let v01 = self.bv_01.get(index.clone()).iter().flatten();
        // for (i, j) in zx.zip(v01) {}
    }

    #[inline(always)]
    fn iter(&self) -> impl Iterator<Item=Logic> + DoubleEndedIterator<Item=Logic> + '_ {
        self.bv_zx.iter().zip(self.bv_01.iter()).map(|(v_xz, v_01)|
            Logic::from(v_xz.as_u8() * 2 + v_01.as_u8())
        )
    }

    //TODO errors
    fn from_iter<I, T>(i: I) -> Result<Self, ()> where I: IntoIterator<Item=T>, Logic: From<T> {
        // TODO optimize with extend etc
        let mut lv = Self::default();
        for e in i.into_iter() {
            lv.push(e);
        }
        Ok(lv)
    }
}

trait LogicVector {
    fn with_capacity(capacity: usize) -> Self;
    fn capacity(&self) -> usize;
    fn len(&self) -> usize;
    fn shrink_to_fit(&mut self);
    fn set(&mut self, index: usize, value: Logic);
    fn get_as<I>(&self, index: usize) -> I where I: Integral + Debug + From<u8>;

    fn push<T>(&mut self, value: T) where Logic: From<T>;

    #[inline(always)]
    fn get(&self, index: usize) -> Logic {
        Logic::from(self.get_as::<u8>(index))
    }

    #[inline(always)]
    fn get_ref(&self, index: usize) -> &Logic {
        let idx = self.get_as::<usize>(index);
        debug_assert!(idx < LOGIC_VALUES.len());
        // index is either 0 (default) or 0..3 (conversion of 2 bits), therefore always safe
        unsafe {
            LOGIC_VALUES.get_unchecked(idx)
        }
    }
}

impl LogicVector for LogicVec1 {
    #[inline(always)]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            bv: BitVec::with_capacity(2 * capacity),
            ..Default::default()
        }
    }

    #[inline(always)]
    fn capacity(&self) -> usize {
        self.bv.capacity() / 2
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.bv.len() / 2
    }

    #[inline(always)]
    fn shrink_to_fit(&mut self) {
        self.bv.shrink_to_fit();
    }

    #[inline(always)]
    fn set(&mut self, index: usize, value: Logic) {
        if index >= self.len() {
            self.resize(index + 1);
        }
        let range = Self::bit_range(index);
        debug_assert!(self.bv.len() >= range.end);
        // we already resized if out of bounds, therefore always safe
        unsafe {
            self.bv.get_unchecked_mut(range).store(value as u8);
        }
    }

    #[inline(always)]
    fn get_as<I>(&self, index: usize) -> I where I: Integral {
        self.bv.get(Self::bit_range(index)).map(|bits| {
            bits.load::<I>()
        }).unwrap_or_default()
    }

    #[inline(always)]
    fn push<T>(&mut self, value: T) where Logic: From<T> {
        let value = Logic::from(value);
        let v = u8::from(value);
        self.bv.extend_from_bitslice(v.view_bits::<Lsb0>())
    }
}

impl LogicVector for LogicVec {
    #[inline(always)]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            bv_zx: BitVec::with_capacity(capacity),
            bv_01: BitVec::with_capacity(capacity),
        }
    }

    #[inline(always)]
    fn capacity(&self) -> usize {
        self.bv_zx.capacity()
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.bv_zx.len()
    }

    #[inline(always)]
    fn shrink_to_fit(&mut self) {
        self.bv_zx.shrink_to_fit();
        self.bv_01.shrink_to_fit();
    }

    #[inline(always)]
    fn set(&mut self, index: usize, value: Logic) {
        if index >= self.len() {
            self.resize(index + 1);
        }
        let v_u8 = value as u8;
        let v = v_u8.view_bits::<Lsb0>();

        self.bv_zx.set(index, v[1]);
        self.bv_01.set(index, v[0]);
    }
    #[inline(always)]
    fn get_as<I>(&self, index: usize) -> I where I: Integral + Debug + From<u8> {
        let v = match (self.bv_zx.get(index), self.bv_01.get(index)) {
            // TODO hacky and not robust to any change in Logic values
            (Some(v_zx), Some(v_01)) => ((v_zx.as_u8()) << 1) + (v_01.as_u8()),
            _ => 0,
        };
        I::from(v)
    }

    #[inline(always)]
    fn push<T>(&mut self, value: T) where Logic: From<T> {
        let value = Logic::from(value);
        let (zx, oz) = match value {
            Logic::Z => { (false, false) }
            Logic::X => { (false, true) }
            Logic::Zero => { (true, false) }
            Logic::One => { (true, true) }
        };
        self.bv_zx.push(zx);
        self.bv_01.push(oz);
    }
}

impl Index<usize> for LogicVec1 {
    type Output = Logic;
    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        self.get_ref(index)
    }
}

impl Index<usize> for LogicVec {
    type Output = Logic;
    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        self.get_ref(index)
    }
}


// Note: does not support u128! (assuming usize == u64)
impl<I> From<I> for LogicVec where I: Integral {
    #[inline(always)]
    fn from(value: I) -> Self {
        let bv_01 = BitVec::from_bitslice(value.as_usize().view_bits());
        Self {
            bv_zx: bitvec![1; bv_01.len()],
            bv_01,
        }
    }
}

impl FromStr for LogicVec {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lv = LogicVec::default();
        for c in s.chars().rev() {
            lv.push(Logic::try_from(c)?);
        }
        Ok(lv)
    }
}

impl Display for LogicVec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for bit in self.iter().rev() {
            std::fmt::Display::fmt(&bit, f)?
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut lv = LogicVec1::default();
        lv.set(5, Logic::Zero);
        lv.set(7, Logic::One);
        lv.set(0, Logic::Zero);
        lv.set(0, Logic::One);
        lv.set(6, Logic::One);
        lv.set(8, Logic::X);
        for i in 0..8 {
            println!("{}: {:?}", i, lv.get(i));
        }
        println!("{:?}", lv);
        assert_eq!(lv.get(5), Logic::Zero);
        assert_eq!(lv[5], Logic::Zero);
        assert_eq!(lv.get(7), Logic::One);
        assert_eq!(lv[7], Logic::One);
        assert_eq!(lv.get(8), Logic::X);
        assert_eq!(lv[8], Logic::X);
        assert_eq!(lv.get(0), Logic::One);
        assert_eq!(lv.get(6), Logic::One);
        assert_eq!(lv[6], Logic::One);
        assert_eq!(lv.get(1), Logic::default());
        assert_eq!(lv.get(100), Logic::default());
        assert_eq!(lv[1000], Logic::default());
    }


    #[test]
    fn test_lv2() {
        let mut lv = LogicVec::default();
        lv.set(5, Logic::Zero);
        lv.set(7, Logic::One);
        lv.set(0, Logic::Zero);
        lv.set(0, Logic::One);
        lv.set(6, Logic::One);
        lv.set(8, Logic::X);
        for i in 0..8 {
            println!("{}: {:?}", i, lv.get(i));
        }
        println!("{:?}", lv);
        assert_eq!(lv.get(5), Logic::Zero);
        assert_eq!(lv[5], Logic::Zero);
        assert_eq!(lv.get(7), Logic::One);
        assert_eq!(lv[7], Logic::One);
        assert_eq!(lv.get(8), Logic::X);
        assert_eq!(lv[8], Logic::X);
        assert_eq!(lv.get(0), Logic::One);
        assert_eq!(lv.get(6), Logic::One);
        assert_eq!(lv[6], Logic::One);
        assert_eq!(lv.get(1), Logic::default());
        assert_eq!(lv.get(100), Logic::default());
        assert_eq!(lv[1000], Logic::default());
    }


    #[test]
    fn test_lv2_from_num() {
        let lv = LogicVec::from(0xaa3);
        assert_eq!(lv[0], Logic::One);
        assert_eq!(lv[2], Logic::Zero);
        assert_eq!(lv[3], Logic::Zero);
        assert_eq!(lv[4], Logic::Zero);
        assert_eq!(lv[5], Logic::One);
        println!("{:?}", lv);
    }

    #[test]
    fn test_lv2_from_str() -> Result<(), Error> {
        let orig = "100101110zZxX010";
        let lv = LogicVec::from_str(orig)?;
        let s = lv.to_string();
        assert_eq!(orig.to_lowercase(), s.to_lowercase());
        assert_eq!(lv[0], Logic::Zero);
        assert_eq!(lv[1], Logic::One);
        assert_eq!(lv[2], Logic::Zero);
        assert_eq!(lv[3], Logic::X);
        assert_eq!(lv[4], Logic::X);
        assert_eq!(lv[5], Logic::Z);
        assert_eq!(lv[6], Logic::Z);
        assert_eq!(lv[7], Logic::Zero);
        assert_eq!(lv[8], Logic::One);
        assert_eq!(lv[9], Logic::One);
        assert_eq!(lv[10], Logic::One);
        assert_eq!(lv[11], Logic::Zero);
        assert_eq!(lv[12], Logic::One);
        assert_eq!(lv[13], Logic::Zero);
        assert_eq!(lv[14], Logic::Zero);
        assert_eq!(lv[15], Logic::One);
        println!("{}", lv);
        Ok(())
    }
}