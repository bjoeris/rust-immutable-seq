
use std::iter;
use std::ops;
use std::convert;

use lazy::Lazy;

use finger_tree;
use finger_tree::FingerTree;
use node;
use measure::Measure;

#[derive(Debug)]
struct Item<T>(T);

impl<T> Measure<usize> for Item<T> {
    fn measure(&self) -> usize {1}
}

#[derive(Debug)]
pub struct Seq<T> (Lazy<FingerTree<Item<T>,usize>>);

impl<T:'static> Seq<T> {
    /// The empty sequence.
    ///
    /// /O(1)/
    pub fn empty() -> Seq<T> {
        Seq(finger_tree::empty())
    }

    /// A sequence with a single value.
    ///
    /// /O(1)/
    pub fn singleton(x: T) -> Seq<T> {
        Seq(finger_tree::single(node::leaf(Item(x))))
    }

    /// A new sequence that is `self` with `x` added to the front.
    ///
    /// /O(1)/
    pub fn push_front(&self, x: T) -> Seq<T> {
        Seq(finger_tree::cons_node(node::leaf(Item(x)), self.inner().clone()))
    }

    /// A new sequence that is `self` with `x` added to the back.
    ///
    /// /O(1)/
    pub fn push_back(&self, x: T) -> Seq<T> {
        Seq(finger_tree::snoc_node(self.inner().clone(), node::leaf(Item(x))))
    }

    /// The concatenation of `self` with `other`.
    ///
    /// /O(log(min(`self.len()`,`other.len()`)))/
    pub fn append(&self, other: &Seq<T>) -> Seq<T> {
        Seq(finger_tree::tree_tree(self.inner().clone(), other.inner().clone()))
    }

    /// Is the sequence empty?
    pub fn is_empty(&self) -> bool {
        self.inner().measure() == 0
    }

    /// The number of elements in the sequence
    ///
    /// /O(1)/
    pub fn len(&self) -> usize {
        self.inner().measure()
    }

    /// The first element in the sequence, if it exists.
    ///
    /// /O(1)/
    pub fn front(&self) -> Option<&T> {
        finger_tree::front(self.inner()).map(|&Item(ref x)| x)
    }

    /// The back element, if it exsts.
    ///
    /// /O(1)/
    pub fn back(&self) -> Option<&T> {
        finger_tree::back(self.inner()).map(|&Item(ref x)| x)
    }

    /// A new sequence that is `self` with the front element removed, together with the front element (if it exists).
    ///
    /// /O(1)/
    pub fn pop_front(&self) -> Seq<T> {
        Seq(finger_tree::pop_front(self.inner()))
    }

    /// A new sequence that is `self` with the back element removed, together with the back element (if it exists).
    ///
    /// /O(1)/
    pub fn pop_back(&self) -> Seq<T> {
        Seq(finger_tree::pop_back(self.inner()))
    }

    /// A new sequence with the element at index `i` replaced by `f(self[i])`.
    ///
    /// If `i` is out of range, returns a clone of `self`.
    ///
    /// /O(log(min(i,`self.len()`-i)))/
    pub fn adjust<F>(&self, i: usize, func: F) -> Seq<T>
        where F: FnOnce(&T) -> T
    {
        if i >= self.len() {
            return self.clone()
        }
        Seq(finger_tree::adjust(move |&Item(ref x)| Item(func(x)), move |j| {i < j}, 0, self.inner()))
    }

    /// A new sequence with the element at index `i` replaced by `x`.
    ///
    /// If `i` is out of range, returns a clone of `self`.
    ///
    /// /O(log(min(i,`self.len()`-i)))/
    pub fn update(&self, i: usize, x: T) -> Seq<T> {
        self.adjust(i, move |_| x)
    }

    /// A new sequence consisting of only the first `n` elements.
    ///
    /// If `n >= self.len()`, then returns a clone of `self`.
    ///
    /// /O(log(min(n,`self.len()`-n)))/
    pub fn truncate(&self, n: usize) -> Seq<T> {
        let (before,_) = self.split(n);
        before
    }

    /// A new sequence consisting of only the last `n` elements.
    ///
    /// If `n >= self.len()`, then returns a clone of `self`.
    ///
    /// /O(log(min(n,`self.len()`-n)))/
    pub fn skip(&self, n: usize) -> Seq<T> {
        let (_,after) = self.split(n);
        after
    }

    /// Two new sequences, consisting of the first `n` elements, and the remaining elements, respectively.
    ///
    /// If `n >= self.len()`, then the first sequence is a clone of `self` and the second is empty.
    ///
    /// /O(log(min(n,`self.len()`-n)))/
    pub fn split(&self, n: usize) -> (Seq<T>, Seq<T>) {
        if n >= self.len() {
            return (self.clone(), Seq::empty())
        }
        let (before,x,after) = finger_tree::split(&move |i| {n < i}, 0, self.inner());
        (Seq(before), Seq(finger_tree::cons_node(x.clone(), after)))
    }

    /// A new sequence with the element at index `i` removed, together with the element at index `i`, if it exists.
    ///
    /// If `i` is out of range, then the returned sequence is a clone of `self`, and the element is `None`.
    ///
    /// /O(log(min(i,`self.len()`-i)))/
    pub fn remove(&self, i: usize) -> Seq<T> {
        if i >= self.len() {
            return self.clone()
        }
        let (before,_,after) = finger_tree::split(&move |j| {i < j}, 0, self.inner());
        Seq(finger_tree::tree_tree(before, after))
    }

    /// A new sequence with `x` inserted at index `i`.
    ///
    /// If `i < self.len()`, then `x` will immediately precede `self[i]` in the new sequence.
    ///
    /// if `i >= self.len()`, then `x` will be the last element in the new sequence.
    ///
    /// /O(log(min(i,`self.len()`-i)))/
    pub fn insert(&self, i: usize, x: T) -> Seq<T> {
        if i >= self.len() {
            return self.push_back(x)
        }
        let (before,y,after) = finger_tree::split(&move |j| {i < j}, 0, self.inner());
        let before = finger_tree::snoc_node(before, node::leaf(Item(x)));
        let after = finger_tree::cons_node(y.clone(), after);
        Seq(finger_tree::tree_tree(before, after))
    }

    /// Get the element at index `i`, if it exists.
    ///
    /// /O(log(min(i,`self.len()`-i)))/
    pub fn get(&self, i: usize) -> Option<&T> {
        if i >= self.len() {
            return None
        }
        match finger_tree::lookup(move |j| {i < j}, 0, self.inner()) {
            (&Item(ref x), _) => Some(x)
        }
    }

    /// An iterator over the sequence.
    ///
    /// /O(1)/
    pub fn iter(&self) -> Iter<T> {
        self.into_iter()
    }

    fn inner(&self) -> &Lazy<FingerTree<Item<T>,usize>> {
        match *self {
            Seq(ref inner) => inner
        }
    }
}

