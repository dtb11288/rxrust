use crate::observable::Observable;
use crate::observer::Observer;
use crate::{Subscription, BaseObserver};

pub struct FilterObservable<F, O> {
    filter: F,
    original: O,
}

unsafe impl<F, O> Send for FilterObservable<F, O> {}
unsafe impl<F, O> Sync for FilterObservable<F, O> {}

pub trait FilterExt<'a>: Observable<'a> + Sized {
    fn filter<F>(self, map: F) -> FilterObservable<F, Self> where F: Fn(&Self::Item) -> bool + 'a {
        FilterObservable { filter: map, original: self }
    }
}

impl<'a, O> FilterExt<'a> for O where O: Observable<'a> {}

impl<'a, F, O> Observable<'a> for FilterObservable<F, O> where O: Observable<'a> + 'a, F: Fn(&O::Item) -> bool + Clone + 'a, O::Item: 'a {
    type Item = O::Item;
    type Error = O::Error;

    fn subscribe(self, obs: impl Observer<Self::Item, Self::Error> + 'a) -> Subscription<'a> {
        let filter = self.filter;
        let observer = BaseObserver::new(obs);
        let next = {
            let observer = observer.clone();
            move |item| { if filter(&item) { observer.on_next(item) } }
        };
        let complete = {
            let observer = observer.clone();
            move || observer.on_completed()
        };
        let error = {
            let observer = observer.clone();
            move |error| observer.on_error(error)
        };
        self.original.subscribe((next, error, complete));
        Subscription::new(|| observer.dispose())
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
                .filter(|x| *x > 1)
                .subscribe(move |x| {
                    data.lock().unwrap().push(x);
                });
        }
        obs.on_next(1);
        obs.on_next(2);
        obs.on_next(3);

        assert_eq!(&vec![2, 3], &*data.lock().unwrap());
    }
}