// This code is based on code from Jonathan Reem's rust-lazy library (https://github.com/reem/rust-lazy)

use std::ops::Deref;
use std::cell::UnsafeCell;
use std::rc::Rc;
use std::mem;
use std::fmt;

use self::Inner::{Evaluated, EvaluationInProgress, Unevaluated, Redirect};

/// Helper macro for writing lazy expressions
///
/// ```rust,ignore
/// #[macro_use] extern crate immutable_seq;
/// # use immutable_seq::lazy::value;
/// # fn main() {
/// let thunk = lazy!{
///     println!("Evaluated!");
///     value(7u32)
/// };
/// assert_eq!(*thunk, 7u32);
/// # }
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! lazy {
    ($($e: stmt);*) => {
        $crate::lazy::Thunk::new(move || { $($e);* })
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! lazy_val {
    ($($e: stmt);*) => {
        $crate::lazy::Thunk::new(move || { value({$($e);*}) })
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! lazy_redirect {
    ($($e: stmt);*) => {
        $crate::lazy::Thunk::new(move || { redirect({$($e);*}) })
    }
}

pub fn strict<T>(v: T) -> Thunk<T> {
    Thunk::evaluated(v)
}

pub fn redirect<T>(t: Thunk<T>) -> ThunkResult<T> {
    ThunkResult::Redirect(t)
}

pub fn value<T>(v: T) -> ThunkResult<T> {
    ThunkResult::Value(v)
}

/// A sometimes-cleaner name for a lazily evaluated value.
pub type Lazy<T> = Thunk<T>;

/// A lazily evaluated value.
pub struct Thunk<T> (UnsafeCell<Rc<UnsafeCell<Inner<T>>>>);

impl<T> Thunk<T> {
    /// Create a lazily evaluated value from a proc that returns that value.
    ///
    /// You can construct Thunk's manually using this, but the lazy! macro
    /// is preferred.
    ///
    /// ```rust,ignore
    /// # use immutable_seq::lazy::{Thunk, value};
    /// let expensive = Thunk::new(|| { println!("Evaluated!"); value(7u32) });
    /// assert_eq!(*expensive, 7u32); // "Evaluated!" gets printed here.
    /// assert_eq!(*expensive, 7u32); // Nothing printed.
    /// ```
    pub fn new<F>(producer: F) -> Thunk<T>
    where F: FnOnce() -> ThunkResult<T> + 'static {
        Thunk(UnsafeCell::new(Rc::new(UnsafeCell::new(Unevaluated(Producer::new(producer))))))
    }

    /// Create a new, evaluated, thunk from a value.
    pub fn evaluated(val: T) -> Thunk<T> {
        Thunk(UnsafeCell::new(Rc::new(UnsafeCell::new(Evaluated(val)))))
    }

    /// Force evaluation of a thunk.
    pub fn force(&self) {
        loop {
            match *self.inner() {
                Evaluated(_) => return,
                EvaluationInProgress => {
                    panic!("Thunk::force called recursively. (A Thunk tried to force itself while trying to force itself).")
                },
                Redirect(ref t) => {
                    self.redirect(t.clone());
                    continue;
                },
                Unevaluated(_) => ()
            };
            break;
        }

        match mem::replace(self.inner(), EvaluationInProgress) {
            Unevaluated(producer) => {
                *self.inner() = EvaluationInProgress;
                match producer.invoke() {
                    ThunkResult::Value(x) =>
                        *self.inner() = Evaluated(x),
                    ThunkResult::Redirect(t) => {
                        t.force();
                        *self.inner() = Redirect(t.clone());
                        self.redirect(t);
                    }
                }
            }
            _ => {
                let x = 42;
                println!("thats not good {}",x);
                // unsafe { debug_unreachable!() }   
            }
        }
    }

    fn inner(&self) -> &mut Inner<T> {
        match *self {
            Thunk(ref cell) => unsafe {
                &mut *(**cell.get()).get()
            }
        }
    }

    fn rc(&self) -> &mut Rc<UnsafeCell<Inner<T>>> {
        match *self {
            Thunk(ref cell) => unsafe {
                &mut *cell.get()
            }
        }
    }

    fn redirect(&self, t: Thunk<T>) {
        *self.rc() = t.rc().clone();
    }
}

impl<T> Deref for Thunk<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.force();
        match *self.inner()  {
            Evaluated(ref val) => val,
            _ => unreachable!(),
        }
    }
}

impl<T> Clone for Thunk<T> {
    fn clone(&self) -> Thunk<T> {
        Thunk(UnsafeCell::new(self.rc().clone()))
    }
}

impl<T> fmt::Debug for Thunk<T>
    where T: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Thunk")
            .field(self.inner())
            .finish()
    }
}

/// Represents the two possible things a `Thunk<T>` can return: either a `T` value, or another `Thunk<T>`.
#[derive(Debug)]
pub enum ThunkResult<T> {
    Value(T),
    Redirect(Thunk<T>)
}

struct Producer<T> {
    inner: Box<Invoke<T>>
}

impl<T> fmt::Debug for Producer<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Producer{{...}}")
    }
}

impl<T> Producer<T> {
    fn new<F: FnOnce() -> T + 'static>(f: F) -> Producer<T> {
        Producer {
            inner: Box::new(move || {
                f()
            }) as Box<Invoke<T>>
        }
    }

    fn invoke(self) -> T {
        self.inner.invoke()
    }
}

#[derive(Debug)]
enum Inner<T> {
    Evaluated(T),
    EvaluationInProgress,
    Unevaluated(Producer<ThunkResult<T>>),
    Redirect(Thunk<T>),
}

#[doc(hidden)]
pub trait Invoke<T> {
    fn invoke(self: Box<Self>) -> T;
}

impl<T, F> Invoke<T> for F
    where F: FnOnce() -> T
{
    fn invoke(self: Box<F>) -> T {
        let f = *self;
        f()
    }
}

#[cfg(test)]
mod test {
    use super::{Thunk, value, redirect, strict};
    use std::sync::{Arc, Mutex};
    use std::thread;

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

    struct Dropper(Arc<Mutex<u64>>);

    impl Drop for Dropper {
        fn drop(&mut self) {
            let Dropper(ref count) = *self;
            *count.lock().unwrap() += 1;
        }
    }
}
