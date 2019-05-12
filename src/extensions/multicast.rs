use std::sync::{Arc, Mutex};
use crate::observable::Observable;
use crate::observer::Observer;
use crate::{BaseObserver, Subscription};

type ObserverBundle<'a, I, E> = Arc<Mutex<Vec<BaseObserver<'a, I, E>>>>;

pub struct Multicast<'a, I, E> {
    observers: ObserverBundle<'a, I, E>,
}

unsafe impl<'a, I, E> Send for Multicast<'a, I, E> {}
unsafe impl<'a, I, E> Sync for Multicast<'a, I, E> {}

pub trait ShareExt<'a>: Observable<'a> + Sized {
    fn share(self) -> Multicast<'a, <Self as Observable<'a>>::Item, <Self as Observable<'a>>::Error>
        where Self: 'a, <Self as Observable<'a>>::Item: Clone + 'a, <Self as Observable<'a>>::Error: Clone + 'a
    { Multicast::new(self) }
}

impl<'a, O> ShareExt<'a> for O where O: Observable<'a> {}

impl<'a, I, E> Multicast<'a, I, E> where I: Clone + 'a, E: Clone + 'a {
    pub fn new<O>(obs: O) -> Self where O: Observable<'a, Item=I, Error=E> + 'a {
        let observers: ObserverBundle<'a, I, E> = Arc::new(Mutex::new(Vec::new()));
        let next = {
            let observers = observers.clone();
            move |x: I| {
                observers.lock().unwrap().iter().for_each(|o| o.on_next(x.clone()))
            }
        };
        let complete = {
            let observers = observers.clone();
            move || {
                observers.lock().unwrap().drain(..).for_each(|o| o.on_completed())
            }
        };
        let error = {
            let observers = observers.clone();
            move |error: E| {
                observers.lock().unwrap().drain(..).for_each(|o| o.on_error(error.clone()))
            }
        };
        obs.subscribe((next, error, complete));
        Self { observers }
    }

    pub fn fork(&self) -> Self {
        Self { observers: self.observers.clone() }
    }
}

impl<'a, I, E> Observable<'a> for Multicast<'a, I, E> where I: Clone + 'a, E: Clone + 'a {
    type Item = I;
    type Error = E;

    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + 'a) -> Subscription<'a> {
        let observer = BaseObserver::new(observer);
        self.observers.lock().unwrap().push(observer.clone());
        Subscription::new(|| observer.dispose())
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::BaseObservable;
    use std::rc::Rc;
    use std::cell::RefCell;

    #[test]
    fn r#async() {
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
            obs.fork().subscribe(move |x| {
                data.borrow_mut().push(x);
            });
            let data = share_data.clone();
            obs.fork().subscribe(move |x| {
                data.borrow_mut().push(x);
            });
            let data = share_data.clone();
            obs.fork().subscribe(move |x| {
                data.borrow_mut().push(x);
            });
        }
        let millis = std::time::Duration::from_millis(50);
        std::thread::sleep(millis);
        assert_eq!(&vec![1, 1, 1, 2, 2, 2, 3, 3, 3], &*share_data.borrow_mut());
    }
}

