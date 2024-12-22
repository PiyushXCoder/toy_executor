use std::{
    future::Future,
    pin::Pin,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    task::{Context, Poll},
};

use futures::task::{self, ArcWake};

pub struct ToyExecutor {
    scheduled: Receiver<Arc<Task>>,
    scheduler: Sender<Arc<Task>>,
}

pub struct Task {
    task_future: Mutex<TaskFuture>,
    scheduler: Sender<Arc<Task>>,
}

pub struct TaskFuture {
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
    poll: Poll<()>,
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.schedule();
    }
}

impl ToyExecutor {
    pub fn new() -> Self {
        let (tx, rx) = channel::<Arc<Task>>();

        Self {
            scheduled: rx,
            scheduler: tx,
        }
    }

    pub fn spawn<F>(&self, fut: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Task::spawn(fut, &self.scheduler);
    }

    pub fn run(&self) {
        while let Ok(task) = self.scheduled.recv() {
            task.poll();
        }
    }
}

impl Task {
    fn spawn<F>(fut: F, scheduler: &Sender<Arc<Task>>)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            task_future: Mutex::new(TaskFuture::new(fut)),
            scheduler: scheduler.clone(),
        });

        scheduler
            .send(task)
            .expect("Unable to send the task while spawning!");
    }

    fn poll(self: &Arc<Self>) {
        let waker = task::waker(self.clone());
        let mut cx = Context::from_waker(&waker);

        let mut task_future = self.task_future.try_lock().unwrap();

        task_future.poll(&mut cx);
    }

    fn schedule(self: &Arc<Self>) {
        self.scheduler.send(self.clone()).unwrap();
    }
}

impl TaskFuture {
    fn new<F>(fut: F) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        TaskFuture {
            future: Box::pin(fut),
            poll: Poll::Pending,
        }
    }

    fn poll(&mut self, cx: &mut Context<'_>) {
        if self.poll.is_pending() {
            self.poll = self.future.as_mut().poll(cx);
        }
    }
}
