use crate::observable::Observable;
use crate::observer::Observer;
use std::sync::{Arc, Mutex};
use crate::{Subscription, BaseObserver};

pub struct FoldObservable<A, F, O> {
    fold: F,
    original: O,
    init: Arc<Mutex<A>>,
}

unsafe impl<A, F, O> Send for FoldObservable<A, F, O> {}
unsafe impl<A, F, O> Sync for FoldObservable<A, F, O> {}

pub trait FoldExt<'a>: Observable<'a> + Sized {
    fn fold<A, F>(self, init: A, fold: F) -> FoldObservable<A, F, Self> where F: Fn(A, Self::Item) -> A + 'a, Self: 'a {
        FoldObservable { fold, original: self, init: Arc::new(Mutex::new(init)) }
    }
}

impl<'a, O> FoldExt<'a> for O where O: Observable<'a> {}

impl<'a, A, F, O> Observable<'a> for FoldObservable<A, F, O> where F: Fn(A, O::Item) -> A + Clone + 'a, A: 'a, O: Observable<'a> + 'a {
    type Item = A;
    type Error = O::Error;

    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + 'a) -> Subscription<'a> {
        let fold = self.fold;
        let init = self.init;
        let observer = BaseObserver::new(observer);
        let next = {
            let obs = observer.clone();
            move |item| {
                let mut init = init.lock().unwrap();
                let acc = fold(unsafe { std::mem::transmute_copy(&*init) }, item);
                *init = unsafe { std::mem::transmute_copy(&acc) };
                obs.on_next(acc);
            }
        };
        let complete = {
            let obs = observer.clone();
            move || obs.on_completed()
        };
        let error = {
            let obs = observer.clone();
            move |error| obs.on_error(error)
        };
        self.original.subscribe((next, error, complete));
        Subscription::new(|| observer.dispose())
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::BaseObservable;
    use std::rc::Rc;
    use std::cell::RefCell;

    #[test]
    fn it_works() {
        let obs = BaseObservable::<i32, ()>::new(|sub| {
            sub.on_next(1);
            sub.on_next(2);
            sub.on_next(3);
        });
        let data = Rc::new(RefCell::new(Vec::new()));
        {
            let data = data.clone();
            obs
                .fold(0, |sum, x| sum + x)
                .subscribe(move |x| {
                    data.borrow_mut().push(x);
                });
        }
        assert_eq!(&vec![1, 3, 6], &*data.borrow_mut());
    }
}
