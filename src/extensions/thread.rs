use crate::observable::Observable;
use crate::observer::Observer;
use crate::{Scheduler, Subscription, BaseObserver};
use std::sync::{Arc, Mutex};

pub struct ThreadObservable<O> {
    scheduler: Scheduler,
    original: O,
}

unsafe impl<O> Send for ThreadObservable<O> {}
unsafe impl<O> Sync for ThreadObservable<O> {}

pub trait ThreadExt<'a>: Observable<'a> + Sized {
    fn subscribe_on(self, scheduler: Scheduler) -> ThreadObservable<Self> where Self: 'a {
        ThreadObservable { scheduler, original: self }
    }
}

impl<'a, O> ThreadExt<'a> for O where O: Observable<'a> {}

impl<O> Observable<'static> for ThreadObservable<O> where O: Observable<'static> + Send + Sync + 'static {
    type Item = O::Item;
    type Error = O::Error;

    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + 'static) -> Subscription<'static> {
        let scheduler = self.scheduler;
        let observer = BaseObserver::new(observer);
        let observable = self.original;
        let sub = Arc::new(Mutex::new(None));
        let send_sub = sub.clone();
        scheduler.run(move || {
            let sub = observable.subscribe(observer);
            send_sub.lock().unwrap().replace(sub);
        });
        Subscription::new(move || {
            if let Some(sub) = sub.lock().unwrap().take() {
                sub.unsubscribe()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use crate::prelude::*;
    use crate::BaseObservable;
    use crate::Scheduler;

    #[test]
    fn it_works() {
        let obs = BaseObservable::<i32, ()>::new(|sub| {
            let ten_millis = std::time::Duration::from_millis(20);
            std::thread::sleep(ten_millis);
            sub.on_next(1);
            sub.on_next(2);
            sub.on_next(3);
            sub.on_completed();
        });
        let data = Arc::new(Mutex::new(Vec::new()));
        {
            let data = data.clone();
            obs
                .subscribe_on(Scheduler::new_thread())
                .subscribe(move |x| {
                    data.lock().unwrap().push(x);
                });
        }
        let expected: Vec<i32> = vec![];
        assert_eq!(&expected, &*data.lock().unwrap());
        let ten_millis = std::time::Duration::from_millis(50);
        std::thread::sleep(ten_millis);
        assert_eq!(&vec![1, 2, 3], &*data.lock().unwrap());
    }
}