#[macro_export]
macro_rules! seq {
    () => {
        Seq::empty()
    };
    ($e0: expr $(, $e: expr)*) => {
        seq!($($e),*).push_front($e0)
    };
}

impl<T:'static> Clone for Seq<T> {
    fn clone(&self) -> Seq<T> {
        Seq(self.inner().clone())
    }
}

#[derive(Debug)]
pub struct Iter<'a, T: 'a> {
    inner: finger_tree::Iter<'a, Item<T>, usize>
}

impl<'a,T:'static> Iter<'a,T> {
    fn new(seq: &'a Seq<T>) -> Iter<'a,T> {
        Iter {
            inner: seq.inner().iter()
        }
    }
}

impl<'a,T:'a> Iterator for Iter<'a,T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        match self.inner.next() {
            None => None,
            Some(&Item(ref x)) => Some(x)
        }
    }
}

impl<'a, T: 'static> iter::IntoIterator for &'a Seq<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a,T> {
        Iter::new(self)
    }
}

impl<T:'static> iter::FromIterator<T> for Seq<T> {
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item=T> {
        let mut iter = iter.into_iter();
        let mut seq = Seq::empty();
        while let Some(x) = iter.next() {
            seq = seq.push_back(x);
        }
        seq
    }
}

impl<T:'static> convert::From<Vec<T>> for Seq<T> {
    fn from(v: Vec<T>) -> Seq<T> {
        v.into_iter().collect()
    }
}

impl<T:'static> ops::Index<usize> for Seq<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        self.get(index).expect("Out of bounds access")
    }
}

// impl<T> ops::Index<ops::Range<usize>> for Seq<T> {
//     type Output = Seq<T>;
//     fn index(&self, index: ops::Range<usize>) -> &Seq<T> {
//         if index.start >= index.end {
//             Seq::empty()
//         } else if index.start == 0 {
//             self.index(ops::RangeTo(index.end))
//         } else if index.end >= self.len() {
//             self.index(ops::RangeFrom(index.start))
//         } else {
//             self.truncate(index.end).truncate_front(index.end - index.start)
//         }
//     }
// }

// impl<T> ops::Index<ops::RangeTo<usize>> for Seq<T> {
//     type Output = Seq<T>;
//     fn index(&self, index: ops::RangeTo<usize>) -> &Seq<T> {
//         if index.end >= self.len() {
//             self.index(ops::RangeFull)
//         } else {
//             self.truncate(index.end)
//         }
//     }
// }

// impl<T> ops::Index<ops::RangeFrom<usize>> for Seq<T> {
//     type Output = Seq<T>;
//     fn index(&self, index: ops::RangeFrom<usize>) -> &Seq<T> {
//         if index.begin >= self.len() {
//             Seq::empty()
//         }
//         if index.begin == 0 {
//             self.index(ops::RangeFull)
//         } else {
//             self.truncate_front(self.len() - index.start)
//         }
//     }
// }

// impl<T> ops::Index<ops::RangeFull> for Seq<T> {
//     type Output = Seq<T>;
//     fn index(&self, index: ops::RangeFull) -> &Seq<T> {
//         self.clone()
//     }
// }
