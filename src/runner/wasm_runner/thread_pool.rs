use log::{debug, info};
/// This custom thread-pool implementation is study from https://crates.io/crates/threadpool
/// But the condition variable we use implements blocking queue instead of channel
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

/// The data for a thread pool
struct ThreadPoolEntry {
    /// A name for the thread-to-be, for identification in panic messages (readonly)
    _thread_name: Option<String>,
    /// The size of the stack for the spawned thread in bytes  (readonly)
    _stack_size: Option<usize>,

    /// The job queue
    _job_queue: Mutex<VecDeque<Job>>,
    // The condition variable for job queue
    _job_queue_not_empty: Condvar,

    /// The number of threads in pool
    _thread_num: AtomicUsize,
    /// The running state thread numbers
    _active_thread_num: AtomicUsize,
    /// The panicked thread numbers
    _panicked_thread_num: AtomicUsize,

    /// The mutex and condition variable for join
    _join_mutex: Mutex<()>,
    _join_cond_var: Condvar,
}

/// [```ThreadPool```]
///
/// # Examples
///
/// ```
/// let thread_num = 10;
/// let job_num = 100;
/// let pool = ThreadPool::new(thread_num, None, None);
/// for _job in 0..job_num {
///     pool.execute(move || {
///         assert_eq!(1, 1);
///     })
/// }
/// pool.join();
/// ```
#[derive(Clone)]
pub(crate) struct ThreadPool {
    _inner: Arc<ThreadPoolEntry>,
}

impl ThreadPool {
    pub(crate) fn new(
        thread_num: usize,
        thread_name: Option<String>,
        stack_size: Option<usize>,
    ) -> Self {
        info!(
            "Start thread pool `{}`, thread number is {}",
            thread_name.as_ref().unwrap_or(&"None".to_string()),
            thread_num
        );

        let pool = Self {
            _inner: Arc::new(ThreadPoolEntry {
                _thread_name: thread_name,
                _stack_size: stack_size,
                _job_queue: Mutex::new(VecDeque::new()),
                _job_queue_not_empty: Condvar::default(),
                _thread_num: AtomicUsize::new(thread_num),
                _active_thread_num: AtomicUsize::new(0),
                _panicked_thread_num: AtomicUsize::new(0),
                _join_mutex: Mutex::default(),
                _join_cond_var: Condvar::default(),
            }),
        };

        for _ in 0..thread_num {
            // create threads
            pool.spawn_one();
        }

        pool
    }

