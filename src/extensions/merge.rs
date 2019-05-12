use crate::observable::Observable;
use crate::observer::Observer;
use crate::{Subscription, BaseObserver};

pub struct MergeObservable<O, OO> {
    original: O,
    other: OO,
}

unsafe impl<O, OO> Send for MergeObservable<O, OO> {}
unsafe impl<O, OO> Sync for MergeObservable<O, OO> {}

pub trait MergeExt<'a>: Observable<'a> + Sized {
    fn merge<O>(self, other: O) -> MergeObservable<Self, O> where O: Observable<'a, Item=<Self as Observable<'a>>::Item, Error=<Self as Observable<'a>>::Error> + 'a {
        MergeObservable { original: self, other }
    }
}

impl<'a, O> MergeExt<'a> for O where O: Observable<'a> {}

impl<'a, O, OO> Observable<'a> for MergeObservable<O, OO> where O: Observable<'a> + 'a, OO: Observable<'a, Item=O::Item, Error=O::Error> + 'a {
    type Item = O::Item;
    type Error = O::Error;

    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + 'a) -> Subscription<'a> {
        let observer = BaseObserver::new(observer);
        let next = {
            let observer = observer.clone();
            move |item| observer.on_next(item)
        };
        let complete = {
            let observer = observer.clone();
            move || observer.on_completed()
        };
        let error = {
            let observer = observer.clone();
            move |error| observer.on_error(error)
        };
        let obs = (next, error, complete);
        self.original.subscribe(obs.clone());
        self.other.subscribe(obs);
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
        let input = Subject::<i64, ()>::new();
        let input2 = Subject::<i64, ()>::new();
        let data = Arc::new(Mutex::new(Vec::new()));
        {
            let data = data.clone();
            input.fork()
                .merge(input2.fork())
                .subscribe(move |x| {
                    data.lock().unwrap().push(x);
                });
        }

        input.on_next(1);
        input2.on_next(2);
        input.on_next(3);
        input2.on_next(1);
        input.on_next(2);
        input2.on_next(3);

        assert_eq!(&vec![1, 2, 3, 1, 2, 3], &*data.lock().unwrap());
    }
}
