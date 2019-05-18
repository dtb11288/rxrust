use crate::observer::Observer;
use crate::BaseObserver;

pub trait Observable {
    type Item;
    type Error;
    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + 'static) -> Subscription;
    fn into_boxed(self) -> Box<Self> where Self: Sized {
        Box::new(self)
    }
}

pub struct Subscription {
    unsubscribe: Box<dyn FnOnce() + 'static>
}

unsafe impl Send for Subscription {}
unsafe impl Sync for Subscription {}

impl Subscription {
    pub fn new(f: impl FnOnce() + 'static) -> Self {
        Self { unsubscribe: Box::new(f) }
    }

    pub fn unsubscribe(self) {
        (self.unsubscribe)()
    }
}

pub struct BaseObservable<I, E> {
    subscribe: Box<dyn FnOnce(BaseObserver<I, E>) + 'static>,
}

unsafe impl<I, E> Send for BaseObservable<I, E> {}
unsafe impl<I, E> Sync for BaseObservable<I, E> {}

impl<I, E> BaseObservable<I, E> {
    pub fn new<F>(subscribe: F) -> Self where F: FnOnce(BaseObserver<I, E>) + 'static {
        Self { subscribe: Box::new(subscribe) }
    }
}

impl<I, E> Observable for BaseObservable<I, E> where I: 'static, E: 'static {
    type Item = I;
    type Error = E;

    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + 'static) -> Subscription {
        let subscribe = self.subscribe;
        let observer = BaseObserver::new(observer);
        subscribe(observer.clone());
        Subscription::new(move || observer.dispose())
    }
}

impl<O> Observable for Box<O> where O: Observable {
    type Item = O::Item;
    type Error = O::Error;

    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + 'static) -> Subscription {
        let sub = (*self).subscribe(observer);
        Subscription::new(move || sub.unsubscribe())
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

