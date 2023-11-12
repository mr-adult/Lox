use std::{array::from_fn, ops::{Index, IndexMut}};

pub (crate) struct FixedVec<T, const N: usize> {
    arr: [Option<T>; N],
    len: usize,
}

impl<T, const N: usize> FixedVec<T, N> {
    pub (crate) fn new() -> Self {
        Self {
            arr: from_fn(|_| None),
            len: 0,
        }
    }

    /// Pushes a value to the array. Returns Err if the array was full.
    pub (crate) fn push(&mut self, val: T) -> Result<(), FixedVecErr> {
        if self.len >= N {
            Err(FixedVecErr::AlreadyFull)
        } else {
            self.arr[self.len] = Some(val);
            self.len += 1;
            Ok(())
        }
    }

    pub (crate) fn pop(&mut self) -> Option<T> {
        if self.len > 0 {
            self.len -= 1;
            Some(std::mem::take(
                &mut self.arr[self.len as usize]
            )).unwrap()
        } else {
            None
        }
    }

    pub (crate) fn get(&self, index: usize) -> Option<&T> {
        match self.arr.get(index) {
            None => None,
            Some(val) => {
                val.as_ref()
            }
        }
    }

    pub (crate) fn len(&self) -> usize {
        self.len
    }

    pub (crate) fn capacity(&self) -> usize {
        N
    }

    pub (crate) fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter { inner: self.arr.iter() }
    }
}

impl<T, const N: usize> IntoIterator for FixedVec<T, N> {
    type Item = T;
    type IntoIter = IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { inner: self.arr.into_iter() }
    }
}

impl<T, const N: usize> Index<usize> for FixedVec<T, N> where T: Copy {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.arr[index].as_ref().unwrap()
    }
}

impl<T, const N: usize> IndexMut<usize> for FixedVec<T, N> where T: Copy {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.arr[index].as_mut().unwrap()
    }
}

pub (crate) struct Iter<'a, T> {
    inner: std::slice::Iter<'a, Option<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            None => None,
            Some(opt) => {
                opt.as_ref()
            }
        }
    }
}

pub (crate) struct IntoIter<T, const N: usize> {
    inner: std::array::IntoIter<Option<T>, N>,
}

impl<T, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            None => None,
            Some(opt) => opt,
        }
    }
}

#[derive(Debug)]
pub (crate) enum FixedVecErr {
    AlreadyFull,
}