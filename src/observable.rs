use crate::observer::Observer;
use crate::BaseObserver;

pub trait Observable<'a> {
    type Item: 'a;
    type Error: 'a;
    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + 'a) -> Subscription<'a>;
}

pub struct Subscription<'a> {
    unsubscribe: Box<dyn FnOnce() + 'a>
}

unsafe impl<'a> Send for Subscription<'a> {}
unsafe impl<'a> Sync for Subscription<'a> {}

impl<'a> Subscription<'a> {
    pub fn new(f: impl FnOnce() + 'a) -> Self {
        Self { unsubscribe: Box::new(f) }
    }

    pub fn unsubscribe(self) {
        (self.unsubscribe)()
    }
}

pub struct BaseObservable<'a, I: 'a, E: 'a> {
    subscribe: Box<dyn FnOnce(BaseObserver<'a, I, E>) + 'a>,
}

unsafe impl<'a, I, E> Send for BaseObservable<'a, I, E> {}
unsafe impl<'a, I, E> Sync for BaseObservable<'a, I, E> {}

impl<'a, I, E> BaseObservable<'a, I, E> {
    pub fn new<F>(subscribe: F) -> Self where F: FnOnce(BaseObserver<'a, I, E>) + 'a {
        Self { subscribe: Box::new(subscribe) }
    }
}

impl<'a, I, E> Observable<'a> for BaseObservable<'a, I, E> {
    type Item = I;
    type Error = E;

    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + 'a) -> Subscription<'a> {
        let subscribe = self.subscribe;
        let observer = BaseObserver::new(observer);
        subscribe(observer.clone());
        Subscription::new(move || observer.dispose())
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::BaseObservable;
    use std::rc::Rc;
    use std::cell::RefCell;

    #[test]
    fn sync() {
        let obs = BaseObservable::<i32, ()>::new(|sub| {
            sub.on_next(1);
            sub.on_next(2);
            sub.on_next(3);
        });
        let data = Rc::new(RefCell::new(Vec::new()));
        {
            let data = data.clone();
            obs.subscribe(move |x| {
                data.borrow_mut().push(x);
            });
        }
        assert_eq!(&vec![1, 2, 3], &*data.borrow_mut());
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
        let data = Rc::new(RefCell::new(Vec::new()));
        let sub = {
            let data = data.clone();
            obs.subscribe(move |x| {
                data.borrow_mut().push(x);
            })
        };

        assert_eq!(&Vec::<i32>::new(), &*data.borrow_mut());

        let millis = std::time::Duration::from_millis(150);
        std::thread::sleep(millis);
        assert_eq!(&vec![1, 2, 3], &*data.borrow_mut());

        let millis = std::time::Duration::from_millis(100);
        std::thread::sleep(millis);
        assert_eq!(&vec![1, 2, 3, 4], &*data.borrow_mut());
        sub.unsubscribe();

        let millis = std::time::Duration::from_millis(100);
        std::thread::sleep(millis);
        assert_eq!(&vec![1, 2, 3, 4], &*data.borrow_mut());
    }
}

