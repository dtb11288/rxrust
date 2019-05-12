use std::sync::{Arc, Mutex};
use crate::observer::Observer;
use crate::observable::Observable;
use crate::{BaseObserver, Subscription};

type ObserverBundle<'a, I, E> = Arc<Mutex<Option<BaseObserver<'a, I, E>>>>;

pub struct Subject<'a, I, E> {
    subscriber: ObserverBundle<'a, I, E>
}

unsafe impl<'a, I, E> Send for Subject<'a, I, E> {}
unsafe impl<'a, I, E> Sync for Subject<'a, I, E> {}

impl<'a, I, E> Subject<'a, I, E> {
    pub fn new() -> Self {
        Self { subscriber: Arc::new(Mutex::new(None)) }
    }

    pub fn fork(&self) -> Self {
        Self { subscriber: self.subscriber.clone() }
    }
}

impl<'a, I, E> Observable<'a> for Subject<'a, I, E> where I: 'a, E: 'a {
    type Item = I;
    type Error = E;

    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + 'a) -> Subscription<'a> {
        let mut subscriber = self.subscriber.lock().unwrap();
        let observer = BaseObserver::new(observer);
        subscriber.replace(observer.clone());
        Subscription::new(move || observer.dispose())
    }
}

impl<'a, I, E> Observer<I, E> for Subject<'a, I, E> {
    fn on_next(&self, item: I) {
        if let Some(sub) = self.subscriber.lock().unwrap().as_ref() {
            sub.on_next(item)
        }
    }

    fn on_error(self, error: E) {
        if self.subscriber.lock().unwrap().is_some() {
            let sub = self.subscriber.lock().unwrap().take().unwrap();
            sub.on_error(error)
        }
    }

    fn on_completed(self) {
        if self.subscriber.lock().unwrap().is_some() {
            let sub = self.subscriber.lock().unwrap().take().unwrap();
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
    fn r#async() {
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
