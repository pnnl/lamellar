use crate::scheduler::{Executor, LamellarExecutor, LamellarTask, LamellarTaskInner};
use tokio::runtime::Runtime;

use futures_util::Future;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct TokioRt {
    max_num_threads: usize,
    rt: Runtime,
}

impl LamellarExecutor for TokioRt {
    fn spawn_task<F>(&self, task: F, executor: Arc<Executor>) -> LamellarTask<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send,
    {
        // trace_span!("spawn_task").in_scope(|| {
        let task = self.rt.spawn(task);
        LamellarTask {
            task: LamellarTaskInner::TokioTask(task),
            executor,
            task_id: 0,
        }
        // })
    }
    fn submit_task<F>(&self, task: F)
    where
        F: Future + Send + 'static,
        F::Output: Send,
    {
        // trace_span!("submit_task").in_scope(|| {
        self.rt.spawn(async move { task.await });
        // });
    }

    fn submit_task_thread<F>(&self, task: F, _: usize)
    where
        F: Future + Send + 'static,
        F::Output: Send,
    {
        // trace_span!("submit_task").in_scope(|| {
        self.rt.spawn(async move { task.await });
        // });
    }
    fn submit_io_task<F>(&self, task: F)
    where
        F: Future + Send + 'static,
        F::Output: Send,
    {
        // trace_span!("submit_task").in_scope(|| {
        self.rt.spawn(async move { task.await });
        // });
    }

    fn submit_immediate_task<F>(&self, task: F)
    where
        F: Future + Send + 'static,
        F::Output: Send,
    {
        // trace_span!("submit_task").in_scope(|| {
        self.rt.spawn(async move { task.await });
        // });
    }

    fn block_on<F: Future>(&self, task: F) -> F::Output {
        // trace_span!("block_on").in_scope(||
        self.rt.block_on(task)
        // )
    }

    // //#[tracing::instrument(skip_all)]
    fn shutdown(&self) {
        // i think we just let tokio do this on drop
        // println!("shutting down tokio runtime");
    }

    // //#[tracing::instrument(skip_all)]
    fn force_shutdown(&self) {
        // i think we just let tokio do this on drop
    }

    // //#[tracing::instrument(skip_all)]
    fn exec_task(&self) {
        // Tokio has no way to run a queued task on the calling thread. Callers sit in a
        // spin loop (barrier, wait_all, blocking_wait, ...) around this call, so yield the
        // core to the workers instead of burning it: with a busy-spinning main thread plus
        // the worker threads, a PE bound to few cores starves its own workers.
        std::thread::yield_now();
    }

    // block_in_place tells tokio's multi-thread runtime this worker thread is about
    // to block, so it spins up/reuses a replacement worker to keep polling other
    // tasks. Only valid inside a runtime worker thread (panics otherwise), so we
    // fall back to calling `f` directly when not on one (e.g. lamellar's MAIN_THREAD).
    fn block_in_place<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::block_in_place(f)
        } else {
            f()
        }
    }

    // fn set_max_workers(&mut self, num_workers: usize) {
    //     self.max_num_threads = num_workers;
    // }

    fn num_workers(&self) -> usize {
        self.max_num_threads
    }

    // fn active(&self) -> bool {
    //     self.status.load(Ordering::SeqCst) == SchedulerStatus::Active as u8
    // }
}

impl TokioRt {
    /// `num_threads` is the requested thread count for the PE and includes the main thread
    /// (as with the work stealing executor). `block_on` does not run pool tasks on the main
    /// thread, so the pool gets one thread fewer, and at least one.
    pub(crate) fn new(num_threads: usize) -> TokioRt {
        TokioRt {
            max_num_threads: num_threads,
            rt: tokio::runtime::Builder::new_multi_thread()
                .worker_threads(num_threads.saturating_sub(1).max(1))
                .enable_all()
                .build()
                .unwrap(),
        }
    }
}
