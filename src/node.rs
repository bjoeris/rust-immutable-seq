use std::ops;

use lazy::{Lazy, strict, value};
use self::Node::{Leaf,Node2,Node3};
use measure::Measure;
use digit::Digit;
use digit::Digit::{One,Two};

/// A node in a 2-3 tree.
///
/// The children are stared as lazy references
#[derive(Debug)]
pub enum Node<T, M>
{
    Leaf(T),
    Node2(M, Lazy<Node<T,M>>, Lazy<Node<T,M>>),
    Node3(M, Lazy<Node<T,M>>, Lazy<Node<T,M>>, Lazy<Node<T,M>>),
}

/// Construct a lazy reference to a leaf
pub fn leaf<T,M>(v: T) -> Lazy<Node<T,M>> {
    strict(Leaf(v))
}

/// Construct a lazy reference to a node with two children
pub fn node2<T,M>(left: Lazy<Node<T,M>>, right: Lazy<Node<T,M>>) -> Lazy<Node<T,M>>
    where T: Measure<M> + 'static,
          M: ops::Add<Output=M> + Copy + 'static
{
    lazy!{
        let m = left.measure() + right.measure();
        let left: Lazy<Node<T,M>> = left;
        value(Node2(m, left, right))
    }
}

/// Construct a lazy reference to a node with three children
pub fn node3<T,M>(left: Lazy<Node<T,M>>, middle: Lazy<Node<T,M>>, right: Lazy<Node<T,M>>) -> Lazy<Node<T,M>>
    where T: Measure<M> + 'static,
          M: ops::Add<Output=M> + Copy + 'static
{
    lazy!{
        let m = left.measure() + middle.measure() + right.measure();
        value(Node3(m, left, middle, right))
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! node {
    ($left: expr, $right: expr) => {
        Node::node2($left, $right)
    };
    ($left: expr, $middle: expr, $right: expr) => {
        Node::node3($left, $middle, $right)
    }
}

pub fn lookup<T,M,P>(pred: P, i: M, node: &Node<T,M>) -> (&T,M)
    where T: Measure<M> + 'static,
          M: ops::Add<Output=M> + Copy + 'static,
          P: Fn(M) -> bool
{
    match *node {
        Leaf(ref x) => (x, i),
        Node2(_, ref left, ref right) => {
            let i1 = i + left.measure();
            if pred(i1) {
                lookup(pred, i, left)
            } else {
                lookup(pred, i1, right)
            }
        },
        Node3(_, ref left, ref middle, ref right) => {
            let i1 = i + left.measure();
            if pred(i1) {
                lookup(pred, i, left)
            } else {
                let i2 = i1 + middle.measure();
                if pred(i2) {
                    lookup(pred, i1, middle)
                } else {
                    lookup(pred, i2, right)
                }
            }
        }
    }
}

pub fn adjust<T,M,P,F>(func: F, pred: P, i: M, node: &Node<T,M>) -> Lazy<Node<T,M>>
    where T: Measure<M> + 'static,
          M: ops::Add<Output=M> + Copy + 'static,
          P: Fn(M) -> bool,
          F: FnOnce(&T) -> T
{
    match *node {
        Leaf(ref x) => leaf(func(x)),
        Node2(_, ref left, ref right) => {
            let i1 = i + left.measure();
            if pred(i1) {
                node2(adjust(func, pred, i, left), right.clone())
            } else {
                node2(left.clone(), adjust(func, pred, i1, right))
            }
        },
        Node3(_, ref left, ref middle, ref right) => {
            let i1 = i + left.measure();
            if pred(i1) {
                node3(adjust(func, pred, i, left), middle.clone(), right.clone())
            } else {
                let i2 = i1 + middle.measure();
                if pred(i2) {
                    node3(left.clone(), adjust(func, pred, i1, middle), right.clone())
                } else {
                    node3(left.clone(), middle.clone(), adjust(func, pred, i2, right))
                }
            }
        }
    }
}

pub fn split_once<'a,T,M,P>(pred: &P, i: M, node: &'a Node<T,M>)
                    -> (Option<Digit<T,M>>, &'a Lazy<Node<T,M>>, Option<Digit<T,M>>)
    where T: Measure<M> + 'static,
          M: ops::Add<Output=M> + Copy + 'static,
          P: Fn(M) -> bool
{
    match *node {
        Leaf(_) => panic!("split_once on Leaf"),
        Node2(_, ref left, ref right) => {
            let i1 = i + left.measure();
            if pred(i1) {
                (None, left, Some(One(right.clone())))
            } else {
                (Some(One(left.clone())), right, None)
            }
        },
        Node3(_, ref left, ref middle, ref right) => {
            let i1 = i + left.measure();
            if pred(i1) {
                return (None, left, Some(Two(middle.clone(), right.clone())))
            }
            let i2 = i1 + middle.measure();
            if pred(i2) {
                (Some(One(left.clone())), middle, Some(One(right.clone())))
            } else {
                (Some(Two(left.clone(), middle.clone())), right, None)
            }
        }
    }
}

impl<T,M> Node<T,M>
{
    /// Iterates over the values in the leaves
    pub fn iter<'a>(&'a self) -> Iter<'a,T,M> {
        Iter::new(self)
    }
}

