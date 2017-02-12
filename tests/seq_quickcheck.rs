extern crate immutable_seq;

#[macro_use]
extern crate quickcheck;

use std::fmt;

use quickcheck::{Gen, Arbitrary};

use immutable_seq::Seq;

#[derive(Clone,Copy,PartialEq,Eq,Hash,PartialOrd,Debug)]
struct Slot(usize);

impl Slot {
    fn unwrap(&self) -> usize {
        self.0
    }
}

#[derive(Clone,PartialEq,Eq,Hash,Debug)]
enum Action<T> {
    Empty,
    Singleton(T),
    PushFront(Slot, T),
    PushBack(Slot, T),
    Append(Slot, Slot),
    IsEmpty(Slot),
    Len(Slot),
    Front(Slot),
    Back(Slot),
    PopFront(Slot),
    PopBack(Slot),
    // Adjust(Slot, usize, Box(FnOnce(&T) -> T)),
    Update(Slot, usize, T),
    Truncate(Slot, usize),
    Skip(Slot, usize),
    Split(Slot, usize),
    Remove(Slot, usize),
    Insert(Slot, usize, T),
    Get(Slot, usize),
    Iter(Slot),
    FromVec(Vec<T>),
}

use Action::{
    Empty,
    Singleton,
    PushFront,
    PushBack,
    Append,
    IsEmpty,
    Len,
    Front,
    Back,
    PopFront,
    PopBack,
    Update,
    Truncate,
    Skip,
    Split,
    Remove,
    Insert,
    Get,
    Iter,
    FromVec,
};

impl<T> Action<T> {
    fn new_slots(&self) -> usize {
        match *self {
            Split(_,_)     => 2,

            Empty          => 1,
            Singleton(_)   => 1,
            PushFront(_,_) => 1,
            PushBack(_,_)  => 1,
            Append(_,_)    => 1,
            PopFront(_)    => 1,
            PopBack(_)     => 1,
            Update(_,_,_)  => 1,
            Truncate(_,_)  => 1,
            Skip(_,_)      => 1,
            Remove(_,_)    => 1,
            Insert(_,_,_)  => 1,
            FromVec(_)     => 1,

            IsEmpty(_)     => 0,
            Len(_)         => 0,
            Front(_)       => 0,
            Back(_)        => 0,
            Get(_,_)       => 0,
            Iter(_)        => 0,
        }
    }
    fn uses_slot(&self,slot: Slot) -> bool {
        match *self {
            Append(s1, s2)  => s1 == slot || s2 == slot,
            Split(s, _)     => s  == slot,
            PushFront(s, _) => s  == slot,
            PushBack(s, _)  => s  == slot,
            Update(s, _, _) => s  == slot,
            Truncate(s, _)  => s  == slot,
            Skip(s, _)      => s  == slot,
            Remove(s, _)    => s  == slot,
            Insert(s, _, _) => s  == slot,
            Get(s, _)       => s  == slot,
            IsEmpty(s)      => s  == slot,
            Len(s)          => s  == slot,
            Front(s)        => s  == slot,
            Back(s)         => s  == slot,
            PopFront(s)     => s  == slot,
            PopBack(s)      => s  == slot,
            Iter(s)         => s  == slot,
            _               => false,
        }
    }
    fn drop_slot(&mut self, slot: Slot) {
        macro_rules! decr {
            ($s: expr) => {
                if * $s > slot {
                    * $s = Slot($s.unwrap() - 1)
                }
            }
        };
        match *self {
            Split     (ref mut s, _)            => decr!(s),
            PushFront (ref mut s, _)            => decr!(s),
            PushBack  (ref mut s, _)            => decr!(s),
            Update    (ref mut s, _, _)         => decr!(s),
            Truncate  (ref mut s, _)            => decr!(s),
            Skip      (ref mut s, _)            => decr!(s),
            Remove    (ref mut s, _)            => decr!(s),
            Insert    (ref mut s, _, _)         => decr!(s),
            Get       (ref mut s, _)            => decr!(s),
            IsEmpty   (ref mut s)               => decr!(s),
            Len       (ref mut s)               => decr!(s),
            Front     (ref mut s)               => decr!(s),
            Back      (ref mut s)               => decr!(s),
            PopFront  (ref mut s)               => decr!(s),
            PopBack   (ref mut s)               => decr!(s),
            Iter      (ref mut s)               => decr!(s),
            Append    (ref mut s1,  ref mut s2) => {decr!(s1); decr!(s2)},
            _                                   => {},
        }
    }
}

