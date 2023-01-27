use std::ops::{Add, Index, Mul};
use bitvec::prelude::*;

#[derive(Clone, Default, Debug, PartialEq, strum::Display, strum::EnumString, strum::FromRepr)]
pub enum Logic {
    #[default]
    #[strum(serialize = "x")]
    X = 0,
    #[strum(serialize = "z")]
    Z = 1,
    #[strum(serialize = "0")]
    Zero = 2,
    #[strum(serialize = "1")]
    One = 3,
}

impl Logic {
    pub fn from_valid_01(first: bool, second: bool) -> Self {
        match (first, second) {
            (true, false) => Self::Zero,
            (true, true) => Self::One,
            (false, false) => Self::X,
            (false, true) => Self::Z,
        }
    }
    pub fn to_valid_01(&self) -> (bool, bool) {
        match &self {
            Self::Zero => (true, false),
            Self::One => (true, true),
            Self::X => (false, false),
            Self::Z => (false, true),
        }
    }
}

impl TryFrom<&Logic> for bool {
    type Error = ();

    fn try_from(value: &Logic) -> Result<Self, Self::Error> {
        match value {
            Logic::Zero => Ok(false),
            Logic::One => Ok(true),
            _ => Err(Default::default()),
        }
    }
}


/*
4-valued logic

___|___|___
 0 | 0 | X
 0 | 1 | z
 1 | 0 | 0
 1 | 1 | 1
-----------
*/
#[derive(Clone, Debug)]
pub struct LogicVec {
    bv: BitVec,
    values: [Logic; 4],
}

impl Default for LogicVec {
    fn default() -> Self {
        Self {
            bv: Default::default(),
            values: [Logic::X, Logic::Z, Logic::Zero, Logic::One],
        }
    }
}

impl LogicVec {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bv: BitVec::with_capacity(2 * capacity),
            ..Default::default()
        }
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.bv.capacity() / 2
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.bv.len() / 2
    }

    #[inline(always)]
    pub fn shrink_to_fit(&mut self) {
        self.bv.shrink_to_fit();
    }
    #[inline(always)]
    fn resize(&mut self, new_len: usize) {
        self.bv.resize(2 * new_len, false);
    }

    #[inline(always)]
    pub fn set(&mut self, index: usize, value: Logic) {
        if index >= self.len() {
            self.resize(index + 1);
        }
        // self.bv.get_mut(2 * index..=2 * index + 1).unwrap().store(value.clone() as u8);
        debug_assert!(self.bv.len() > (2 * index + 1));
        unsafe {
            self.bv.get_unchecked_mut(2 * index..=2 * index + 1).store(value as u8);
        }
    }

    pub fn get(&self, index: usize) -> Logic {
        match self.bv.get(2 * index..=2 * index + 1) {
            Some(xz_zo) => {
                Logic::from_repr(xz_zo.load()).unwrap()
            }
            None => Logic::X,
        }
    }
}

// impl Index<usize> for LogicVec {
//     type Output = Logic;
//
//     fn index(&self, index: usize) -> &Self::Output {
//         let i = index * 2;
//         match self.bv.get(i..i + 1) {
//             Some(xz_zo) => {
//                 &Logic::from_repr(xz_zo.load()).unwrap()
//             }
//             None => &Logic::X,
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut lv = LogicVec::default();
        lv.set(5, Logic::Zero);
        lv.set(7, Logic::One);
        for i in 0..8 {
            println!("{}: {:?}", i, lv.get(i));
        }
        println!("{:?}", lv);
        assert_eq!(lv.get(5), Logic::Zero);
        assert_eq!(lv.get(7), Logic::One);
        assert_eq!(lv.get(1), Logic::X);
        assert_eq!(lv.get(100), Logic::X);
    }
}