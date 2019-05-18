use std::rc::Rc;
use std::cell::RefCell;
use crate::observer::Observer;
use crate::observable::Observable;
use crate::{BaseObserver, Subscription};

type ObserverBundle<I, E> = Rc<RefCell<Option<BaseObserver<I, E>>>>;

pub struct Subject<I, E> {
    subscriber: ObserverBundle<I, E>
}

unsafe impl<I, E> Send for Subject<I, E> {}
unsafe impl<I, E> Sync for Subject<I, E> {}

impl<I, E> Subject<I, E> {
    pub fn new() -> Self {
        Self { subscriber: Rc::new(RefCell::new(None)) }
    }

    pub fn fork(&self) -> Self {
        Self { subscriber: self.subscriber.clone() }
    }
}

impl<I, E> Observable for Subject<I, E> where I: 'static, E: 'static {
    type Item = I;
    type Error = E;

    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + 'static) -> Subscription {
        let observer = BaseObserver::new(observer);
        self.subscriber.borrow_mut().replace(observer.clone());
        Subscription::new(move || observer.dispose())
    }
}

impl<I, E> Observer<I, E> for Subject<I, E> {
    fn on_next(&self, item: I) {
        if let Some(sub) = self.subscriber.borrow_mut().as_ref() {
            sub.on_next(item)
        }
    }

    fn on_error(self, error: E) {
        if let Some(sub) =  self.subscriber.borrow_mut().take() {
            sub.on_error(error)
        }
    }

    fn on_completed(self) {
        if let Some(sub) =  self.subscriber.borrow_mut().take() {
            sub.on_completed()
        }
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
        let data = Arc::new(Mutex::new(Vec::new()));
        {
            let data = data.clone();
            input.fork()
                .subscribe(move |x| {
                    data.lock().unwrap().push(x);
                });
        }

        std::thread::spawn(move || {
            input.on_next(1);
            input.on_next(2);
            input.on_next(3);
        });

        let millis = std::time::Duration::from_millis(10);
        std::thread::sleep(millis);
        assert_eq!(&vec![1, 2, 3], &*data.lock().unwrap());
    }
}