    #[inline(always)]
    pub(crate) fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let mut q = self._inner._job_queue.lock().unwrap();
        q.push_back(Box::new(f));
        self._inner._job_queue_not_empty.notify_one();
    }

    #[inline(always)]
    pub(crate) fn queued_job_num(&self) -> usize {
        self._inner._job_queue.lock().unwrap().len()
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub(crate) fn active_thread_num(&self) -> usize {
        self._inner._active_thread_num.load(Ordering::Relaxed)
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub(crate) fn panicked_thread_num(&self) -> usize {
        self._inner._panicked_thread_num.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub(crate) fn thread_num(&self) -> usize {
        self._inner._thread_num.load(Ordering::Relaxed)
    }

    pub(crate) fn set_thread_num(&self, size: usize) {
        let old_size = self._inner._thread_num.swap(size, Ordering::Release);
        if old_size < size {
            // if expand, spawn the new threads
            for _ in old_size..size {
                self.spawn_one();
            }
        }
    }

    /// get a job from job queue
    fn get_job(&self) -> Option<Job> {
        let mut q = self._inner._job_queue.lock().unwrap();
        while q.is_empty() {
            q = self._inner._job_queue_not_empty.wait(q).unwrap();
        }

        // active number increase
        self._inner
            ._active_thread_num
            .fetch_add(1, Ordering::SeqCst);

        q.pop_front()
    }

    /// spawn a new thread for a thread pool
    fn spawn_one(&self) {
        let mut builder = thread::Builder::new();
        if let Some(ref name) = self._inner._thread_name {
            builder = builder.name(name.clone());
        }
        if let Some(stack_size) = self._inner._stack_size {
            builder = builder.stack_size(stack_size);
        }

        let pool = self.clone();
        builder
            .spawn(move || {
                let mut sentinel = Sentinel::new(&pool);

                loop {
                    if pool._inner._active_thread_num.load(Ordering::SeqCst) > pool.thread_num() {
                        break; // shrink
                    }

                    let job = match pool.get_job() {
                        Some(val) => val,
                        None => {
                            break;
                        }
                    };

                    job(); // may throw panic, and caught by sentinel

                    let previous = pool
                        ._inner
                        ._active_thread_num
                        .fetch_sub(1, Ordering::SeqCst);
                    if previous == 1 && pool.queued_job_num() == 0 {
                        // notify all join thread
                        pool._inner._join_cond_var.notify_all();
                    }
                }

                sentinel.cancel(); // normally stop
            })
            .unwrap();
    }

    #[inline(always)]
    #[allow(dead_code)]
    fn has_work(&self) -> bool {
        self._inner._active_thread_num.load(Ordering::SeqCst) > 0 || self.queued_job_num() > 0
    }

    #[allow(dead_code)]
    pub(crate) fn join(&self) {
        if !self.has_work() {
            return;
        }

        // wait for no jobs in pool
        let mut lock = self._inner._join_mutex.lock().unwrap();
        while self.has_work() {
            lock = self._inner._join_cond_var.wait(lock).unwrap();
        }
    }
}

/// for fix the panicked thread in thread pool
struct Sentinel<'a> {
    _pool: &'a ThreadPool,
    _active: bool,
}

impl<'a> Sentinel<'a> {
    fn new(thread_pool: &'a ThreadPool) -> Self {
        Self {
            _pool: thread_pool,
            _active: true,
        }
    }

    fn cancel(&mut self) {
        self._active = false;
    }
}

impl<'a> Drop for Sentinel<'a> {
    fn drop(&mut self) {
        if self._active {
            let previous = self
                ._pool
                ._inner
                ._active_thread_num
                .fetch_sub(1, Ordering::SeqCst);

            if previous == 1 && self._pool.queued_job_num() == 0 {
                self._pool._inner._join_cond_var.notify_all();
            }

            if std::thread::panicking() {
                debug!("{:?} panic", thread::current());
                self._pool
                    ._inner
                    ._panicked_thread_num
                    .fetch_add(1, Ordering::SeqCst);
            }
            self._pool.spawn_one(); // spawn a new thread in pool to fix the panicked thread
        }
    }
}

#[cfg(test)]
mod test {
    use super::ThreadPool;
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        sync::{Arc, Barrier},
        thread,
        thread::sleep,
        time::Duration,
    };

    #[test]
    fn test_active() {
        let thread_num = 5;
        let b_start = Arc::new(Barrier::new(thread_num + 1));
        let b_end = Arc::new(Barrier::new(thread_num + 1));

        let pool = ThreadPool::new(thread_num, None, None);
        for _job in 0..thread_num {
            let _b_start = b_start.clone();
            let _b_end = b_end.clone();
            pool.execute(move || {
                _b_start.wait();
                _b_end.wait();
            });
        }

        b_start.wait();

        assert_eq!(thread_num, pool.thread_num());
        assert_eq!(thread_num, pool.active_thread_num());
        assert_eq!(0, pool.queued_job_num());
        assert_eq!(0, pool.panicked_thread_num());

        b_end.wait();
    }

    #[test]
    fn test_panic() {
        let thread_num = 5;
        let pool = ThreadPool::new(thread_num, Some("thread_name".parse().unwrap()), None);

        let exec_num = Arc::new(AtomicUsize::new(0));
        for _job in 0..thread_num {
            pool.execute(move || {
                assert_eq!(thread::current().name().unwrap(), "thread_name");
                panic!("{:?} should panic\n", thread::current().id());
            });
        }
        for _job in 0..thread_num {
            let e = exec_num.clone();
            pool.execute(move || {
                e.fetch_add(1, Ordering::Release);
            })
        }

        pool.join();
        assert_eq!(thread_num, pool.thread_num());
        assert_eq!(0, pool.active_thread_num());
        assert_eq!(0, pool.queued_job_num());
        assert_eq!(thread_num, pool.thread_num());
        assert_eq!(thread_num, exec_num.load(Ordering::Acquire));
    }

    #[test]
    fn test_shrink() {
        let before = 10;
        let after = 2;
        let pool = ThreadPool::new(before, None, None);
        for _job in 0..before {
            pool.execute(move || {
                assert_eq!(1, 1);
            })
        }

        pool.set_thread_num(after); // shrink
        for _job in 0..(after * 2) {
            pool.execute(move || {
                sleep(Duration::from_secs(20));
            })
        }

        sleep(Duration::from_secs(1));
        assert_eq!(after, pool.thread_num());
        // assert_eq!(after, pool.active_thread_num());
        // assert_eq!(after, pool.queued_job_num());
        assert_eq!(0, pool.panicked_thread_num());
    }

    #[test]
    fn test_expand() {
        let pool = ThreadPool::new(1, None, None);
        for _job in 0..5 {
            pool.execute(move || {
                sleep(Duration::from_millis(500));
            })
        }

        // firstly: [0]
        // then: [0,1,2,3,4]
        sleep(Duration::from_millis(100));
        assert_eq!(1, pool.thread_num());
        assert_eq!(1, pool.active_thread_num());
        assert_eq!(4, pool.queued_job_num());

        pool.set_thread_num(5);
        sleep(Duration::from_millis(100));
        assert_eq!(5, pool.thread_num());
        assert_eq!(5, pool.active_thread_num());

        pool.join();
    }

    #[test]
    fn test_empty() {
        let thread_num = 10;
        let pool = ThreadPool::new(thread_num, None, None);
        assert_eq!(thread_num, pool.thread_num());
        assert_eq!(0, pool.panicked_thread_num());
        assert_eq!(0, pool.active_thread_num());
        assert_eq!(0, pool.queued_job_num());

        pool.join();
    }

    #[test]
    fn test_join() {
        let thread_num = 10;
        let test_num: usize = 50;
        let pool = ThreadPool::new(thread_num, None, None);
        let exec_num = Arc::new(AtomicUsize::new(0));

        for _job in 0..test_num {
            let num = exec_num.clone();
            pool.execute(move || {
                num.fetch_add(1, Ordering::Release);
            })
        }

        pool.join();
        assert_eq!(test_num, exec_num.load(Ordering::Acquire));

        for _job in 0..test_num {
            let num = exec_num.clone();
            pool.execute(move || {
                num.fetch_add(1, Ordering::Release);
            })
        }

        for _joins in 0..4 {
            let _pool = pool.clone();
            let _exec_num = exec_num.clone();
            let t = thread::Builder::new();
            t.spawn(move || {
                _pool.join();
                assert_eq!(test_num * 2, _exec_num.load(Ordering::Acquire));
            })
            .unwrap();
        }

        pool.join();
        assert_eq!(test_num * 2, exec_num.load(Ordering::Acquire));
    }
}
