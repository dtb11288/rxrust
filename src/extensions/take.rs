use crate::observable::Observable;
use crate::observer::Observer;
use crate::{Subscription, BaseObserver};
use std::sync::{Arc, Mutex};

pub struct TakeObservable<O> {
    original: O,
    count: Arc<Mutex<u64>>,
}

pub trait TakeExt<'a>: Observable<'a> + Sized {
    fn take(self, count: u64) -> TakeObservable<Self> {
        TakeObservable { original: self, count: Arc::new(Mutex::new(count)) }
    }
}

impl<'a, O> TakeExt<'a> for O where O: Observable<'a> {}

impl<'a, O> Observable<'a> for TakeObservable<O> where O: Observable<'a> + 'a {
    type Item = O::Item;
    type Error = O::Error;

    fn subscribe(self, obs: impl Observer<Self::Item, Self::Error> + 'a) -> Subscription<'a> {
        let observer = BaseObserver::new(obs);
        let count = self.count;
        let next = {
            let observer = observer.clone();
            move |item| {
                let mut count = count.lock().unwrap();
                if *count == 0 {
                    let observer = observer.clone();
                    observer.on_completed()
                } else {
                    *count -= 1;
                    observer.on_next(item);
                }
            }
        };
        let complete = {
            let observer = observer.clone();
            move || observer.on_completed()
        };
        let error = move |error| observer.on_error(error);
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
                .take(2)
                .subscribe(move |x| {
                    data.borrow_mut().push(x);
                });
        }
        assert_eq!(&*data.borrow_mut(), &vec![1, 2]);
    }
}
