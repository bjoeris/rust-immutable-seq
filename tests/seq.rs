#[macro_use]
extern crate immutable_seq;

use immutable_seq::seq::Seq;

#[test]
fn test_iter_empty() {
    let result: Vec<u32> = Seq::empty().iter().map(|x| *x).collect();
    let expected: Vec<u32> = vec![];
    assert_eq!(result, expected);
}

#[test]
fn test_empty_is_empty() {
    assert!(Seq::<u32>::empty().is_empty());
}

#[test]
fn test_empty_len() {
    assert_eq!(Seq::<u32>::empty().len(), 0);
}

#[test]
fn test_iter_singleton() {
    let result: Vec<u32> = Seq::singleton(42).iter().map(|x| *x).collect();
    let expected: Vec<u32> = vec![42];
    assert_eq!(result, expected);
}

#[test]
fn test_singleton_not_is_empty() {
    assert!(! Seq::singleton(7u32).is_empty());
}

#[test]
fn test_singleton_len() {
    assert_eq!(Seq::singleton(7u32).len(), 1);
}

#[test]
fn test_iter_push_front() {
    let mut seq: Seq<u32> = Seq::empty();
    for i in 0..20 {
        seq = seq.push_front(i);
    }
    let result: Vec<u32> = seq.iter().map(|x| *x).collect();
    let mut expected: Vec<u32> = (0..20).into_iter().collect();
    expected.reverse();
    assert_eq!(result, expected);
}

#[test]
fn test_iter_push_back() {
    let mut seq: Seq<u32> = Seq::empty();
    for i in 0..20 {
        seq = seq.push_back(i);
    }
    let result: Vec<u32> = seq.iter().map(|x| *x).collect();
    let expected: Vec<u32> = (0..20).into_iter().collect();
    assert_eq!(result, expected);
}

#[test]
fn test_vec_macro() {
    let seq: Seq<u32> = seq![0,1,2,3,4,5,6,7,8,9];
    let result: Vec<u32> = seq.iter().map(|x| *x).collect();
    let expected: Vec<u32> = vec![0,1,2,3,4,5,6,7,8,9];
    assert_eq!(result, expected);
}

#[test]
fn test_iter_append() {
    let mut xs: Seq<u32> = Seq::empty();
    let mut ys: Seq<u32> = Seq::empty();
    for i in 0..20 {
        xs = xs.push_back(i);
        ys = ys.push_back(20 + i);
    }
    let seq = xs.append(&ys);
    let result: Vec<u32> = seq.iter().map(|x| *x).collect();
    let expected: Vec<u32> = (0..40).into_iter().collect();
    assert_eq!(result, expected);
}

#[test]
fn test_front_empty() {
    assert_eq!(Seq::<u32>::empty().front(), None);
}

#[test]
fn test_front_nonempty() {
    let seq: Seq<u32> = seq![0,1,2,3,4,5,6,7,8,9];
    assert_eq!(seq.front().map(|x| *x),Some(0));
}

#[test]
fn test_front_pop_front() {
    let mut seq: Seq<u32> = seq![0,1,2,3,4,5,6,7,8,9];
    let mut vec: Vec<u32> = vec![];
    loop {
        if let Some(front) = seq.front() {
            vec.push(*front);
        } else {
            break;
        }
        seq = seq.pop_front();
    }
    assert_eq!(vec,vec![0,1,2,3,4,5,6,7,8,9]);
}

#[test]
fn test_back_empty() {
    assert_eq!(Seq::<u32>::empty().back(), None);
}

#[test]
fn test_back_nonempty() {
    let seq: Seq<u32> = seq![0,1,2,3,4,5,6,7,8,9];
    assert_eq!(seq.back().map(|x| *x),Some(9));
}

#[test]
fn test_back_pop_back() {
    let mut seq: Seq<u32> = seq![0,1,2,3,4,5,6,7,8,9];
    let mut vec: Vec<u32> = vec![];
    loop {
        if let Some(back) = seq.back() {
            vec.push(*back);
        } else {
            break;
        }
        seq = seq.pop_back();
    }
    assert_eq!(vec,vec![9,8,7,6,5,4,3,2,1,0]);
}

#[test]
fn test_get_empty() {
    let seq: Seq<u32> = seq![];
    assert_eq!(seq.get(0), None);
    assert_eq!(seq.get(1), None);
}

