# Reactive Extension written in Rust
This is an subset of extensions that basically working

## Example:
```rust
use rx::prelude::*;
use rx::{Scheduler, BaseObservable, BaseObserver};

fn wait(millis: u64) {
    let millis = std::time::Duration::from_millis(millis);
    std::thread::sleep(millis);
}

fn main() {
    let obs = rx::factory::create(move |sub: BaseObserver<i32, ()>| {
        let mut i = 0;
        loop {
            if i == 20 {
                return sub.on_completed();
            }
            i += 1;
            sub.on_next(i)
        }
    })
        .subscribe_on(Scheduler::new_thread())
        .share();

    let obs1 = obs.fork()
        .map(|x| *&*x)
        .filter(|x| x % 2 == 0 )
        .and_then(|x| {
            BaseObservable::new(move |sub| {
                wait(10);
                sub.on_next(x + 1);
                wait(10);
                sub.on_next(x + 2);
                wait(10);
                sub.on_next(x + 3);
                wait(10);
                sub.on_completed();
            }).subscribe_on(Scheduler::new_thread())
        })
        .fold(0, |sum, x| x + sum);

    let obs2 = obs.fork()
        .map(|x| *&*x)
        .filter(|x| x % 3 == 0 )
        .map(|x| x * 3);

    obs1.merge(obs2)
        .subscribe(|x| {
            println!("x: {}", x);
        });

    wait(500);
}
```