impl<'a,T,M> IntoIterator for &'a Node<T,M> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T, M>;

    fn into_iter(self) -> Iter<'a,T,M> {
        self.iter()
    }
}

impl<T,M> Measure<M> for Node<T,M>
    where T: Measure<M>,
          M: Copy
{
    fn measure(&self) -> M {
        match self {
            &Leaf(ref value) => value.measure(),
            &Node2(measure, _, _) => measure,
            &Node3(measure, _, _, _) => measure,
        }
    }
}

/// Iterator over the values in the leaves of a 2-3 tree
#[derive(Debug)]
pub struct Iter<'a, T:'a, M:'a> {
    stack: Vec<&'a Node<T,M>>,
}

impl<'a, T, M> Iter<'a, T, M> {
    fn new(node: &'a Node<T,M>) -> Iter<'a, T, M> {
        Iter {
            stack: vec![node],
        }
    }
    /// An `iter` that is empty (yields no values).
    ///
    /// This is helpful in  `finger_tree::Iter`.
    pub fn empty() -> Iter<'a, T, M> {
        Iter {
            stack: vec![],
        }
    }
}

impl<'a, T:'a, M> Iterator for Iter<'a,T,M> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        let mut node: &'a Node<T,M> = {
            match self.stack.pop() {
                None => return None,
                Some(node) => node
            }
        };
        loop {
            match *node {
                Leaf(ref x) => return Some(x),
                Node2(_, ref left, ref right) => {
                    self.stack.push(&*right);
                    node = &*left;
                },
                Node3(_, ref left, ref middle, ref right) => {
                    self.stack.push(&*right);
                    self.stack.push(&*middle);
                    node = &*left;
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use measure::Measure;

    struct Item<T>(T);

    impl<T> Measure<usize> for Item<T> {
        fn measure(&self) -> usize {
            1
        }
    }

    #[test]
    fn test_node_iter() {
        let tree: Lazy<Node<Item<u32>, usize>> =
            node2(
                node3(
                    leaf(Item(0)),
                    leaf(Item(1)),
                    leaf(Item(2))),
                node2(
                    leaf(Item(3)),
                    leaf(Item(4))));
        let result:Vec<u32> = tree.iter().map(|&Item(x)| x).collect();
        let expected:Vec<u32> = vec![0,1,2,3,4];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tree23_measure() {
        let tree: Lazy<Node<Item<u32>, usize>> =
            node2(
                node3(
                    leaf(Item(0)),
                    leaf(Item(1)),
                    leaf(Item(2))),
                node2(
                    leaf(Item(3)),
                    leaf(Item(4))));
        assert_eq!(tree.measure(), 5);
    }
}
