use std::rc::Rc;
use std::cell::RefCell;
use crate::observable::Observable;
use crate::observer::Observer;
use crate::{BaseObserver, Subscription};

type ObserverBundle<'a, I, E> = Rc<RefCell<Vec<BaseObserver<'a, Rc<I>, Rc<E>>>>>;

pub struct Multicast<'a, I, E> {
    observers: ObserverBundle<'a, I, E>,
    subscription: Rc<RefCell<Option<Subscription<'a>>>>,
}

unsafe impl<'a, I, E> Send for Multicast<'a, I, E> {}
unsafe impl<'a, I, E> Sync for Multicast<'a, I, E> {}

pub trait ShareExt<'a>: Observable<'a> + Sized {
    fn share(self) -> Multicast<'a, <Self as Observable<'a>>::Item, <Self as Observable<'a>>::Error>
        where Self: 'a, <Self as Observable<'a>>::Item: 'a, <Self as Observable<'a>>::Error: 'a
    { Multicast::new(self) }
}

impl<'a, O> ShareExt<'a> for O where O: Observable<'a> {}

impl<'a, I, E> Multicast<'a, I, E> {
    pub fn new<O>(original: O) -> Self where O: Observable<'a, Item=I, Error=E> + 'a {
        let observers: ObserverBundle<'a, I, E> = Rc::new(RefCell::new(Vec::new()));
        let next = {
            let observers = observers.clone();
            move |item: I| {
                let item = Rc::new(item);
                observers.borrow_mut().iter().for_each(move |o| o.on_next(item.clone()))
            }
        };
        let complete = {
            let observers = observers.clone();
            move || {
                observers.borrow_mut().drain(..).for_each(move |o| o.on_completed())
            }
        };
        let error = {
            let observers = observers.clone();
            move |error: E| {
                let error = Rc::new(error);
                observers.borrow_mut().drain(..).for_each(move |o| o.on_error(error.clone()))
            }
        };
        let sub = original.subscribe((next, error, complete));
        Self { observers, subscription: Rc::new(RefCell::new(Some(sub))) }
    }

    pub fn fork(&self) -> Self {
        Self { observers: self.observers.clone(), subscription: self.subscription.clone() }
    }
}

impl<'a, I, E> Observable<'a> for Multicast<'a, I, E> {
    type Item = Rc<I>;
    type Error = Rc<E>;

    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + 'a) -> Subscription<'a> {
        let observer = BaseObserver::new(observer);
        self.observers.borrow_mut().push(observer.clone());
        Subscription::new(move || {
            let mut observers = self.observers.borrow_mut();
            if let Some(index) = observers.iter().position(|o| o.id() == observer.id()) {
                observers.remove(index);
            }
            if observers.is_empty() {
                if let Some(sub) = self.subscription.borrow_mut().take() {
                    sub.unsubscribe()
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::BaseObservable;
    use std::rc::Rc;
    use std::cell::RefCell;

    #[test]
    fn it_works() {
        let obs = BaseObservable::<i32, ()>::new(|sub| {
            std::thread::spawn(move || {
                let millis = std::time::Duration::from_millis(10);
                std::thread::sleep(millis);
                sub.on_next(1);

                let millis = std::time::Duration::from_millis(10);
                std::thread::sleep(millis);
                sub.on_next(2);

                let millis = std::time::Duration::from_millis(10);
                std::thread::sleep(millis);
                sub.on_next(3);
            });
        }).share();
        let share_data = Rc::new(RefCell::new(Vec::new()));
        {
            let data = share_data.clone();
            obs.fork().subscribe(move |x: Rc<i32>| {
                data.borrow_mut().push(*&*x);
            });
            let data = share_data.clone();
            obs.fork().subscribe(move |x: Rc<i32>| {
                data.borrow_mut().push(*&*x);
            });
            let data = share_data.clone();
            obs.fork().subscribe(move |x: Rc<i32>| {
                data.borrow_mut().push(*&*x);
            });
        }
        let millis = std::time::Duration::from_millis(50);
        std::thread::sleep(millis);
        assert_eq!(&vec![1, 1, 1, 2, 2, 2, 3, 3, 3], &*share_data.borrow_mut());
    }
}

