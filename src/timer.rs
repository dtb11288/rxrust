use crate::observable::Observable;
use crate::observer::Observer;
use crate::Subscription;

pub struct Interval {
    time: i32
}

impl Interval {
    pub fn new(e: i32) -> Self {
        Self { time: e }
    }
}

impl<'a> Observable<'a> for Interval {
    type Item = i32;
    type Error = ();

    fn subscribe(self, observer: impl Observer<Self::Item, Self::Error> + 'a) -> Subscription<'a> {
        Subscription::new(|| { /* destroy interval */ })
    }
}