extern crate futures;
extern crate futures_cpupool;

use futures::{Future, BoxFuture, Stream};
use futures_cpupool::CpuPool;

fn run(n: u64) -> BoxFuture<u64, ()> {
    let res = (0..n).fold(0, |sum, x| sum + x);
    futures::done(Ok(res)).boxed()
}

fn main() {
    let pool = CpuPool::new(4);
    // let a = pool.spawn_fn(move || { run(100000) });
    // let b = pool.spawn_fn(move || { run(10000) });
    // let c = pool.spawn_fn(move || { run(1000) });
    let a = pool.spawn(run(100000));
    let b = pool.spawn(run(10000));
    let c = pool.spawn(run(1000));

    let futures = vec!(a,b,c);
    let stream = futures::stream::futures_unordered(futures.into_iter());

    let iterate = stream.for_each(|v| {
        println!("Value {}", v);
        Ok(())
    });

    iterate.wait().unwrap();

    // This collects all futures into a vecto
    // let result = stream.collect().wait().unwrap();

    // for r in result {
    //     println!("result {:?}", r);
    //
    // }

}
