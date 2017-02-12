use std::ops::Add;

use lazy::{Lazy,strict,value,redirect};

use digit;
use digit::Digit;
use digit::Digit::{One, Two, Three, Four};
use self::FingerTree::{Empty, Single, Deep};
use node::{Node, node3};
use node::Node::{Leaf};
use node;
use measure::Measure;
use zero::Zero;

#[derive(Debug)]
pub enum FingerTree<T,M> {
    Empty,
    Single(Lazy<Node<T,M>>),
    Deep(M, Digit<T,M>, Lazy<FingerTree<T,M>>, Digit<T,M>)
}

impl<T,M> FingerTree<T,M> {
    pub fn iter(&self) -> Iter<T,M> {
        Iter::new(self)
    }
}
pub fn empty<T,M>() -> Lazy<FingerTree<T,M>> {
    strict(Empty)
}
pub fn single<T,M>(node: Lazy<Node<T,M>>) -> Lazy<FingerTree<T,M>> {
    strict(Single(node))
}
pub fn deep<T,M>(left: Digit<T,M>, middle: Lazy<FingerTree<T,M>>, right: Digit<T,M>)
                 -> Lazy<FingerTree<T,M>>
    where T: Measure<M> + 'static,
          M: Add<Output=M> + Zero + Copy + 'static
{
    lazy_val!{
        let measure = left.measure() +
            middle.measure() +
            right.measure();
        Deep(measure, left, middle, right)
    }
}

pub fn cons_node<T,M>(x0: Lazy<Node<T,M>>, tree: Lazy<FingerTree<T,M>>)
                      -> Lazy<FingerTree<T,M>>
    where T: Measure<M> + 'static,
          M: Add<Output=M> + Zero + Copy + 'static
{
    lazy_val!{
        match *tree {
            Empty => Single(x0),
            Single(ref x1) => {
                let measure = x0.measure() + x1.measure();
                Deep(measure, One(x0), strict(Empty), One(x1.clone()))
            },
            Deep(measure, ref left, ref middle, ref right) => {
                let measure = measure + x0.measure();
                match *left {
                    Four(ref x1,ref x2,ref x3,ref x4) => {
                        let left = Two(x0.clone(),x1.clone());
                        let middle = cons_node(
                            node3(x2.clone(),x3.clone(),x4.clone()),
                            middle.clone());
                        Deep(measure, left, middle, right.clone())
                    },
                    Three(ref x1,ref x2,ref x3) => {
                        let left = Four(x0.clone(),x1.clone(),x2.clone(),x3.clone());
                        Deep(measure, left, middle.clone(), right.clone())
                    },
                    Two(ref x1,ref x2) => {
                        let left = Three(x0.clone(),x1.clone(),x2.clone());
                        Deep(measure, left, middle.clone(), right.clone())
                    },
                    One(ref x1) => {
                        let left = Two(x0.clone(),x1.clone());
                        Deep(measure, left, middle.clone(), right.clone())
                    },
                }
            }
        }
    }
}

pub fn snoc_node<T,M>(tree: Lazy<FingerTree<T,M>>, x0: Lazy<Node<T,M>>)
                      -> Lazy<FingerTree<T,M>>
    where T: Measure<M> + 'static,
          M: Add<Output=M> + Zero + Copy + 'static
{
    lazy_val!{
        match *tree {
            Empty => Single(x0),
            Single(ref x1) => {
                let measure = x1.measure() + x0.measure();
                Deep(measure, One(x1.clone()), strict(Empty), One(x0))
            },
            Deep(measure, ref left, ref middle, ref right) => {
                let measure = measure + x0.measure();
                match *right {
                    Four(ref x4,ref x3,ref x2,ref x1) => {
                        let right = Two(x1.clone(),x0.clone());
                        let middle = snoc_node(
                            middle.clone(),
                            node3(x4.clone(),x3.clone(),x2.clone()));
                        Deep(measure, left.clone(), middle, right)
                    },
                    Three(ref x3,ref x2,ref x1) => {
                        let right = Four(x3.clone(),x2.clone(),x1.clone(),x0.clone());
                        Deep(measure, left.clone(), middle.clone(), right)
                    },
                    Two(ref x2,ref x1) => {
                        let right = Three(x2.clone(),x1.clone(),x0.clone());
                        Deep(measure, left.clone(), middle.clone(), right)
                    },
                    One(ref x1) => {
                        let right = Two(x1.clone(),x0.clone());
                        Deep(measure, left.clone(), middle.clone(), right)
                    },
                }
            }
        }
    }
}

