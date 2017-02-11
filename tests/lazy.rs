
#[macro_use]
extern crate immutable_seq;

pub use immutable_seq::lazy::{Thunk, value, redirect, strict};
pub use std::sync::{Arc, Mutex};
pub use std::thread;

#[test]
fn test_thunk_should_evaluate_when_accessed() {
    let val = lazy!(value(7));
    assert_eq!(*val, 7);
}

#[test]
fn test_thunk_should_evaluate_through_redirect() {
    let val = lazy!(redirect(lazy!(value(7))));
    assert_eq!(*val, 7);
}

#[test]
fn test_thunk_should_evaluate_just_once() {
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();
    let val = lazy!({
        let mut data = counter.lock().unwrap();
        *data += 1;
        value(())
    });
    *val;
    *val;
    assert_eq!(*counter_clone.lock().unwrap(), 1);
}

#[test]
fn test_thunk_should_not_evaluate_if_not_accessed() {
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();
    let _val = lazy!({
        let mut data = counter.lock().unwrap();
        *data += 1;
        value(())
    });
    assert_eq!(*counter_clone.lock().unwrap(), 0);
}

#[test]
fn test_strict_should_produce_already_evaluated_thunk() {
    let x = strict(10);
    assert_eq!(*x, 10);
}

#[test]
fn test_drop_internal_data_just_once() {
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();
    let result = thread::spawn(move || {
        let value = Dropper(counter_clone);
        let t = Thunk::<()>::new(move || {
            // Get a reference so value is captured.
            let _x = &value;

            panic!("Muahahahah")
        });
        t.force();
    }).join();

    match result {
        Err(_) => {
            assert_eq!(*counter.lock().unwrap(), 1);
        },
        _ => panic!("Unexpected success in spawned task.")
    }
}

pub struct Dropper(Arc<Mutex<u64>>);

impl Drop for Dropper {
    fn drop(&mut self) {
        let Dropper(ref count) = *self;
        *count.lock().unwrap() += 1;
    }
}
