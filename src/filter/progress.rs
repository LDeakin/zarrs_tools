use std::{
    sync::{atomic::AtomicUsize, Mutex},
    time::{Duration, Instant},
};

pub struct ProgressStats {
    pub step: usize,
    pub num_steps: usize,
    pub read: Duration,
    pub process: Duration,
    pub process_steps: Vec<Duration>,
    pub write: Duration,
}

pub struct Progress<'a> {
    progress_callback: &'a ProgressCallback<'a>,
    step: AtomicUsize,
    num_steps: usize,
    duration_read: Mutex<Duration>,
    duration_process: Mutex<Duration>,
    duration_process_steps: Mutex<Vec<Duration>>,
    duration_write: Mutex<Duration>,
    // Chunk cache hit/cache miss?
    // Bytes read/written?
}

impl<'a> Progress<'a> {
    pub fn new(num_steps: usize, progress_callback: &'a ProgressCallback) -> Self {
        let progress: Progress = Self {
            progress_callback,
            step: AtomicUsize::new(0),
            num_steps,
            duration_read: Mutex::new(Duration::ZERO),
            duration_process: Mutex::new(Duration::ZERO),
            duration_process_steps: Mutex::new(vec![]),
            duration_write: Mutex::new(Duration::ZERO),
        };
        progress.update(0);
        progress
    }

    pub fn read<F: FnOnce() -> T, T>(&self, f: F) -> T {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();
        *self.duration_read.lock().unwrap() += elapsed;
        result
    }

    pub fn process<F: FnOnce() -> T, T>(&self, f: F) -> T {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();
        *self.duration_process.lock().unwrap() += elapsed;
        result
    }

    pub fn process_step<F: FnOnce() -> T, T>(&self, step: usize, f: F) -> T {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();
        {
            let mut steps = self.duration_process_steps.lock().unwrap();
            if step >= steps.len() {
                steps.resize(step + 1, Duration::ZERO);
            }
            steps[step] += elapsed;
        }
        result
    }

    pub fn write<F: FnOnce() -> T, T>(&self, f: F) -> T {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();
        *self.duration_write.lock().unwrap() += elapsed;
        result
    }

    fn update(&self, step: usize) {
        let read = *self.duration_read.lock().unwrap();
        let process = *self.duration_process.lock().unwrap();
        let process_steps = self.duration_process_steps.lock().unwrap().clone();
        let write = *self.duration_write.lock().unwrap();
        let stats = ProgressStats {
            step,
            num_steps: self.num_steps,
            read,
            process,
            process_steps,
            write,
        };
        self.progress_callback.update(stats);
    }

    pub fn next(&self) {
        let step = 1 + self.step.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.update(step);
    }
}

pub struct ProgressCallback<'a> {
    callback: &'a (dyn Fn(ProgressStats) + Send + Sync),
}

impl<'a> ProgressCallback<'a> {
    pub fn new(callback: &'a (dyn Fn(ProgressStats) + Send + Sync)) -> Self {
        Self { callback }
    }

    pub fn update(&self, stats: ProgressStats) {
        (self.callback)(stats);
    }
}