macro_rules! cons_nodes {
    ( ; $t: expr) => { $t };
    ($e0: expr $(, $e: expr)* ; $t: expr) => {
        cons_node(
            $e0,
            cons_nodes!($($e),* ; $t))
    };
}

macro_rules! snoc_nodes{
    ($t: expr ; ) => { $t };
    ($t: expr ; $e0: expr $(, $e: expr)* ) => {
        snoc_nodes!(
            snoc_node($t, $e0) ;
            $($e),*)
    };
}

fn cons_digit<T,M>(digit: Digit<T,M>, tree: Lazy<FingerTree<T,M>>)
                   -> Lazy<FingerTree<T,M>>
    where T: Measure<M> + 'static,
          M: Add<Output=M> + Zero + Copy + 'static
{
    match digit {
        One(x0) =>
            cons_nodes!(x0; tree),
        Two(x0, x1) =>
            cons_nodes!(x0, x1; tree),
        Three(x0, x1, x2) =>
            cons_nodes!(x0, x1, x2; tree),
        Four(x0, x1, x2, x3) =>
            cons_nodes!(x0, x1, x2, x3; tree),
    }
}

fn snoc_digit<T,M>(tree: Lazy<FingerTree<T,M>>, digit: Digit<T,M>)
                   -> Lazy<FingerTree<T,M>>
    where T: Measure<M> + 'static,
          M: Add<Output=M> + Zero + Copy + 'static
{
    match digit {
        One(x0) =>
            snoc_nodes!(tree; x0),
        Two(x1, x0) =>
            snoc_nodes!(tree; x1, x0),
        Three(x2, x1, x0) =>
            snoc_nodes!(tree; x2, x1, x0),
        Four(x3, x2, x1, x0) =>
            snoc_nodes!(tree; x3, x2, x1, x0),
    }
}

pub fn tree_tree<T,M>(left: Lazy<FingerTree<T,M>>, right: Lazy<FingerTree<T,M>>)
                      -> Lazy<FingerTree<T,M>>
    where T: Measure<M> + 'static,
          M: Add<Output=M> + Zero + Copy + 'static
{
    lazy!{
        if let Empty = *left {
            return redirect(right)
        };
        if let Empty = *right {
            return redirect(left)
        };
        if let Single(ref node) = *left {
            return redirect(cons_node(node.clone(), right))
        };
        if let Single(ref node) = *right {
            return redirect(snoc_node(left, node.clone()))
        };
        if let Deep(s0, ref l0, ref m0, ref r0) = *left {
            if let Deep(s1, ref l1, ref m1, ref r1) = *right {
                let s = s0 + s1;
                let l = l0.clone();
                let m = tree_digit_tree(
                    m0.clone(),
                    add_digits!(r0.clone(), l1.clone()),
                    m1.clone());
                let r = r1.clone();
                return value(Deep(s, l, m, r))
            }
        };
        unsafe {debug_unreachable!()}
    }
}

fn tree_digit_tree<T,M>(left: Lazy<FingerTree<T,M>>, d: Digit<T,M>, right: Lazy<FingerTree<T,M>>)
                        -> Lazy<FingerTree<T,M>>
    where T: Measure<M> + 'static,
          M: Add<Output=M> + Zero + Copy + 'static
{
    lazy!{
        if let Empty = *left {
            return redirect(cons_digit(d, right))
        };
        if let Empty = *right {
            return redirect(snoc_digit(left, d))
        };
        if let Single(ref node) = *left {
            return redirect(cons_node(node.clone(),
                                      cons_digit(d, right)))
        };
        if let Single(ref node) = *right {
            return redirect(snoc_node(snoc_digit(left, d),
                                      node.clone()))
        };
        if let Deep(s0, ref l0, ref m0, ref r0) = *left {
            if let Deep(s1, ref l1, ref m1, ref r1) = *right {
                let s = s0 + d.measure() + s1;
                let l = l0.clone();
                let m = tree_digit_tree(m0.clone(), add_digits!(r0.clone(), d.clone(), l1.clone()), m1.clone());
                let r = r1.clone();
                return value(Deep(s, l, m, r))
            }
        };
        unsafe {debug_unreachable!()}
    }
}

