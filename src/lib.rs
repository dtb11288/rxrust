mod observable;
mod observer;

pub mod prelude {
    pub use crate::observable::Observable;
    pub use crate::observer::Observer;
}

pub use observable::{BaseObservable, Subscription};
pub use observer::BaseObserver;
