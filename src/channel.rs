use std::collections::HashMap;
use std::io::{Read, Result, Write};
use std::sync::Arc;
use std::time::Duration;
use std::net::{Shutdown, SocketAddr};
use mio::*;
use mio::net::TcpStream;
use acceptor::*;
use eventloop::*;
use chashmap::CHashMap;

pub type Closure = Box<Fn(&mut ChanCtx) + Send + Sync>;

#[derive(Clone)]
pub enum OptionValue {
    NUMBER(usize),
    BOOL(bool),
}

pub struct Channel {
    pub receive_handler: Arc<Closure>,
    pub ready_handler: Arc<Closure>,
    pub close_handler: Arc<Closure>,
    pub ctx: ChanCtx,
}

impl Channel {
    pub fn create(
        stream: &mut TcpStream,
        addr: &SocketAddr,
        id: Token,
        opts: HashMap<String, OptionValue>,
        ready: Arc<Closure>,
        receive: Arc<Closure>,
        close: Arc<Closure>,
    ) -> Channel {
        Channel {
            ready_handler: ready,
            receive_handler: receive,
            close_handler: close,
            ctx: ChanCtx::new(addr, stream, id, opts),
        }
    }

    pub fn register(&self, selector: &Poll) {
        selector.register(
            &self.ctx.chan,
            self.ctx.id,
            Ready::readable(),
            PollOpt::edge(),
        );
    }
}

pub struct ChanCtx {
    remote_addr: SocketAddr,
    chan: TcpStream,
    id: Token,
    options: HashMap<String, OptionValue>,
    closed: bool,
}

impl ChanCtx {
    pub fn new(
        addr: &SocketAddr,
        stream: &mut TcpStream,
        chan_id: Token,
        opts: HashMap<String, OptionValue>,
    ) -> ChanCtx {
        let ch = stream.try_clone().unwrap();
        for (k, ref v) in opts.iter() {
            match k.as_ref() {
                "ttl" => match v {
                    OptionValue::NUMBER(ttl) => {
                        ch.set_ttl(*ttl as u32);
                    }
                    OptionValue::BOOL(_) => {}
                },
                "linger" => match v {
                    OptionValue::NUMBER(linger) => {
                        ch.set_linger(Some(Duration::from_millis(*linger as u64)));
                    }
                    OptionValue::BOOL(_) => {}
                },
                "nodelay" => match v {
                    OptionValue::NUMBER(_) => {}
                    OptionValue::BOOL(b) => {
                        ch.set_nodelay(*b);
                    }
                },
                "keep_alive" => match v {
                    OptionValue::NUMBER(keepalive) => {
                        ch.set_keepalive(Some(Duration::from_millis(*keepalive as u64)));
                    }
                    OptionValue::BOOL(_) => {}
                },
                "recv_buf_size" => match v {
                    OptionValue::NUMBER(bufsize) => {
                        ch.set_recv_buffer_size(*bufsize);
                    }
                    OptionValue::BOOL(_) => {}
                },
                "send_buf_size" => match v {
                    OptionValue::NUMBER(bufsize) => {
                        ch.set_send_buffer_size(*bufsize);
                    }
                    OptionValue::BOOL(_) => {}
                },
                _ => {}
            }
        }
        ChanCtx {
            remote_addr: addr.clone(),
            chan: ch,
            id: chan_id,
            options: opts,
            closed: false,
        }
    }

    pub fn close(&mut self) {
        self.closed = true;
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.chan.write_all(data)
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        self.chan.read_exact(buf)
    }

    //TODO pri
    pub fn check_buffer(&self, buf: &mut [u8]) -> Result<usize> {
        self.chan.peek(buf)
    }

    pub fn chan_id(&self) -> Token {
        self.id
    }
}