pub fn front<T,M>(tree: &Lazy<FingerTree<T,M>>) -> Option<&T> {
    let front_node = match **tree {
        Empty => return None,
        Single(ref node) => node,
        Deep(_, ref left, _, _) => match *left {
            One  (ref node)          => node,
            Two  (ref node, _)       => node,
            Three(ref node, _, _)    => node,
            Four (ref node, _, _, _) => node,
        }
    };
    match **front_node {
        Leaf(ref x) => Some(x),
        _ => unsafe { debug_unreachable!() },
    }
}

pub fn back<T,M>(tree: &Lazy<FingerTree<T,M>>) -> Option<&T> {
    let back_node = match **tree {
        Empty => return None,
        Single(ref node) => node,
        Deep(_, _, _, ref right) => match *right {
            One  (ref node)          => node,
            Two  (_, ref node)       => node,
            Three(_, _, ref node)    => node,
            Four (_, _, _, ref node) => node,
        }
    };
    match **back_node {
        Leaf(ref x) => Some(x),
        _ => unsafe { debug_unreachable!() },
    }
}

impl<'a,T,M> From<&'a Digit<T,M>> for Lazy<FingerTree<T,M>>
    where T: Measure<M> + 'static,
          M: Add<Output=M> + Zero + Copy + 'static
{
    fn from(digit: &'a Digit<T,M>) -> Lazy<FingerTree<T,M>> {
        match *digit {
            One(ref x0) =>
                single(x0.clone()),
            Two(ref x0, ref x1) =>
                deep(One(x0.clone()), empty(), One(x1.clone())),
            Three(ref x0, ref x1, ref x2) =>
                deep(Two(x0.clone(), x1.clone()), empty(), One(x2.clone())),
            Four(ref x0, ref x1, ref x2, ref x3) =>
                deep(Two(x0.clone(), x1.clone()), empty(), Two(x2.clone(), x3.clone())),
        }
    }
}

impl<T,M> From<Digit<T,M>> for Lazy<FingerTree<T,M>>
    where T: Measure<M> + 'static,
          M: Add<Output=M> + Zero + Copy + 'static
{
    fn from(digit: Digit<T,M>) -> Lazy<FingerTree<T,M>> {
        (&digit).into()
    }
}

impl<T,M> From<Option<Digit<T,M>>> for Lazy<FingerTree<T,M>>
    where T: Measure<M> + 'static,
          M: Add<Output=M> + Zero + Copy + 'static
{
    fn from(digit: Option<Digit<T,M>>) -> Lazy<FingerTree<T,M>> {
        match digit {
            None => empty(),
            Some(digit) => digit.into(),
        }
    }
}

fn viewl_node<T,M>(tree: &Lazy<FingerTree<T,M>>) -> (Option<&Node<T,M>>, Lazy<FingerTree<T,M>>)
    where T: Measure<M> + 'static,
          M: Add<Output=M> + Zero + Copy + 'static
{
    match **tree {
        Empty => (None, empty()),
        Single(ref node) => (Some(node),empty()),
        Deep(_, ref left, ref middle, ref right) =>
            match *left {
                Two(ref x0, ref x1) =>
                    (Some(x0), deep(One(x1.clone()),middle.clone(), right.clone())),
                Three(ref x0, ref x1, ref x2) =>
                    (Some(x0), deep(Two(x1.clone(),x2.clone()),middle.clone(), right.clone())),
                Four(ref x0, ref x1, ref x2, ref x3) =>
                    (Some(x0), deep(Three(x1.clone(),x2.clone(),x3.clone()),middle.clone(), right.clone())),
                One(ref x0) => {
                    let remx = match viewl_node(middle) {
                        (None, _) =>
                            right.into(),
                        (Some(y),remy) =>
                            deep(y.into(), remy, right.clone())
                    };
                    (Some(x0), remx)
                }
            }
    }
}

