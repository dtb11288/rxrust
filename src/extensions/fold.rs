use crate::observable::Observable;
use crate::observer::Observer;
use crate::{Subscription, BaseObserver};
use std::rc::Rc;
use std::cell::RefCell;

pub struct FoldObservable<A, F, O> {
    fold: F,
    original: O,
    init: Rc<RefCell<A>>,
}

unsafe impl<A, F, O> Send for FoldObservable<A, F, O> {}
unsafe impl<A, F, O> Sync for FoldObservable<A, F, O> {}

pub trait FoldExt: Observable + Sized {
    fn fold<A, F>(self, init: A, fold: F) -> FoldObservable<A, F, Self> where F: Fn(A, Self::Item) -> A + 'static {
        FoldObservable { fold, original: self, init: Rc::new(RefCell::new(init)) }
    }
}

impl<O> FoldExt for O where O: Observable {}

impl<A, F, O> Observable for FoldObservable<A, F, O> where F: Fn(A, O::Item) -> A + Clone + 'static, O: Observable, O::Error: 'static, A: 'static {
    type Item = A;
    type Error = O::Error;

    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + 'static) -> Subscription {
        let fold = self.fold;
        let init = self.init;
        let observer = BaseObserver::new(observer);
        let next = {
            let obs = observer.clone();
            move |item| {
                let acc = fold(unsafe { std::mem::transmute_copy(&*init.borrow()) }, item);
                init.replace(unsafe { std::mem::transmute_copy(&acc) });
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
        let sub = self.original.subscribe((next, error, complete));
        Subscription::new(|| sub.unsubscribe())
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
