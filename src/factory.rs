use crate::observer::Observer;
use crate::observable::Observable;
use crate::{BaseObserver, BaseObservable, Subject};

pub fn create<I, E>(subscribe: impl FnOnce(BaseObserver<I, E>) + 'static) -> BaseObservable<I, E> {
    BaseObservable::new(subscribe)
}

pub fn subject<I, E>() -> Subject<I, E> {
    Subject::new()
}

pub fn empty<I, E>() -> impl Observable<Item=I, Error=E> where I: 'static, E: 'static {
    create(move |sub| sub.on_completed())
}

pub fn never<I, E>() -> impl Observable<Item=I, Error=E> where I: 'static, E: 'static {
    create(move |_| {})
}

pub fn throw<I, E>(error: E) -> impl Observable<Item=I, Error=E> where I: 'static, E: 'static {
    BaseObservable::new(|sub: BaseObserver<I, E>| sub.on_error(error))
}

pub fn from_value<I, E>(i: I) -> impl Observable<Item=I, Error=E> where I: 'static, E: 'static {
    create(move |sub| {
        sub.on_next(i);
        sub.on_completed();
    })
}

pub fn from_iter<I, E>(iter: impl IntoIterator<Item=I> + 'static) -> impl Observable<Item=I, Error=E> where I: 'static, E: 'static {
    create(move |sub| {
        iter.into_iter().for_each(|x| sub.on_next(x));
        sub.on_completed();
    })
}

pub fn from_result<I, E>(res: Result<I, E>) -> impl Observable<Item=I, Error=E> where I: 'static, E: 'static {
    create(move |sub| {
        match res {
            Ok(item) => {
                sub.on_next(item);
                sub.on_completed();
            }
            Err(err) => sub.on_error(err),
        }
    })
}

pub fn from_option<I, E>(opt: Option<I>) -> impl Observable<Item=I, Error=E> where I: 'static, E: 'static {
    create(|sub| {
        match opt {
            Some(item) => {
                sub.on_next(item);
                sub.on_completed();
            }
            None => sub.on_completed(),
        }
    })
}
