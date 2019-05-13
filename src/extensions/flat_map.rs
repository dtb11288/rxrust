use crate::observable::Observable;
use crate::observer::Observer;
use std::rc::Rc;
use std::cell::RefCell;
use crate::{Subscription, BaseObserver};

pub struct FlatMapObservable<FM, O> {
    and_then: FM,
    original: O,
}

unsafe impl<FM, O> Send for FlatMapObservable<FM, O> {}
unsafe impl<FM, O> Sync for FlatMapObservable<FM, O> {}

pub trait FlatMapExt<'a>: Observable<'a> + Sized {
    fn and_then<FM, OO>(self, and_then: FM) -> FlatMapObservable<FM, Self> where OO: Observable<'a> + 'a, FM: Fn(Self::Item) -> OO + Clone + 'a {
        FlatMapObservable { and_then, original: self }
    }
}

impl<'a, O> FlatMapExt<'a> for O where O: Observable<'a> {}

impl<'a, FM, O, OO> Observable<'a> for FlatMapObservable<FM, O>
    where O: Observable<'a> + 'a,
          OO: Observable<'a, Error=O::Error> + 'a,
          FM: Fn(O::Item) -> OO + Clone + 'a,
{
    type Item = OO::Item;
    type Error = O::Error;

    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + 'a) -> Subscription<'a> {
        let and_then = self.and_then;
        let observer = BaseObserver::new(observer);
        let completed = Rc::new(RefCell::new(false));
        let counting = Rc::new(RefCell::new(0));
        let all_completed = Rc::new(RefCell::new(0));
        let next = {
            let observer = observer.clone();
            let counting = counting.clone();
            let completed = completed.clone();
            let all_completed = all_completed.clone();
            move |item| {
                let observable = and_then(item);
                *counting.clone().borrow_mut() += 1;
                let observer = observer.clone();
                let next = {
                    let observer = observer.clone();
                    move |item| observer.on_next(item)
                };
                let complete = {
                    let observer = observer.clone();
                    let completed = completed.clone();
                    let counting = counting.clone();
                    let all_completed = all_completed.clone();
                    move || {
                        if *&*completed.borrow() && &*counting.borrow() == &*all_completed.borrow() {
                            observer.on_completed()
                        } else {
                            *all_completed.borrow_mut() += 1;
                        }
                    }
                };
                let error = |_| {};
                observable.subscribe((next, error, complete));
            }
        };
        let complete = {
            let observer = observer.clone();
            move || {
                completed.replace(true);
                if &*counting.borrow() == &*all_completed.borrow() {
                    observer.on_completed()
                }
            }
        };
        let error = {
            move |_e| {}
        };
        self.original.subscribe((next, error, complete));
        Subscription::new(|| observer.dispose())
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
                        sub.on_next(x + 1);
                        sub.on_next(x + 2);
                        sub.on_completed();
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
        input.on_next(2);
        input.on_next(3);
        input.on_completed();

        assert_eq!(&vec![2, 3, 3, 4, 4, 5, 10], &*data.lock().unwrap());
    }
}