pub fn pop_front<T,M>(tree: &Lazy<FingerTree<T,M>>) -> Lazy<FingerTree<T,M>>
    where T: Measure<M> + 'static,
          M: Add<Output=M> + Zero + Copy + 'static
{
    match viewl_node(tree) {
        (_,rem) => rem,
    }
}


fn viewr_node<T,M>(tree: &Lazy<FingerTree<T,M>>) -> (Lazy<FingerTree<T,M>>, Option<&Node<T,M>>)
    where T: Measure<M> + 'static,
          M: Add<Output=M> + Zero + Copy + 'static
{
    match **tree {
        Empty => (empty(), None),
        Single(ref node) => (empty(), Some(node)),
        Deep(_, ref left, ref middle, ref right) =>
            match *right {
                Two(ref x1,ref x0) =>
                    (deep(left.clone(), middle.clone(), One(x1.clone())), Some(x0)),
                Three(ref x2,ref x1,ref x0) =>
                    (deep(left.clone(), middle.clone(), Two(x2.clone(), x1.clone())), Some(x0)),
                Four(ref x3,ref x2,ref x1,ref x0) =>
                    (deep(left.clone(), middle.clone(), Three(x3.clone(), x2.clone(), x1.clone())), Some(x0)),
                One(ref x0) => {
                    let remx = match viewr_node(middle) {
                        (_, None) =>
                            left.into(),
                        (remy, Some(y)) =>
                            deep(left.clone(), remy, y.into())
                    };
                    (remx, Some(x0))
                }
            }
    }
}

pub fn pop_back<T,M>(tree: &Lazy<FingerTree<T,M>>) -> Lazy<FingerTree<T,M>>
    where T: Measure<M> + 'static,
          M: Add<Output=M> + Zero + Copy + 'static
{
    match viewr_node(tree) {
        (rem,_) => rem,
    }
}

pub fn lookup<T,M,P>(pred: P, i: M, tree: &Lazy<FingerTree<T,M>>) -> (&T,M)
    where T: Measure<M> + 'static,
          M: Add<Output=M> + Zero + Copy + 'static,
          P: Fn(M) -> bool
{
    match **tree {
        Empty => panic!("lookup in empty tree"),
        Single(ref node) => node::lookup(pred, i, node),
        Deep(_, ref left, ref middle, ref right) => {
            let i1 = i + left.measure();
            if pred(i1) {
                return digit::lookup(pred, i, left)
            }
            let i2 = i1 + middle.measure();
            if pred(i2) {
                lookup(pred, i1, middle)
            } else {
                digit::lookup(pred, i2, right)
            }
        }
    }
}

pub fn adjust<T,M,P,F>(func: F, pred: P, i: M, tree: &Lazy<FingerTree<T,M>>) -> Lazy<FingerTree<T,M>>
    where T: Measure<M> + 'static,
          M: Add<Output=M> + Zero + Copy + 'static,
          P: Fn(M) -> bool,
          F: FnOnce(&T) -> T
{
    match **tree {
        Empty => tree.clone(),
        Single(ref node) =>
            single(node::adjust(func, pred, i, node)),
        Deep(_, ref left, ref middle, ref right) => {
            let i1 = i + left.measure();
            if pred(i1) {
                return deep(digit::adjust(func, pred, i, left), middle.clone(), right.clone())
            }
            let i2 = i1 + middle.measure();
            if pred(i2) {
                deep(left.clone(), adjust(func, pred, i1, middle), right.clone())
            } else {
                deep(left.clone(), middle.clone(), digit::adjust(func, pred, i2, right))
            }
        }
    }
}

fn deep_left<T,M>(left: Option<Digit<T,M>>, middle: Lazy<FingerTree<T,M>>, right: Digit<T,M>)
              -> Lazy<FingerTree<T,M>>
    where T: Measure<M> + 'static,
          M: Add<Output=M> + Zero + Copy + 'static
{
    match left {
        Some(left) => deep(left, middle, right),
        None => {
            match viewl_node(&middle) {
                (None,_) => right.into(),
                (Some(node), rem) =>
                    deep(node.into(), rem, right.clone())
            }
        }
    }
}

