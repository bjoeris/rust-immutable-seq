use std::ops;

use lazy::Lazy;
use measure::Measure;
use node;
use node::Node;
use node::Node::{Node2, Node3};
use self::Digit::{One, Two, Three, Four};
use self::IterDigit::{IZero, IOne, ITwo, IThree};

/// A short sequence of 2-3-trees.
#[derive(Debug)]
pub enum Digit<T,M> {
    One  (Lazy<Node<T,M>>),
    Two  (Lazy<Node<T,M>>, Lazy<Node<T,M>>),
    Three(Lazy<Node<T,M>>, Lazy<Node<T,M>>, Lazy<Node<T,M>>),
    Four (Lazy<Node<T,M>>, Lazy<Node<T,M>>, Lazy<Node<T,M>>, Lazy<Node<T,M>>),
}

impl<T,M> Digit<T,M> {
    /// Iterate of the items in the 2-3-trees in this digit.
    pub fn iter<'a>(&'a self) -> Iter<'a, T, M> {
        Iter::new(self)
    }
}

impl<'a,T,M> From<&'a Node<T,M>> for Digit<T,M> {
    fn from(node: &'a Node<T,M>) -> Digit<T,M> {
        match *node {
            Node2(_, ref x0, ref x1) =>
                Two(x0.clone(), x1.clone()),
            Node3(_, ref x0, ref x1, ref x2) =>
                Three(x0.clone(), x1.clone(), x2.clone()),
            _ => unreachable!(),
        }
    }
}

impl<T,M> Clone for Digit<T,M> {
    fn clone(&self) -> Digit<T,M> {
        match *self {
            One(ref x0) =>
                One(x0.clone()),
            Two(ref x0, ref x1) =>
                Two(x0.clone(), x1.clone()),
            Three(ref x0, ref x1, ref x2) =>
                Three(x0.clone(), x1.clone(), x2.clone()),
            Four(ref x0, ref x1, ref x2, ref x3) =>
                Four(x0.clone(), x1.clone(), x2.clone(), x3.clone()),
        }
    }
}

impl<T,M> Measure<M> for Digit<T,M>
    where T: Measure<M>,
          M: ops::Add<Output=M> + Copy {
    fn measure(&self) -> M {
        match *self {
            One(ref x0) =>
                x0.measure(),
            Two(ref x0, ref x1) =>
                x0.measure() + x1.measure(),
            Three(ref x0, ref x1, ref x2) =>
                x0.measure() + x1.measure() + x2.measure(),
            Four(ref x0, ref x1, ref x2, ref x3) =>
                x0.measure() + x1.measure() + x2.measure() + x3.measure(),
        }
    }
}

/// A suffix of a digit, used to track the state in `Iter`
#[derive(Debug)]
pub enum IterDigit<'a, T: 'a, M: 'a> {
    IZero,
    IOne  (&'a Lazy<Node<T,M>>),
    ITwo  (&'a Lazy<Node<T,M>>, &'a Lazy<Node<T,M>>),
    IThree(&'a Lazy<Node<T,M>>, &'a Lazy<Node<T,M>>, &'a Lazy<Node<T,M>>),
}

/// An iterator over the values in the leaves of the 2-3-trees in this digit
#[derive(Debug)]
pub struct Iter<'a, T: 'a, M: 'a> {
    digit: IterDigit<'a,T,M>,
    inner: node::Iter<'a,T,M>,
}

impl<'a, T, M> Iter<'a, T, M> {
    fn new(digit: &'a Digit<T,M>) -> Iter<'a, T, M> {
        match *digit {
            One(ref x0) => Iter {
                digit: IZero,
                inner: x0.iter(),
            },
            Two(ref x0, ref x1) => Iter {
                digit: IOne(x1),
                inner: x0.iter(),
            },
            Three(ref x0, ref x1, ref x2) => Iter {
                digit: ITwo(x1,x2),
                inner: x0.iter(),
            },
            Four(ref x0, ref x1, ref x2, ref x3) => Iter {
                digit: IThree(x1,x2,x3),
                inner: x0.iter(),
            },
        }
    }
    /// An `Iter` that is empty (yield no values)
    ///
    /// This is helpful in  `finger_tree::Iter`.
    pub fn empty() -> Iter<'a, T, M> {
        Iter {
            inner: node::Iter::empty(),
            digit: IZero,
        }
    }
}

