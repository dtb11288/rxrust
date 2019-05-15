type Executor = Box<dyn FnOnce() + Send + Sync + 'static>;

pub struct Scheduler {
    executor: Box<dyn FnOnce(Executor) + Send + Sync + 'static>
}

impl Scheduler {
    pub fn new_thread() -> Self {
        Self::new(|f| {
            std::thread::spawn(move || f());
        })
    }

    pub fn new(executor: impl FnOnce(Executor) + Send + Sync + 'static) -> Self {
        Self { executor: Box::new(executor) }
    }

    pub fn run(self, exec: impl FnOnce() + Send + Sync + 'static) {
        let executor = self.executor;
        executor(Box::new(exec))
    }
}
