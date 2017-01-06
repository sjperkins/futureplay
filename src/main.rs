#![feature(conservative_impl_trait)]
extern crate chrono;
extern crate futures;
extern crate futures_cpupool;

use std::thread;

use chrono::{DateTime, UTC};
use futures::{Future, Stream};

struct Fibonacci { curr: usize, next: usize, }

// Implement `Iterator` for `Fibonacci`.
impl Iterator for Fibonacci {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        let old = self.curr;
        let new = self.curr + self.next;
        self.curr = self.next;
        self.next = new;
        Some(old)
    }
}

/// Returns nth Fibonacci sequence number
fn fibonacci(n: usize) -> usize {
    Fibonacci { curr: 0, next: 1 }.nth(n).unwrap()
}

struct WorkResult {
    pub n: usize,
    pub thread: thread::Thread,
    pub start_time: DateTime<UTC>,
    pub sum: usize,
}

fn work(n: usize) -> impl Future<Item=WorkResult, Error=usize> {
    let result = match n {
        // Fail on 1000 to create an or_else() case on the stream
        1000 => Err(n),
        // Default case
        _ => {
            Ok(WorkResult {
                n: n,
                thread: thread::current(),
                start_time: UTC::now(),
                sum: fibonacci(n)
            })
        }
    };

    // Returned a boxed future of the result
    futures::done(result)
}

fn main() {
    // Create a thread pool with thread name prefixes
    // alternatively, could use:
    //let pool = futures_cpupool::CpuPool::new_num_cpus();
    let pool = futures_cpupool::Builder::new()
        .name_prefix("pool_thread_")
        .create();

    // Arguments for our work function
    let args = vec!(1000000, 100000, 10000, 1000);

    let iterate = args.into_iter()
        // pool.spawn_fn creates a lazy future for the cpu pool to execute
        // shorthand for pool.spawn(futures::lazy(move || work(v)))
        .map(|v| pool.spawn_fn(move || work(v)));

    // Iterate over the cpu futures, printing results
    let stream = futures::stream::futures_unordered(iterate)
        // Handle the error case in work
        .or_else(|e| pool.spawn_fn(move || work(e+1)))
        // Handle result case of work
        .for_each(|r| {
            let finish_time = UTC::now();
            println!("Argument     {}\n\
                      Result       {}\n\
                      Ran on thead {}\n\
                      Started at   {}\n\
                      Completed at {}\n\
                      Duration     {}\n",
                        r.n,
                        r.sum,
                        r.thread.name().unwrap(),
                        r.start_time.format("%H:%M:%S%.6f"),
                        finish_time.format("%H:%M:%S%.6f"),
                        finish_time - r.start_time);
            Ok(())
        });

    // Execute the stream
    stream.wait().unwrap();
}