fn deep_right<T,M>(left: Digit<T,M>, middle: Lazy<FingerTree<T,M>>, right: Option<Digit<T,M>>)
              -> Lazy<FingerTree<T,M>>
    where T: Measure<M> + 'static,
          M: Add<Output=M> + Zero + Copy + 'static
{
    match right {
        Some(right) => deep(left, middle, right),
        None => {
            match viewr_node(&middle) {
                (_, None) => left.into(),
                (rem, Some(node)) =>
                    deep(left.clone(), rem, node.into())
            }
        }
    }
}

pub fn split<'a,T,M,P>(pred: &P, i: M, tree: &'a FingerTree<T,M>)
                     -> (Lazy<FingerTree<T,M>>,&'a Lazy<Node<T,M>>,Lazy<FingerTree<T,M>>)
    where T: Measure<M> + 'static,
          M: Add<Output=M> + Zero + Copy + 'static,
          P: Fn(M) -> bool
{
    match *tree {
        Empty => panic!("split in empty tree"),
        Single(ref node) => {
            (empty(), node, empty())
            // let (before,x,after) = node::split_once(pred, i, node);
            // let before = before.into();
            // let after = after.into();
            // (before, x, after)
        },
        Deep(_, ref left, ref middle, ref right) => {
            let i1 = i + left.measure();
            if pred(i1) {
                let (before,x,after) = digit::split_once(pred, i, left);
                let before:Lazy<FingerTree<T,M>> = before.into();
                let after = deep_left(after, middle.clone(), right.clone());
                return (before, x, after)
            }
            let i2 = i1 + middle.measure();
            if pred(i2) {
                let (before,node,after) = split(pred, i1, middle);
                let i_node = i1 + before.measure();
                let (node_before, x, node_after) = node::split_once(pred, i_node, node);
                let before = deep_right(left.clone(), before, node_before);
                let after = deep_left(node_after, after, right.clone());
                (before, x, after)
            } else {
                let (before,x,after) = digit::split_once(pred, i2, right);
                let before = deep_right(left.clone(), middle.clone(), before);
                let after:Lazy<FingerTree<T,M>> = after.into();
                (before, x, after)
            }
        }
    }
}

#[derive(Debug)]
pub enum IterFrame<'a, T:'a, M:'a> {
    NodeFrame(&'a Node<T,M>),
    FingerTreeFrame(&'a FingerTree<T,M>),
}
use self::IterFrame::{NodeFrame, FingerTreeFrame};

#[derive(Debug)]
pub struct Iter<'a, T:'a, M:'a> {
    stack: Vec<IterFrame<'a,T,M>>,
    inner: node::Iter<'a, T, M>,
}

impl<'a, T, M> Iter<'a, T, M> {
    fn new(node: &'a FingerTree<T,M>) -> Iter<'a, T, M> {
        Iter {
            stack: vec![FingerTreeFrame(node)],
            inner: node::Iter::empty(),
        }
    }
    fn push_digit(&mut self, digit: &'a Digit<T,M>) {
        match *digit {
            One(ref x0) =>
                self.stack.push(NodeFrame(x0)),
            Two(ref x0,ref x1) => {
                self.stack.push(NodeFrame(x1)); self.stack.push(NodeFrame(x0));},
            Three(ref x0,ref x1,ref x2) => {
                self.stack.push(NodeFrame(x2)); self.stack.push(NodeFrame(x1));
                self.stack.push(NodeFrame(x0));},
            Four(ref x0,ref x1,ref x2,ref x3) => {
                self.stack.push(NodeFrame(x3)); self.stack.push(NodeFrame(x2));
                self.stack.push(NodeFrame(x1)); self.stack.push(NodeFrame(x0));},
        }
    }
}

impl<'a, T:'a, M> Iterator for Iter<'a,T,M> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        if let v@Some(_) = self.inner.next() {
            return v
        }
        loop {
            match self.stack.pop() {
                None => return None,
                Some(NodeFrame(node)) => {
                    self.inner = node.iter();
                    return self.inner.next();
                },
                Some(FingerTreeFrame(tree)) => {
                    match *tree {
                        Empty => {},
                        Single(ref x) => {
                            self.inner = x.iter();
                            return self.inner.next();
                        }
                        Deep(_, ref left, ref middle, ref right) => {
                            self.push_digit(right);
                            self.stack.push(FingerTreeFrame(middle));
                            self.push_digit(left);
                        }
                    }
                },
            }
        }
    }
}