#[test]
fn test_get_nonempty() {
    let seq: Seq<usize> = seq![0,1,2,3,4,5,6,7,8,9];
    for i in 0..seq.len() {
        assert_eq!(seq.get(i),Some(&i))
    }
}

#[test]
fn test_get_out_of_bounds() {
    let seq: Seq<u32> = seq![0,1,2,3,4,5,6,7,8,9];
    assert_eq!(seq.get(10),None)
}

#[test]
fn test_index() {
    let seq: Seq<usize> = seq![0,1,2,3,4,5,6,7,8,9];
    for i in 0..seq.len() {
        assert_eq!(seq[i],i)
    }
}

#[test]
#[should_panic]
fn test_index_out_of_bounds() {
    let seq: Seq<u32> = seq![0,1,2,3,4,5,6,7,8,9];
    seq[10];
}

#[test]
fn test_split_empty() {
    let seq: Seq<usize> = Seq::empty();
    assert!(seq.split(0).0.is_empty());
    assert!(seq.split(0).1.is_empty());
    assert!(seq.split(1).0.is_empty());
    assert!(seq.split(1).1.is_empty());
}

#[test]
fn test_split_singleton() {
    let seq: Seq<usize> = Seq::singleton(42);
    assert_eq!(seq.split(0).0.len(), 0);
    assert_eq!(seq.split(0).1.len(), 1);
    assert_eq!(seq.split(1).0.len(), 1);
    assert_eq!(seq.split(1).1.len(), 0);
}

#[test]
fn test_split() {
    let n = 10;
    let seq: Seq<usize> = (0..n).into_iter().collect();
    for i in 0..n {
        let (before, after) = seq.split(i);
        let before:Vec<usize> = before.iter().map(|x| *x).collect();
        let before_expected:Vec<usize> = (0..i).into_iter().collect();
        let after:Vec<usize> = after.iter().map(|x| *x).collect();
        let after_expected:Vec<usize> = (i..n).into_iter().collect();
        assert_eq!(before, before_expected);
        assert_eq!(after, after_expected);
    }
}

#[test]
fn test_truncate() {
    let n = 10;
    let seq: Seq<usize> = (0..n).into_iter().collect();
    for i in 0..n {
        let res:Vec<usize> = seq.truncate(i).iter().map(|x| *x).collect();
        let expected:Vec<usize> = (0..i).into_iter().collect();
        assert_eq!(res, expected);
    }
}

#[test]
fn test_skip() {
    let n = 10;
    let seq: Seq<usize> = (0..n).into_iter().collect();
    for i in 0..n {
        let res:Vec<usize> = seq.skip(i).iter().map(|x| *x).collect();
        let expected:Vec<usize> = (i..n).into_iter().collect();
        assert_eq!(res, expected);
    }
}

#[test]
fn test_remove() {
    let n = 10;
    let seq: Seq<usize> = (0..n).into_iter().collect();
    for i in 0..n {
        let res:Vec<usize> = seq.remove(i).iter().map(|x| *x).collect();
        let expected:Vec<usize> = (0..n).into_iter().filter(|&j| {j != i}).collect();
        assert_eq!(res, expected);
    }
}

#[test]
fn test_insert() {
    let n = 10;
    let seq: Seq<usize> = (0..n).into_iter().collect();
    for i in 0..n {
        let res:Vec<usize> = seq.insert(i,n).iter().map(|x| *x).collect();
        let mut expected:Vec<usize> = (0..n).into_iter().collect();
        expected.insert(i,n);
        assert_eq!(res, expected);
    }
}

#[test]
fn test_adjust() {
    let n = 10;
    let seq: Seq<usize> = (0..n).into_iter().collect();
    for i in 0..n {
        println!("{}",i);
        let res:Vec<usize> = seq.adjust(i,|x| x+1).iter().map(|x| *x).collect();
        let mut expected:Vec<usize> = (0..n).into_iter().collect();
        expected[i] += 1;
        assert_eq!(res, expected);
    }
}

#[test]
fn test_update() {
    let n = 10;
    let seq: Seq<usize> = (0..n).into_iter().collect();
    for i in 0..n {
        println!("{}",i);
        let res:Vec<usize> = seq.update(i,n).iter().map(|x| *x).collect();
        let mut expected:Vec<usize> = (0..n).into_iter().collect();
        expected[i] = n;
        assert_eq!(res, expected);
    }
}

