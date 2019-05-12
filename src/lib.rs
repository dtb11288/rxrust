mod observable;
mod observer;
mod subject;
mod extensions;

pub mod prelude {
    pub use crate::observable::Observable;
    pub use crate::observer::Observer;
    pub use crate::extensions::multicast::ShareExt;
    pub use crate::extensions::map::MapExt;
    pub use crate::extensions::filter::FilterExt;
    pub use crate::extensions::tap::TapExt;
    pub use crate::extensions::fold::FoldExt;
    pub use crate::extensions::merge::MergeExt;
}

pub use observable::{BaseObservable, Subscription};
pub use observer::BaseObserver;
pub use subject::Subject;

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use crate::prelude::*;
    use crate::Subject;

    #[test]
    fn it_works() {
        let input = Subject::<i32, ()>::new();
        let once = Arc::new(Mutex::new(Vec::new()));
        let data1 = Arc::new(Mutex::new(Vec::new()));
        let data2 = Arc::new(Mutex::new(Vec::new()));
        {
            let data = once.clone();
            let obs = input.fork()
                .tap(move |x| {
                    data.lock().unwrap().push(*x);
                })
                .share();

            let data = data1.clone();
            obs
                .fork()
                .filter(|x| **x > 1)
                .map(|x| *x * 2)
                .subscribe(move |x| {
                    data.lock().unwrap().push(x);
                });

            let data = data2.clone();
            obs
                .fork()
                .filter(|x| **x < 4)
                .fold(0, |sum, x| { sum + *x })
                .subscribe(move |x| {
                    data.lock().unwrap().push(x);
                });
        }
        input.on_next(1);
        input.on_next(2);
        input.on_next(3);
        input.on_next(4);

        assert_eq!(&vec![1, 2, 3, 4], &*once.lock().unwrap());
        assert_eq!(&vec![4, 6, 8], &*data1.lock().unwrap());
        assert_eq!(&vec![1, 3, 6], &*data2.lock().unwrap());
    }
}
