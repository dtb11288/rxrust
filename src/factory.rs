use crate::observer::Observer;
use crate::observable::Observable;
use crate::{BaseObserver, BaseObservable, Subject};

pub fn create<'a, I, E>(subscribe: impl FnOnce(BaseObserver<'a, I, E>) + 'a) -> BaseObservable<'a, I, E> where I: 'a, E: 'a {
    BaseObservable::new(subscribe)
}

pub fn subject<'a, I, E>() -> Subject<'a, I, E> {
    Subject::new()
}

pub fn empty<'a, I, E>() -> impl Observable<'a, Item=I, Error=E> where I: 'a, E: 'a {
    create(move |sub| sub.on_completed())
}

pub fn never<'a, I, E>() -> impl Observable<'a, Item=I, Error=E> where I: 'a, E: 'a {
    create(move |_| {})
}

pub fn throw<'a, I, E>(error: E) -> impl Observable<'a, Item=I, Error=E> where I: 'a, E: 'a {
    BaseObservable::new(|sub: BaseObserver<'a, I, E>| sub.on_error(error))
}

pub fn from_iter<'a, I, E>(iter: impl IntoIterator<Item=I> + 'a) -> impl Observable<'a, Item=I, Error=E> where I: 'a, E: 'a {
    create(move |sub| {
        iter.into_iter().for_each(|x| sub.on_next(x));
        sub.on_completed();
    })
}

pub fn from_result<'a, I, E>(res: Result<I, E>) -> impl Observable<'a, Item=I, Error=E> where I: 'a, E: 'a {
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

pub fn from_option<'a, I, E>(opt: Option<I>) -> impl Observable<'a, Item=I, Error=E> where I: 'a, E: 'a {
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
