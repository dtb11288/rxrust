use std::rc::Rc;
use std::cell::RefCell;

pub trait Observer<I, E> {
    fn on_next(&self, item: I);
    fn on_error(self, error: E);
    fn on_completed(self);
}

type ObserverBundle<'a, I, E> = Rc<RefCell<Option<Box<dyn BoxedObserver<I, E> + 'a>>>>;

pub struct BaseObserver<'a, I: 'a, E: 'a> {
    observer: ObserverBundle<'a, I, E>
}

impl<'a, I, E> Clone for BaseObserver<'a, I, E> {
    fn clone(&self) -> Self {
        Self { observer: self.observer.clone() }
    }
}

unsafe impl<'a, I, E> Send for BaseObserver<'a, I, E> {}
unsafe impl<'a, I, E> Sync for BaseObserver<'a, I, E> {}

impl<'a, I, E> BaseObserver<'a, I, E> {
    pub fn new(observer: impl Observer<I, E> + 'a) -> Self {
        Self { observer: Rc::new(RefCell::new(Some(Box::new(observer)))) }
    }

    pub fn dispose(self) {
        self.observer.borrow_mut().take();
    }
}

impl<'a, I, E> Observer<I, E> for BaseObserver<'a, I, E> {
    fn on_next(&self, item: I) {
        if let Some(observer) = self.observer.borrow_mut().as_ref() {
            observer.on_next(item)
        }
    }

    fn on_error(self, error: E) {
        if let Some(observer) = self.observer.borrow_mut().take() {
            observer.on_error_box(error)
        }
    }

    fn on_completed(self) {
        if let Some(observer) = self.observer.borrow_mut().take() {
            observer.on_completed_box()
        }
    }
}

impl<I, F, E> Observer<I, E> for F where F: Fn(I) + Clone {
    fn on_next(&self, item: I) {
        self(item);
    }

    fn on_error(self, _error: E) {
        panic!("observable unexpected error");
    }

    fn on_completed(self) {}
}

impl<I, N, E, Err> Observer<I, Err> for (N, E) where N: Fn(I) + Clone, E: FnOnce(Err) + Clone {
    fn on_next(&self, item: I) {
        self.0(item);
    }

    fn on_error(self, error: Err) {
        self.1(error);
    }

    fn on_completed(self) {}
}

impl<I, N, C, E, Err> Observer<I, Err> for (N, E, C) where N: Fn(I) + Clone, C: FnOnce() + Clone, E: FnOnce(Err) + Clone {
    fn on_next(&self, item: I) {
        self.0(item);
    }

    fn on_error(self, error: Err) {
        self.1(error);
    }

    fn on_completed(self) {
        self.2();
    }
}

trait BoxedObserver<I, E>: Observer<I, E> {
    fn on_completed_box(self: Box<Self>);
    fn on_error_box(self: Box<Self>, error: E);
}

impl<O, I, E> BoxedObserver<I, E> for O where O: Observer<I, E> {
    fn on_completed_box(self: Box<Self>) {
        self.on_completed();
    }

    fn on_error_box(self: Box<Self>, error: E) {
        self.on_error(error);
    }
}