impl<'a, T, M> IntoIterator for &'a FingerTree<T,M> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T, M>;

    fn into_iter(self) -> Iter<'a,T,M> {
        self.iter()
    }
}

impl<T,M> Measure<M> for FingerTree<T,M>
    where T: Measure<M>,
          M: Add<Output=M> + Zero + Copy {
    fn measure(&self) -> M {
        match *self {
            Empty => M::zero(),
            Single(ref x) => x.measure(),
            Deep(measure,_,_,_) => measure
        }
    }
}

#[cfg(test)]
mod test {
    use super::FingerTree;
    use super::*;
    use node::{leaf, node2, node3};
    use digit::Digit::{One,Two};

    #[derive(Clone)]
    struct Item<T>(T);

    impl<T> Measure<usize> for Item<T> {
        fn measure(&self) -> usize {
            1
        }
    }

    #[test]
    fn test_iter_empty() {
        let tree: Lazy<FingerTree<Item<u32>, usize>> =
            empty();
        let result:Vec<u32> = tree.iter().map(|&Item(x)| x).collect();
        let expected:Vec<u32> = vec![];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_iter_single() {
        let tree: Lazy<FingerTree<Item<u32>, usize>> =
            single(leaf(Item(0)));
        let result:Vec<u32> = tree.iter().map(|&Item(x)| x).collect();
        let expected:Vec<u32> = vec![0];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_iter_inner_empty() {
        let tree: Lazy<FingerTree<Item<u32>, usize>> =
            deep(
                One(
                    node2(
                        leaf(Item(0)),
                        leaf(Item(1)))),
                deep(
                    One(
                        node2(
                            node3(
                                leaf(Item(2)),
                                leaf(Item(3)),
                                leaf(Item(4))),
                            node2(
                                leaf(Item(5)),
                                leaf(Item(6))))),
                    empty(),
                    One(
                        node2(
                            node2(
                                leaf(Item(7)),
                                leaf(Item(8))),
                            node2(
                                leaf(Item(9)),
                                leaf(Item(10)))))),
                Two(
                    node2(
                        leaf(Item(11)),
                        leaf(Item(12))),
                    node2(
                        leaf(Item(13)),
                        leaf(Item(14)))));
        let result:Vec<u32> = tree.iter().map(|&Item(x)| x).collect();
        let expected:Vec<u32> = (0..15).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_iter_inner_single() {
        let tree: Lazy<FingerTree<Item<u32>, usize>> =
            deep(
                One(
                    node2(
                        leaf(Item(0)),
                        leaf(Item(1)))),
                deep(
                    One(
                        node2(
                            node3(
                                leaf(Item(2)),
                                leaf(Item(3)),
                                leaf(Item(4))),
                            node2(
                                leaf(Item(5)),
                                leaf(Item(6))))),
                    single(
                        node2(
                            node2(
                                leaf(Item(7)),
                                leaf(Item(8))),
                            node2(
                                leaf(Item(9)),
                                leaf(Item(10))))),
                    One(
                        node2(
                            node2(
                                leaf(Item(11)),
                                leaf(Item(12))),
                            node2(
                                leaf(Item(13)),
                                leaf(Item(14)))))),
                Two(
                    node2(
                        leaf(Item(15)),
                        leaf(Item(16))),
                    node2(
                        leaf(Item(17)),
                        leaf(Item(18)))));
        let result:Vec<u32> = tree.iter().map(|&Item(x)| x).collect();
        let expected:Vec<u32> = (0..19).collect();
        assert_eq!(result, expected);
    }
}
