use crate::observable::Observable;
use crate::observer::Observer;
use crate::{Subscription, BaseObserver};
use std::sync::{Mutex, Arc};

pub struct CombineObservable<O, OO> {
    original: O,
    other: OO,
}

pub trait CombineExt<'a>: Observable<'a> + Sized {
    fn combine<O>(self, other: O) -> CombineObservable<Self, O> where O: Observable<'a, Error=<Self as Observable<'a>>::Error> + 'a {
        CombineObservable { original: self, other }
    }
}

impl<'a, O> CombineExt<'a> for O where O: Observable<'a> {}

impl<'a, O, OO> Observable<'a> for CombineObservable<O, OO>
    where
        O: Observable<'a> + 'a,
        OO: Observable<'a, Error=O::Error> + 'a,
        <O as Observable<'a>>::Item: Send + Sync,
        <OO as Observable<'a>>::Item: Send + Sync,
{
    type Item = (O::Item, OO::Item);
    type Error = O::Error;

    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + Send + Sync + 'a) -> Subscription<'a> {
        let item_1: Arc<Mutex<Option<O::Item>>> = Arc::new(Mutex::new(None));
        let item_2: Arc<Mutex<Option<OO::Item>>> = Arc::new(Mutex::new(None));
        let observer = BaseObserver::new(observer);
        let next_1 = {
            let item_1 = item_1.clone();
            let item_2 = item_2.clone();
            let observer = observer.clone();
            move |item| {
                let item_2 = unsafe { std::mem::transmute_copy(&*item_2.lock().unwrap()) };
                item_1.lock().unwrap().replace(unsafe { std::mem::transmute_copy(&item) });
                if let Some(item_2) = item_2 {
                    observer.on_next((item, item_2))
                }
            }
        };
        let next_2 = {
            let item_1 = item_1.clone();
            let item_2 = item_2.clone();
            let observer = observer.clone();
            move |item| {
                let item_1 = unsafe { std::mem::transmute_copy(&*item_1.lock().unwrap()) };
                item_2.lock().unwrap().replace(unsafe { std::mem::transmute_copy(&item) });
                if let Some(item_1) = item_1 {
                    observer.on_next((item_1, item))
                }
            }
        };
        let complete = {
            let completed = Arc::new(Mutex::new(false));
            let observer = observer.clone();
            move || {
                if *&*completed.lock().unwrap() {
                    observer.on_completed()
                } else {
                    *completed.lock().unwrap() = true;
                }
            }
        };
        let error = move |error| observer.on_error(error);
        let obs_1 = (next_1, error.clone(), complete.clone());
        let obs_2 = (next_2, error, complete);
        let sub1 = self.original.subscribe(obs_1);
        let sub2 = self.other.subscribe(obs_2);
        Subscription::new(move || {
            sub1.unsubscribe();
            sub2.unsubscribe();
        })
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
            let finish = data.clone();
            input.fork()
                .combine(input2.fork())
                .subscribe((
                    move |(x1, x2)| { data.lock().unwrap().push((x1, x2)); },
                    move |_| {},
                    move || { finish.lock().unwrap().push((10, 10)); }
                ));
        }

        input.on_next(1);
        input2.on_next(2);
        input.on_next(3);
        input2.on_next(1);
        input.on_next(2);
        input.on_completed();
        input2.on_completed();

        assert_eq!(&vec![(1, 2), (3, 2), (3, 1), (2, 1), (10, 10)], &*data.lock().unwrap());
    }
}
