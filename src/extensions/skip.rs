use crate::observable::Observable;
use crate::observer::Observer;
use crate::{Subscription, BaseObserver};
use std::sync::{Arc, Mutex};

pub struct SkipObservable<O> {
    original: O,
    count: Arc<Mutex<u64>>,
}

pub trait SkipExt<'a>: Observable<'a> + Sized {
    fn skip(self, count: u64) -> SkipObservable<Self> {
        SkipObservable { original: self, count: Arc::new(Mutex::new(count)) }
    }
}

impl<'a, O> SkipExt<'a> for O where O: Observable<'a> {}

impl<'a, O> Observable<'a> for SkipObservable<O> where O: Observable<'a> + 'a {
    type Item = O::Item;
    type Error = O::Error;

    fn subscribe(self, obs: impl Observer<Self::Item, Self::Error> + Send + Sync + 'a) -> Subscription<'a> {
        let observer = BaseObserver::new(obs);
        let count = self.count;
        let next = {
            let observer = observer.clone();
            move |item| {
                let mut count = count.lock().unwrap();
                if *count == 0 {
                    observer.on_next(item);
                } else {
                    *count -= 1;
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
    use std::sync::{Mutex, Arc};

    #[test]
    fn it_works() {
        let obs = BaseObservable::<i32, ()>::new(|sub| {
            sub.on_next(1);
            sub.on_next(2);
            sub.on_next(3);
            sub.on_next(4);
            sub.on_completed();
        });
        let data = Arc::new(Mutex::new(Vec::new()));
        {
            let data = data.clone();
            obs
                .skip(2)
                .subscribe(move |x| {
                    data.lock().unwrap().push(x);
                });
        }
        assert_eq!(&*data.lock().unwrap(), &vec![3, 4]);
    }
}