impl<T: Arbitrary> Action<T> {
    fn shrink(&self) -> Box<Iterator<Item=Action<T>>> {
        match *self {
            Split(s, n) =>
                Box::new(n.shrink().map(move |i| Split(s,i))),
            Singleton(ref x) =>
                Box::new(x.shrink().map(move |y| Singleton(y))),
            PushFront(s, ref x) =>
                Box::new(x.shrink().map(move |y| PushFront(s, y))),
            PushBack(s, ref x) =>
                Box::new(x.shrink().map(move |y| PushBack(s, y))),
            Update(s, n, ref x) =>
                Box::new((n,x.clone()).shrink()
                         .map(move |(i, y)| Update(s,i,y))),
            Truncate(s, n) =>
                Box::new(n.shrink().map(move |i| Truncate(s,i))),
            Skip(s, n) =>
                Box::new(n.shrink().map(move |i| Skip(s,i))),
            Remove(s, n) =>
                Box::new(n.shrink().map(move |i| Remove(s,i))),
            Insert(s, n, ref x) =>
                Box::new((n,x.clone()).shrink()
                         .map(move |(i, y)| Insert(s,i,y))),
            FromVec(ref v) =>
                Box::new(v.shrink().map(move |u| FromVec(u))),
            Get(s, n) =>
                Box::new(n.shrink().map(move |i| Get(s, i))),
            _ => quickcheck::empty_shrinker(),
        }
    }
}


#[derive(Clone,Debug)]
struct ActionSeries<T> {
    actions: Vec<Action<T>>,
    // num_slots: u32,
    slots_after: Vec<usize>,
}

impl<T> ActionSeries<T> {
    fn empty() -> ActionSeries<T> {
        ActionSeries::new(vec![])
    }
    fn new(actions: Vec<Action<T>>) -> ActionSeries<T> {
        // let mut slots_after = vec![];
        // let mut num_slots = 0;
        // for a in actions {
        //     num_slots += a.new_slots();
        //     slots_after.push(num_slots)
        // }
        let mut num_slots = 0;
        let slots_after = actions.iter().map(|a| {
            num_slots += a.new_slots();
            num_slots
        }).collect();
        ActionSeries {
            actions: actions,
            // num_slots: 0,
            slots_after: slots_after
        }
    }
}

// macro_rules! alternatives {
//     ($gen: expr ; $($e: expr),*) => {
//         alternatives!($gen, 0 ; $($e),*)
//     };
//     ($gen: expr, $n: expr $(, ($i: expr, $b: expr))* ; ) => {
//         match $gen.next_u32() % $n {
//             $($i => $b , )*
//             _ => unreachable!()
//         }
//     };
//     ($gen: expr, $n: expr $(, ($i: expr, $b: expr))* ; $e0: expr $(, $e: expr)*) => {
//         alternatives!($gen, $n+1 $(, ($i , $b))* , ($n , $e0) ; $($e),* )
//     }
// }

impl<T> ActionSeries<T>
    where T: Arbitrary + fmt::Debug
{
    fn num_slots(&self) -> usize {
        self.slots_after.last().map(|x| *x).unwrap_or(0usize)
    }
    fn arbitrary_slot<G: Gen>(&self, g: &mut G) -> Slot {
        if self.num_slots() == 0 {
            panic!("arbitrary_slot called with zero slots");
        } else {
            Slot(g.next_u32() as usize % self.num_slots())
        }
    }
    fn arbitrary_action<G: Gen>(&self, g: &mut G) -> Action<T> {
        if self.num_slots() == 0 {
            match g.next_u32() % 3 {
                0 => Action::Empty,
                1 => Action::Singleton      (T::arbitrary        (g)),
                2 => Action::FromVec        (Vec::<T>::arbitrary (g)),
                _ => unreachable!(),
            }
        } else {
            match g.next_u32() % 20 {
                0  => Action::Empty,
                1  => Action::Singleton (T::arbitrary        (g)),
                2  => Action::PushFront (self.arbitrary_slot (g), T::arbitrary        (g)),
                3  => Action::PushBack  (self.arbitrary_slot (g), T::arbitrary        (g)),
                4  => Action::Append    (self.arbitrary_slot (g), self.arbitrary_slot (g)),
                5  => Action::IsEmpty   (self.arbitrary_slot (g)),
                6  => Action::Len       (self.arbitrary_slot (g)),
                7  => Action::Front     (self.arbitrary_slot (g)),
                8  => Action::Back      (self.arbitrary_slot (g)),
                9  => Action::PopFront  (self.arbitrary_slot (g)),
                10 => Action::PopBack   (self.arbitrary_slot (g)),
                11 => Action::Update    (self.arbitrary_slot (g), usize::arbitrary    (g), T::arbitrary (g)),
                12 => Action::Truncate  (self.arbitrary_slot (g), usize::arbitrary    (g)),
                13 => Action::Skip      (self.arbitrary_slot (g), usize::arbitrary    (g)),
                14 => Action::Split     (self.arbitrary_slot (g), usize::arbitrary    (g)),
                15 => Action::Remove    (self.arbitrary_slot (g), usize::arbitrary    (g)),
                16 => Action::Insert    (self.arbitrary_slot (g), usize::arbitrary    (g), T::arbitrary (g)),
                17 => Action::Get       (self.arbitrary_slot (g), usize::arbitrary    (g)),
                18 => Action::Iter      (self.arbitrary_slot (g)),
                19 => Action::FromVec   (Vec::<T>::arbitrary (g)),
                _ => unreachable!(),
            }
        }
    }
    fn extend_arbitrarily<G: Gen>(&mut self, g: &mut G) {
        let action = self.arbitrary_action(g);
        self.actions.push(action.clone());
        let n = self.num_slots() + action.new_slots();
        self.slots_after.push(n);
    }
    fn drop_slot(&mut self, from: usize, slot: Slot) {
        let mut action_index = from;
        while action_index < self.actions.len() {
            let uses_slot = self.actions[action_index].uses_slot(slot);
            if uses_slot {
                self.drop_action(action_index);
            } else {
                action_index += 1;
            }
        }
        action_index = from;
        while action_index < self.actions.len() {
            self.actions[action_index].drop_slot(slot);
            if self.slots_after[action_index] > slot.unwrap() {
                self.slots_after[action_index] -= 1;
            }
            action_index += 1;
        }
    }
    fn drop_action(&mut self, action_index: usize) {
        let slots_before = if action_index > 0 {
            self.slots_after[action_index-1]
        } else {
            0
        };
        let slots_after = self.slots_after[action_index];
        let mut slot_index = slots_after;
        while slot_index > slots_before {
            slot_index -= 1;
            self.drop_slot(action_index+1, Slot(slot_index));
        }
        self.actions.remove(action_index);
        self.slots_after.remove(action_index);
    }
}

