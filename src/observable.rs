use crate::observer::Observer;
use crate::BaseObserver;

pub trait Observable<'a> {
    type Item: 'a;
    type Error: 'a;
    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + Send + Sync + 'a) -> Subscription<'a>;
    fn into_boxed(self) -> Box<Self> where Self: Sized + 'a {
        Box::new(self)
    }
}

pub struct Subscription<'a> {
    unsubscribe: Box<dyn FnOnce() + Send + Sync + 'a>
}

impl<'a> Subscription<'a> {
    pub fn new<F>(f: F) -> Self where F: FnOnce() + Send + Sync + 'a {
        Self { unsubscribe: Box::new(f) }
    }

    pub fn unsubscribe(self) {
        (self.unsubscribe)()
    }
}

pub struct BaseObservable<'a, I: 'a, E: 'a> {
    subscribe: Box<dyn FnOnce(BaseObserver<'a, I, E>) + Send + Sync + 'a>,
}

impl<'a, I, E> BaseObservable<'a, I, E> {
    pub fn new<F>(subscribe: F) -> Self where F: FnOnce(BaseObserver<'a, I, E>) + Send + Sync + 'a {
        Self { subscribe: Box::new(subscribe) }
    }
}

impl<'a, I, E> Observable<'a> for BaseObservable<'a, I, E> {
    type Item = I;
    type Error = E;

    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + Send + Sync + 'a) -> Subscription<'a> {
        let subscribe = self.subscribe;
        let observer = BaseObserver::new(observer);
        subscribe(observer.clone());
        Subscription::new(move || observer.dispose())
    }
}

impl<'a, O> Observable<'a> for Box<O> where O: Observable<'a> + 'a {
    type Item = O::Item;
    type Error = O::Error;

    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + Send + Sync + 'a) -> Subscription<'a> {
        let sub = (*self).subscribe(observer);
        Subscription::new(move || sub.unsubscribe())
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::BaseObservable;
    use std::sync::{Mutex, Arc};

    #[test]
    fn sync() {
        let obs = BaseObservable::<i32, ()>::new(|sub| {
            sub.on_next(1);
            sub.on_next(2);
            sub.on_next(3);
        });
        let data = Arc::new(Mutex::new(Vec::new()));
        {
            let data = data.clone();
            obs.subscribe(move |x| {
                data.lock().unwrap().push(x);
            });
        }
        assert_eq!(&vec![1, 2, 3], &*data.lock().unwrap());
    }

    #[test]
    fn r#async() {
        let obs = BaseObservable::<i32, ()>::new(|sub| {
            std::thread::spawn(move || {
                let millis = std::time::Duration::from_millis(100);
                std::thread::sleep(millis);
                sub.on_next(1);
                sub.on_next(2);
                sub.on_next(3);
                let millis = std::time::Duration::from_millis(100);
                std::thread::sleep(millis);
                sub.on_next(4);
                let millis = std::time::Duration::from_millis(100);
                std::thread::sleep(millis);
                sub.on_next(5);
            });
        });
        let data = Arc::new(Mutex::new(Vec::new()));
        {
            let data = data.clone();
            std::thread::spawn(move || {
                let data = data.clone();
                let sub = obs.subscribe(move |x| {
                    data.lock().unwrap().push(x);
                });

                let millis = std::time::Duration::from_millis(250);
                std::thread::sleep(millis);
                sub.unsubscribe();
            });
        }

        assert_eq!(&Vec::<i32>::new(), &*data.lock().unwrap());

        let millis = std::time::Duration::from_millis(150);
        std::thread::sleep(millis);
        assert_eq!(&vec![1, 2, 3], &*data.lock().unwrap());

        let millis = std::time::Duration::from_millis(100);
        std::thread::sleep(millis);
        assert_eq!(&vec![1, 2, 3, 4], &*data.lock().unwrap());

        let millis = std::time::Duration::from_millis(100);
        std::thread::sleep(millis);
        assert_eq!(&vec![1, 2, 3, 4], &*data.lock().unwrap());
    }
}

