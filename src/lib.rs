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
        let share_data = Arc::new(Mutex::new(Vec::new()));
        {
            let data = share_data.clone();
            let obs = input.fork()
                .tap(move |x| {
                    let x = x.clone();
                    data.lock().unwrap().push(x);
                })
                .share();

            let data = share_data.clone();
            obs
                .fork()
                .filter(|x| *x > 1)
                .map(|x| x * 2)
                .subscribe(move |x| {
                    data.lock().unwrap().push(x);
                });

            let data = share_data.clone();
            obs
                .fork()
                .filter(|x| *x < 3)
                .map(|x| x * 5)
                .subscribe(move |x| {
                    data.lock().unwrap().push(x);
                });
        }
        input.on_next(1);
        input.on_next(2);
        input.on_next(3);

        assert_eq!(&vec![
            1, 5,
            2, 4, 10,
            3, 6
        ], &*share_data.lock().unwrap());
    }
}
