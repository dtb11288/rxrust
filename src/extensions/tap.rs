use crate::observable::Observable;
use crate::observer::Observer;
use crate::{Subscription, BaseObserver};

pub struct TapObservable<T, O> {
    tap: T,
    original: O,
}

unsafe impl<T, O> Send for TapObservable<T, O> {}
unsafe impl<T, O> Sync for TapObservable<T, O> {}

pub trait TapExt: Observable + Sized {
    fn tap<M>(self, tap: M) -> TapObservable<M, Self> where M: Fn(&Self::Item) {
        TapObservable { tap, original: self }
    }
}

impl<O> TapExt for O where O: Observable {}

impl<T, O> Observable for TapObservable<T, O> where O: Observable, T: for<'a> Fn(&'a O::Item) + Clone + 'static, O::Item: 'static, O::Error: 'static {
    type Item = O::Item;
    type Error = O::Error;

    fn subscribe(self, obs: impl Observer<Self::Item, Self::Error> + 'static) -> Subscription {
        let tap = self.tap;
        let observer = BaseObserver::new(obs);
        let next = {
            let observer = observer.clone();
            move |item| {
                tap(&item);
                observer.on_next(item);
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