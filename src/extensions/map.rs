use crate::observable::Observable;
use crate::observer::Observer;
use crate::{Subscription, BaseObserver};

pub struct MapObservable<M, O> {
    map: M,
    original: O,
}

unsafe impl<M, O> Send for MapObservable<M, O> {}
unsafe impl<M, O> Sync for MapObservable<M, O> {}

pub trait MapExt<'a>: Observable<'a> + Sized {
    fn map<M, I>(self, map: M) -> MapObservable<M, Self> where M: Fn(Self::Item) -> I + 'a {
        MapObservable { map, original: self }
    }
}

impl<'a, O> MapExt<'a> for O where O: Observable<'a> {}

impl<'a, I, M, O> Observable<'a> for MapObservable<M, O> where O: Observable<'a> + 'a, M: Fn(O::Item) -> I + Clone + 'a, I: 'a {
    type Item = I;
    type Error = O::Error;

    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + 'a) -> Subscription<'a> {
        let map = self.map;
        let observer = BaseObserver::new(observer);
        let next = {
            let observer = observer.clone();
            move |item| observer.on_next(map(item))
        };
        let complete = {
            let observer = observer.clone();
            move || observer.on_completed()
        };
        let error = {
            let observer = observer.clone();
            move |error| observer.on_error(error)
        };
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
                .map(|x| x * 2)
                .subscribe(move |x| {
                    data.lock().unwrap().push(x);
                });
        }
        obs.on_next(1);
        obs.on_next(2);
        obs.on_next(3);

        assert_eq!(&vec![2, 4, 6], &*data.lock().unwrap());
    }
}
