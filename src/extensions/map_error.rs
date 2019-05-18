use crate::observable::Observable;
use crate::observer::Observer;
use crate::{Subscription, BaseObserver};

pub struct MapErrorObservable<M, O> {
    map: M,
    original: O,
}

unsafe impl<M, O> Send for MapErrorObservable<M, O> {}
unsafe impl<M, O> Sync for MapErrorObservable<M, O> {}

pub trait MapErrorExt: Observable + Sized {
    fn map_err<M, E>(self, map: M) -> MapErrorObservable<M, Self> where M: Fn(Self::Error) -> E + 'static {
        MapErrorObservable { map, original: self }
    }
}

impl<O> MapErrorExt for O where O: Observable {}

impl<E, M, O> Observable for MapErrorObservable<M, O> where O: Observable, M: Fn(O::Error) -> E + Clone + 'static, O::Item: 'static, E: 'static {
    type Item = O::Item;
    type Error = E;

    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + 'static) -> Subscription {
        let map = self.map;
        let observer = BaseObserver::new(observer);
        let next = {
            let observer = observer.clone();
            move |item| observer.on_next(item)
        };
        let complete = {
            let observer = observer.clone();
            move || observer.on_completed()
        };
        let error = move |error| observer.on_error(map(error));
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
            let data_error = data.clone();
            obs
                .fork()
                .map_err(|_| "error")
                .subscribe((
                    move |x| { data.lock().unwrap().push(x) },
                    move |e| {
                        assert_eq!("error", e);
                        data_error.lock().unwrap().push(10);
                    }
                ));
        }
        obs.on_next(1);
        obs.on_next(2);
        obs.on_next(3);
        obs.on_error(());

        assert_eq!(&vec![1, 2, 3, 10], &*data.lock().unwrap());
    }
}