impl<'a, T:'a, M:'a> Iterator for Iter<'a, T, M> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        loop {
            match self.inner.next() {
                Some(x) => return Some(x),
                None => {}
            };
            match self.digit {
                IZero => return None,
                IOne(x0) => {
                    self.digit = IZero;
                    self.inner = x0.iter();
                },
                ITwo(x0,x1) => {
                    self.digit = IOne(x1);
                    self.inner = x0.iter();
                },
                IThree(x0,x1,x2) => {
                    self.digit = ITwo(x1,x2);
                    self.inner = x0.iter();
                },
            }
        }
    }
}

#[macro_export]
macro_rules! digit {
    ($e0: expr) => {
        $crate::digit::Digit::One($e0)
    };
    ($e0: expr, $e1: expr) => {
        $crate::digit::Digit::Two($e0, $e1)
    };
    ($e0: expr, $e1: expr, $e2: expr) => {
        $crate::digit::Digit::Three($e0, $e1, $e2)
    };
    ($e0: expr, $e1: expr, $e2: expr, $e3: expr) => {
        $crate::digit::Digit::Four($e0, $e1, $e2, $e3)
    };
}

#[macro_export]
macro_rules! opt_digit {
    () => {
        ::std::option::Option::None
    };
    ($($e: expr),+) => {
        ::std::option::Option::Some(digit!($($e),*))
    }
}

#[macro_export]
macro_rules! build_digit {
    ($($f0: expr, $f1: expr, $f2: expr),+) => {
        digit!($($crate::node::node3($f0,$f1,$f2)),*)
    };
    ($e0: expr, $e1: expr $(, $f0: expr, $f1: expr, $f2: expr)*) => {
        digit!($crate::node::node2($e0, $e1)
               $(, $crate::node::node3($f0,$f1,$f2))*)
    };
    ($e0: expr, $e1: expr, $e2: expr, $e3: expr $(, $f0: expr, $f1: expr, $f2: expr)*) => {
        digit!($crate::node::node2($e0, $e1),
               $crate::node::node2($e2, $e3)
               $(, $crate::node::node3($f0,$f1,$f2))*)
    };
}

#[macro_export]
macro_rules! add_digits {
    ( ; $($e: expr),*) => {
        build_digit!($($e),*)
    };
    ($d0: expr $(, $d: expr)* ; $($e: expr),*) => {
        match $d0 {
            One(x0) =>
                add_digits!($($d),* ; $($e, )* x0),
            Two(x0, x1) =>
                add_digits!($($d),* ; $($e, )* x0, x1),
            Three(x0, x1, x2) =>
                add_digits!($($d),* ; $($e, )* x0, x1, x2),
            Four(x0, x1, x2, x3) =>
                add_digits!($($d),* ; $($e, )* x0, x1, x2, x3),
        }
    };
    ($($e:expr),*) => {
        add_digits!($($e),*;)
    }
}

pub fn add_2_digits<T,M>(d0: Digit<T,M>, d1: Digit<T,M>)
                    -> Digit<T,M>
    where T: Measure<M> + 'static,
          M: ops::Add<Output=M> + Copy + 'static
{
    add_digits!(d0, d1 ; )
}

pub fn add_3_digits<T,M>(d0: Digit<T,M>, d1: Digit<T,M>, d2: Digit<T,M>)
                    -> Digit<T,M>
    where T: Measure<M> + 'static,
          M: ops::Add<Output=M> + Copy + 'static
{
    add_digits!(d0, d1, d2 ; )
}

macro_rules! lookup {
    ($pred: expr, $i: expr ; $n0: expr) => {
        node::lookup($pred, $i, $n0)
    };
    ($pred: expr, $i: expr ; $n0: expr $(, $n: expr)*) => {{
        let j = $i + $n0.measure();
        if $pred(j) {
            node::lookup($pred, $i, $n0)
        } else {
            lookup!($pred, j ; $($n),*)
        }
    }};
}

