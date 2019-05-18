use crate::observable::Observable;
use crate::observer::Observer;
use std::rc::Rc;
use std::cell::Cell;
use crate::{Subscription, BaseObserver};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::SystemTime;

pub struct FlatMapObservable<FM, O> {
    and_then: FM,
    original: O,
}

unsafe impl<FM, O> Send for FlatMapObservable<FM, O> {}
unsafe impl<FM, O> Sync for FlatMapObservable<FM, O> {}

pub trait FlatMapExt<'a>: Observable<'a> + Sized {
    fn and_then<FM, OO>(self, and_then: FM) -> FlatMapObservable<FM, Self> where OO: Observable<'a> + 'a, FM: Fn(Self::Item) -> OO + 'a {
        FlatMapObservable { and_then, original: self }
    }
}

impl<'a, O> FlatMapExt<'a> for O where O: Observable<'a> {}

impl<'a, FM, O, OO> Observable<'a> for FlatMapObservable<FM, O>
    where O: Observable<'a, Error=OO::Error> + 'a,
          OO: Observable<'a> + 'a,
          FM: Fn(O::Item) -> OO + 'a,
{
    type Item = OO::Item;
    type Error = OO::Error;

    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + 'a) -> Subscription<'a> {
        let and_then = self.and_then;
        let observer = BaseObserver::new(observer);
        let completed = Rc::new(Cell::new(false));
        let subs = Rc::new(Mutex::new(HashMap::new()));
        let next = {
            let observer = observer.clone();
            let completed = completed.clone();
            let subs = subs.clone();
            move |item| {
                let id = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().subsec_nanos();
                let observable = and_then(item);
                let observer = observer.clone();
                let next = {
                    let observer = observer.clone();
                    move |item| observer.on_next(item)
                };
                let complete = {
                    let observer = observer.clone();
                    let completed = completed.clone();
                    let subs = subs.clone();
                    move || {
                        let mut subs = subs.lock().unwrap();
                        subs.remove(&id);
                        if completed.get() && subs.len() == 0 {
                            observer.on_completed()
                        }
                    }
                };
                let error = move |error| observer.on_error(error);
                let sub = observable.subscribe((next, error, complete));
                subs.lock().unwrap().insert(id, sub);
            }
        };
        let complete = {
            let observer = observer.clone();
            let subs = subs.clone();
            move || {
                completed.set(true);
                if subs.lock().unwrap().len() == 0 {
                    observer.on_completed()
                }
            }
        };
        let error = move |error| observer.on_error(error);
        let sub = self.original.subscribe((next, error, complete));
        Subscription::new(move || {
            subs.lock().unwrap().drain().into_iter().for_each(move |(_, sub)| sub.unsubscribe());
            sub.unsubscribe();
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use crate::prelude::*;
    use crate::{Subject, BaseObservable};

    #[test]
    fn it_works() {
        let input = Subject::<i64, &'static str>::new();
        let data = Arc::new(Mutex::new(Vec::new()));
        {
            let data = data.clone();
            let finish = data.clone();
            input.fork()
                .and_then(move |x| {
                    BaseObservable::new(move |sub| {
                        let millis = std::time::Duration::from_millis(50);
                        std::thread::sleep(millis);
                        std::thread::spawn(move || {
                            sub.on_next(x + 1);
                            let millis = std::time::Duration::from_millis(50);
                            std::thread::sleep(millis);
                            sub.on_next(x + 2);
                            let millis = std::time::Duration::from_millis(50);
                            std::thread::sleep(millis);
                            sub.on_completed();
                        });
                    })
                })
                .subscribe((
                    move |x| {
                        data.lock().unwrap().push(x);
                    },
                    move |_| {
                        assert_eq!("this", "never happen");
                    },
                    move || {
                        finish.lock().unwrap().push(10);
                    },
                ));
        }

        input.on_next(1);
        let millis = std::time::Duration::from_millis(50);
        std::thread::sleep(millis);
        input.on_next(2);
        let millis = std::time::Duration::from_millis(50);
        std::thread::sleep(millis);
        input.on_next(3);
        let millis = std::time::Duration::from_millis(50);
        std::thread::sleep(millis);
        input.on_completed();

        let millis = std::time::Duration::from_millis(20);
        std::thread::sleep(millis);
        assert_eq!(&vec![2, 3, 3, 4, 4, 5], &*data.lock().unwrap());

        let millis = std::time::Duration::from_millis(200);
        std::thread::sleep(millis);
        assert_eq!(&vec![2, 3, 3, 4, 4, 5, 10], &*data.lock().unwrap());
    }
}