struct Shrinker<T> {
    series: ActionSeries<T>,
    action_index: usize,
    action_shrinker: Box<Iterator<Item=Action<T>>>,
}

impl<T:'static> Shrinker<T> {
    fn new(series: ActionSeries<T>) -> Box<Shrinker<T>> {
        Box::new(Shrinker {
            series: series,
            action_index: 0,
            action_shrinker: quickcheck::empty_shrinker(),
        })
    }
}

impl<T> Iterator for Shrinker<T>
    where T: Arbitrary + fmt::Debug
{
    type Item=ActionSeries<T>;
    fn next(&mut self) -> Option<ActionSeries<T>> {
        if let Some(a) = self.action_shrinker.next() {
            let mut s = self.series.clone();
            s.actions[self.action_index] = a;
            return Some(s)
        }
        let n = self.series.actions.len();
        let i = &mut self.action_index;
        *i += 1;
        if *i < n {
            self.action_shrinker = self.series.actions[*i].shrink();
            let mut s = self.series.clone();
            s.drop_action(*i);
            Some(s)
        } else {
            None
        }
    }
}

impl<T> Arbitrary for ActionSeries<T>
    where T: Arbitrary + fmt::Debug
{
    fn arbitrary<G: Gen>(g: &mut G) -> ActionSeries<T> {
        let mut series = ActionSeries::<T>::empty();
        let size = { let s = g.size(); g.gen_range(0, s) };
        for _ in 0..size {
            series.extend_arbitrarily(g);
        }
        series
    }
    fn shrink(&self) -> Box<Iterator<Item=ActionSeries<T>>> {
        // quickcheck::empty_shrinker()
        Shrinker::new(self.clone())
    }
}

struct Model<T> {
    reference: Vec<Vec<T>>,
    subject: Vec<Seq<T>>,
}

