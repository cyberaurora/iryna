#![feature(match_default_bindings)]
mod eventloop;
mod channel;
mod acceptor;

extern crate concurrent_hashmap;
extern crate mio;

#[cfg(test)]
mod tests {

    use std;
    use std::io::Write;
    use std::io::Result;
    use eventloop::*;
    use channel::*;
    use acceptor::*;

    #[test]
    fn it_works() {
        Acceptor::new()
            .worker_count(4)
            .bind("127.0.0.1", 12345)
            .on_receive(|ref mut ch| {
                ch.write("Hello, world.\n".as_bytes());
                Ok(())
            })
            .on_ready(|ref mut ch| {
                ch.write("Welcome.\n".as_bytes());
                Ok(())
            })
            .accept();
        std::thread::sleep_ms(100000);
    }
}
