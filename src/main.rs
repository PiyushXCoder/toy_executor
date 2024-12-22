mod executor;

use std::{
    future::Future,
    sync::{Arc, Mutex},
    task::{Poll, Waker},
    thread,
    time::{Duration, Instant},
};

struct Delay {
    when: Instant,
    waker: Option<Arc<Mutex<Waker>>>,
}

impl Future for Delay {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if Instant::now() >= self.when {
            println!("Finished!");
            return Poll::Ready(());
        }

        if let Some(waker) = &self.waker {
            let mut waker = waker.lock().unwrap();

            if !waker.will_wake(cx.waker()) {
                *waker = cx.waker().clone();
            }
        } else {
            let when = self.when;
            let waker = Arc::new(Mutex::new(cx.waker().clone()));

            thread::spawn(move || {
                let now = Instant::now();

                println!("Starting the wait!");

                if now < when {
                    thread::sleep(when - now);
                }

                let waker = waker.lock().unwrap();
                waker.wake_by_ref();
            });
        };

        Poll::Pending
    }
}

fn main() {
    println!("Starting...");
    let ex = executor::ToyExecutor::new();

    ex.spawn(Delay {
        when: Instant::now() + Duration::from_secs(3),
        waker: None,
    });

    ex.spawn(async {
        futures::join!(
            async {
                for i in 0..10 {
                    println!("- {}", i);
                }
            },
            async {
                for i in 0..10 {
                    println!("* {}", i);
                }
            }
        );
    });

    ex.run();
}
