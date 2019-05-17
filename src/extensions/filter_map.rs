use crate::observable::Observable;
use crate::observer::Observer;
use crate::{Subscription, BaseObserver};

pub struct FilterMapObservable<M, O> {
    map: M,
    original: O,
}

unsafe impl<M, O> Send for FilterMapObservable<M, O> {}
unsafe impl<M, O> Sync for FilterMapObservable<M, O> {}

pub trait FilterMapExt<'a>: Observable<'a> + Sized {
    fn filter_map<M, I>(self, map: M) -> FilterMapObservable<M, Self> where M: Fn(Self::Item) -> Option<I> + 'a {
        FilterMapObservable { map, original: self }
    }
}

impl<'a, O> FilterMapExt<'a> for O where O: Observable<'a> {}

impl<'a, I, M, O> Observable<'a> for FilterMapObservable<M, O> where O: Observable<'a> + 'a, M: Fn(O::Item) -> Option<I> + Clone + 'a, I: 'a {
    type Item = I;
    type Error = O::Error;

    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + 'a) -> Subscription<'a> {
        let map = self.map;
        let observer = BaseObserver::new(observer);
        let next = {
            let observer = observer.clone();
            move |item| {
                if let Some(item) = map(item) {
                    observer.on_next(item)
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
    use std::sync::{Arc, Mutex};
    use crate::prelude::*;
    use crate::Subject;

    #[test]
    fn it_works() {
        let obs = Subject::<i32, ()>::new();
        let data = Arc::new(Mutex::new(Vec::new()));
        {
            let data = data.clone();
            obs
                .fork()
                .filter_map(|x| {
                    if x > 1 {
                        Some(x * 2)
                    } else {
                        None
                    }
                })
                .subscribe(move |x| { data.lock().unwrap().push(x) });
        }
        obs.on_next(1);
        obs.on_next(1);
        obs.on_next(2);
        obs.on_next(3);

        assert_eq!(&vec![4, 6], &*data.lock().unwrap());
    }
}

