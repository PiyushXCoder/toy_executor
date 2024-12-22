mod executor;

fn main() {
    println!("Starting...");
    let ex = executor::ToyExecutor::new();
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
