use std::rc::Rc;
use lazy::static_fn::Lazy;
use self::Tree23::{Leaf,Node2,Node3};
use measure::Measure;
use monoid::Monoid;

pub enum Tree23<T, M>
{
    Leaf(T),
    Node2(M, Lazy<Tree23<T,M>>, Lazy<Tree23<T,M>>),
    Node3(M, Lazy<Tree23<T,M>>, Lazy<Tree23<T,M>>, Lazy<Tree23<T,M>>),
}

impl<T,M> Tree23<T,M>
    where T: Measure<M> + 'static,
          M: Monoid + Copy + 'static
{
    pub fn leaf(value: T) -> L<Tree23<T,M>> {
        Lazy::evaluated(Leaf(value))
    }
    pub fn node2(left: L<Tree23<T,M>>, right: L<Tree23<T,M>>) -> L<Tree23<T,M>> {
        ref_lazy!({
            let measure = (**left).measure().op(right.measure());
            Node2(measure, left, right)
        })
    }
    pub fn node3(left: L<Tree23<T,M>>, middle: L<Tree23<T,M>>, right: L<Tree23<T,M>>) -> L<Tree23<T,M>> {
        ref_lazy!({
            let measure = left.measure().op(middle.measure()).op(right.measure());
            Node3(measure, left, middle, right)
        })
    }
    // pub fn node2<F0,F1>(left: F0, right: F1) -> Tree23<T,M>
    //     where F0: FnOnce() -> Tree23<T,M> + 'static,
    //           F1: FnOnce() -> Tree23<T,M> + 'static,
    //           T: Measure<M>,
    //           M: Monoid + Copy
    // {
    //     let left = Rc::new(Lazy::new(left));
    //     let right = Rc::new(Lazy::new(right));
    //     let measure = (&**left).measure()
    //         .op((&**right).measure());
    //     Node2(measure, left, right)
    // }
    // pub fn node3<F0,F1,F2>(left: F0, middle: F1, right: F2) -> Tree23<T,M>
    //     where F0: FnOnce() -> Tree23<T,M> + 'static,
    //           F1: FnOnce() -> Tree23<T,M> + 'static,
    //           F2: FnOnce() -> Tree23<T,M> + 'static,
    //           T: Measure<M>,
    //           M: Monoid + Copy
    // {
    //     let left = Rc::new(Lazy::new(left));
    //     let middle = Rc::new(Lazy::new(middle));
    //     let right = Rc::new(Lazy::new(right));
    //     let measure = (&**left).measure()
    //         .op((&**middle).measure())
    //         .op((&**right).measure());
    //     Node3(measure, left, middle, right)
    // }
}

impl<T,M> Tree23<T,M>
{
    pub fn iter<'a>(&'a self) -> Iter<'a,T,M> {
        Iter::new(self)
    }
}

pub struct Iter<'a, T:'a, M:'a> {
    stack: Vec<&'a Tree23<T,M>>,
}

impl<'a, T, M> Iter<'a, T, M> {
    fn new(node: &'a Tree23<T,M>) -> Iter<'a, T, M> {
        Iter {
            stack: vec![node],
        }
    }
    pub fn empty() -> Iter<'a, T, M> {
        Iter {
            stack: vec![],
        }
    }
}

impl<'a, T:'a, M> Iterator for Iter<'a,T,M> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        let mut node: &'a Tree23<T,M> = {
            match self.stack.pop() {
                None => return None,
                Some(node) => node
            }
        };
        loop {
            match node {
                &Leaf(ref x) => return Some(x),
                &Node2(_, ref left, ref right) => {
                    self.stack.push(&**right);
                    node = &**left;
                },
                &Node3(_, ref left, ref middle, ref right) => {
                    self.stack.push(&**right);
                    self.stack.push(&**middle);
                    node = &**left;
                }
            }
        }
    }
}

impl<T,M> Measure<M> for Tree23<T,M>
    where T: Measure<M>,
          M: Copy + Monoid
{
    fn measure(&self) -> M {
        match self {
            &Leaf(ref value) => value.measure(),
            &Node2(measure, _, _) => measure,
            &Node3(measure, _, _, _) => measure,
        }
    }
}

#[macro_export]
macro_rules! tree23 {
    ($e:expr) => {
        $crate::tree23::Tree23::leaf($e)
    };
    ($e0:expr, $e1:expr) => {
        $crate::tree23::Tree23::node2(tree23!($e0), tree23!($e1))
    };
    ($e0:expr, $e1:expr, $e2:expr) => {
        $crate::tree23::Tree23::node3(tree23!($e0), tree23!($e1), tree23!($e2))
    };
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
    fn test_tree23_iter() {
        let tree: L<Tree23<Item<u32>, usize>> =
            Tree23::node2(
                Tree23::node3(
                    Tree23::leaf(Item(0)),
                    Tree23::leaf(Item(1)),
                    Tree23::leaf(Item(2))),
                Tree23::node2(
                    Tree23::leaf(Item(3)),
                    Tree23::leaf(Item(4))));
        let result:Vec<u32> = tree.iter().map(|&Item(x)| x).collect();
        let expected:Vec<u32> = vec![0,1,2,3,4];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tree23_measure() {
        let tree: L<Tree23<Item<u32>, usize>> =
            Tree23::node2(
                Tree23::node3(
                    Tree23::leaf(Item(0)),
                    Tree23::leaf(Item(1)),
                    Tree23::leaf(Item(2))),
                Tree23::node2(
                    Tree23::leaf(Item(3)),
                    Tree23::leaf(Item(4))));
        assert_eq!(tree.measure(), 5);
    }
}