// macro_rules! lookup {
//     ($node: expr, $func: expr, $i: expr ; $n0: expr) => {
//         $node($func, $i, $n0)
//     };
//     ($node: expr, $func: expr, $i: expr ; $n0: expr $(, $n: expr)*) => {{
//         let j = $i + $n0.measure();
//         if $func(j) {
//             $node($func, $i, $n0)
//         } else {
//             lookup!($node, $func, j ; $($n),*)
//         }
//     }};
// }

pub fn lookup<T,M,P>(pred: P, i: M, digit: &Digit<T,M>) -> (&T,M)
    where T: Measure<M> + 'static,
          M: ops::Add<Output=M> + Copy + 'static,
          P: Fn(M) -> bool
{
    match *digit {
        One(ref x0) =>
            lookup!(pred, i ; x0),
        Two(ref x0, ref x1) =>
            lookup!(pred, i ; x0, x1),
        Three(ref x0, ref x1, ref x2) =>
            lookup!(pred, i ; x0, x1, x2),
        Four(ref x0, ref x1, ref x2, ref x3) =>
            lookup!(pred, i ; x0, x1, x2, x3),
    }
}

macro_rules! adjust {
    ($func: expr, $pred: expr, $i: expr $(, $b: expr)*; $n0: expr) => {
        digit!($($b.clone() , )* node::adjust($func, $pred, $i, $n0))
    };
    ($func: expr, $pred: expr, $i: expr $(, $b: expr)* ; $n0: expr $(, $n: expr)*) => {{
        let j = $i + $n0.measure();
        if $pred(j) {
            digit!($($b.clone() , )* node::adjust($func, $pred, $i, $n0) $(, $n.clone() )*)
        } else {
            adjust!($func, $pred, j $(, $b)* , $n0 ; $($n),*)
        }
    }};
}

pub fn adjust<T,M,P,F>(func: F, pred: P, i: M, digit: &Digit<T,M>) -> Digit<T,M>
    where T: Measure<M> + 'static,
          M: ops::Add<Output=M> + Copy + 'static,
          P: Fn(M) -> bool,
          F: FnOnce(&T) -> T
{
    match *digit {
        One(ref x0) =>
            adjust!(func, pred, i ; x0),
        Two(ref x0, ref x1) =>
            adjust!(func, pred, i ; x0, x1),
        Three(ref x0, ref x1, ref x2) =>
            adjust!(func, pred, i ; x0, x1, x2),
        Four(ref x0, ref x1, ref x2, ref x3) =>
            adjust!(func, pred, i ; x0, x1, x2, x3),
    }
}

macro_rules! split_once {
    ($pred: expr, $i: expr $(, $b: expr)* ; $n0: expr) => {
        (opt_digit!($( $b.clone() ),*) , $n0, ::std::option::Option::None)
    };
    ($pred: expr, $i: expr $(, $b: expr)* ; $n0: expr $(, $n: expr)*) => {{
        let j = $i + $n0.measure();
        if $pred(j) {
            (opt_digit!($( $b.clone() ),*) , $n0 , opt_digit!($( $n.clone() ),*))
        } else {
            split_once!($pred, j, $($b , )* $n0 ; $($n),*)
        }
    }};
}

pub fn split_once<'a,T,M,P>(pred: &P, i: M, digit: &'a Digit<T,M>)
                    -> (Option<Digit<T,M>>,&'a Lazy<Node<T,M>>,Option<Digit<T,M>>)
    where T: Measure<M> + 'static,
          M: ops::Add<Output=M> + Copy + 'static,
          P: Fn(M) -> bool
{
    match *digit {
        One(ref x0) =>
            split_once!(pred, i ; x0),
        Two(ref x0, ref x1) =>
            split_once!(pred, i ; x0, x1),
        Three(ref x0, ref x1, ref x2) =>
            split_once!(pred, i ; x0, x1, x2),
        Four(ref x0, ref x1, ref x2, ref x3) =>
            split_once!(pred, i ; x0, x1, x2, x3),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use node::leaf;

    #[test]
    fn test_digit_iter() {
        let digit: Digit<u32,usize> = digit!(
            leaf(0),
            leaf(1),
            leaf(2),
            leaf(3));
        let result:Vec<u32> = digit.iter().map(|x| *x).collect();
        let expected:Vec<u32> = vec![0,1,2,3];
        assert_eq!(result,expected);
    }
}