impl<T> Model<T>
    where T: Clone + PartialEq + fmt::Debug + 'static
{
    fn new() -> Model<T> {
        Model {
            reference: vec![],
            subject: vec![],
        }
    }
    fn push(&mut self, vec: Vec<T>, seq:Seq<T>) {
        self.reference.push(vec);
        self.subject.push(seq);
    }
    fn get(&mut self, slot: Slot) -> (Vec<T>, Seq<T>) {
        let slot = slot.unwrap();
        (self.reference[slot].clone(), self.subject[slot].clone())
    }
    fn apply_action(&mut self, action: &Action<T>) -> Result<(), String> {
        macro_rules! check {
            ($e: expr, $msg: expr) => {
                if ! $e {
                    return Err($msg);
                }
            }
        }
        macro_rules! check_eq {
            ($e1: expr, $e2: expr, $msg: expr) => {
                check!($e1 == $e2, $msg)
            };
            ($e1: expr, $e2: expr) => {{
                    assert_eq!($e1, $e2);
                    check!($e1 == $e2, "fail".into())
            }}
        }
        match *action {
            Empty =>
                self.push(vec![], Seq::empty()),
            Singleton(ref x) =>
                self.push(vec![x.clone()], Seq::singleton(x.clone())),
            PushFront(slot, ref x) => {
                let (mut vec,seq) = self.get(slot);
                vec.insert(0, x.clone());
                self.push(vec, seq.push_front(x.clone()));
            },
            PushBack(slot, ref x) => {
                let (mut vec, seq) = self.get(slot);
                vec.push(x.clone());
                self.push(vec, seq.push_back(x.clone()));
            },
            Append(slot1, slot2) => {
                let (mut vec1, seq1) = self.get(slot1);
                let (mut vec2, seq2) = self.get(slot2);
                vec1.append(&mut vec2);
                self.push(vec1, seq1.append(&seq2));
            },
            IsEmpty(slot) => {
                let (vec, seq) = self.get(slot);
                check_eq!(vec.is_empty(), seq.is_empty())
            },
            Len(slot) => {
                let (vec, seq) = self.get(slot);
                check_eq!(vec.len(), seq.len())
            },
            Front(slot) => {
                let (vec, seq) = self.get(slot);
                check_eq!(vec.first(), seq.front())
            },
            Back(slot) => {
                let (vec, seq) = self.get(slot);
                check_eq!(vec.last(), seq.back())
            },
            PopFront(slot) => {
                let (mut vec, seq) = self.get(slot);
                if vec.len() > 0 {
                    vec.remove(0);
                }
                self.push(vec, seq.pop_front());
            },
            PopBack(slot) => {
                let (mut vec, seq) = self.get(slot);
                vec.pop();
                self.push(vec, seq.pop_back());
            },
            Update(slot, index, ref x) => {
                let (mut vec, seq) = self.get(slot);
                if let Some(v) = vec.get_mut(index) {
                    *v = x.clone();
                }
                self.push(vec, seq.update(index, x.clone()));
            },
            Truncate(slot, length) => {
                let (mut vec, seq) = self.get(slot);
                vec.truncate(length);
                self.push(vec, seq.truncate(length));
            },
            Skip(slot, length) => {
                let (vec, seq) = self.get(slot);
                let vec = vec.into_iter().skip(length).collect();
                self.push(vec, seq.skip(length));
            },
            Split(slot, index) => {
                let (mut vec1, seq) = self.get(slot);
                let vec2 = if index <= vec1.len() {
                    vec1.split_off(index)
                } else {
                    vec![]
                };
                let (seq1, seq2) = seq.split(index);
                self.push(vec1, seq1);
                self.push(vec2, seq2);
            },
            Remove(slot, index) => {
                let (mut vec, seq) = self.get(slot);
                if index < vec.len() {
                    vec.remove(index);
                }
                self.push(vec, seq.remove(index));
            },
            Insert(slot, index, ref x) => {
                let (mut vec, seq) = self.get(slot);
                if index >= vec.len() {
                    vec.push(x.clone())
                } else {
                    vec.insert(index, x.clone());
                }
                self.push(vec, seq.insert(index, x.clone()));
            },
            Get(slot, index) => {
                let (vec, seq) = self.get(slot);
                check_eq!(vec.get(index), seq.get(index));
            },
            Iter(slot) => {
                let (vec, seq) = self.get(slot);
                check_eq!(vec, seq.iter().map(|x| x.clone()).collect::<Vec<T>>());
            },
            FromVec(ref vec) => {
                self.push(vec.clone(), vec.into_iter().map(|x| x.clone()).collect())
            },
        };
        return Ok(())
    }
    fn apply_series(&mut self, series: ActionSeries<T>) -> Result<(),String> {
        for ref action in series.actions {
            try!(self.apply_action(action))
        }
        return Ok(())
    }
}

#[test]
fn test_action_series_drop_slot() {
    let mut series = ActionSeries::<u32>::new(vec![Empty, Insert(Slot(0), 22, 59), Append(Slot(0),Slot(1)), Split(Slot(1),39), Back(Slot(3))]);
    series.drop_action(1);
    assert_eq!(series.actions, vec![Empty]);
    assert_eq!(series.slots_after, vec![1]);
}

#[test]
fn test_check_temp() {
    let series = ActionSeries::<u32>::new(
        // vec![Singleton(1), Empty, PushBack(Slot(1), 0), Insert(Slot(2), 0, 0), Split(Slot(3), 0), Back(Slot(3))]
        vec![Singleton(48), Singleton(0), PopBack(Slot(1)), Insert(Slot(2), 1, 0), Back(Slot(3))]
    );
    assert_eq!(Model::new().apply_series(series), Ok(()));
}

quickcheck! {
    fn check_model_u32(series: ActionSeries<u32>) -> Result<(),String> {
        Model::new().apply_series(series)
    }
}
