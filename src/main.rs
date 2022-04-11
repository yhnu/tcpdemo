use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use anyhow::{Result, anyhow};
mod pb;

#[derive(Debug)]
pub struct  Connection{
    _conn: Option<TcpStream>,
    is_connected: bool,
    pub socket_addr: String
}

impl Connection {
    fn new(socket_addr: String) -> Connection {
        Connection{
            _conn: None,
            is_connected: false,
            socket_addr
        }
    }

    pub fn connect(&mut self) -> Result<()> {
        if let Ok(stream) = TcpStream::connect(String::from("12")) {
            println!("Connected to the server!");
            self._conn = Some(stream.try_clone()?);
            self.is_connected = true;
        } else {
            println!("Couldn't connect to server...");
        }
        Ok(())
    }

    pub fn settimeout(&mut self, timeout:u64) -> Result<()>{
        match &self._conn {
            Some(conn) => {
                conn.set_read_timeout(Some(Duration::from_secs(timeout)));
                conn.set_write_timeout(Some(Duration::from_secs(timeout)));
                Ok(())
            }
            None => {
                Err(anyhow!("No connection"))
            }
        }
    }

    pub fn is_connected(&mut self) -> bool {
        self.is_connected
    }

    pub fn disconnect(&mut self) {
        if self._conn.is_some() && self.is_connected() {
            let conn = self._conn.as_ref().unwrap();
            println!("will shutdown {:?}", conn.peer_addr());
            conn.shutdown(Shutdown::Both).expect("shutdown call failed");
        }
    }

    pub fn send(&mut self, buf: &[u8]) -> Result<()> {
        return match &mut self._conn {
            Some(conn) => {
                let r = conn.write_all(buf);
                println!("send {:?}, {:?}", buf, r.err());

                Ok(())
            }
            None => {
                Err(anyhow!("No conn"))
            }
        };
    }

    pub fn recv(&mut self, len: usize) -> Result<Vec<u8>> {
        if self._conn.is_some() {
            let mut conn = self._conn.as_ref().unwrap();
            let mut data = vec![0 as u8; len];
            conn.read_exact(&mut data);
            Ok(data.to_vec())
        } else {
            Err(anyhow!("No connection"))
        }
    }
}

pub struct  MessageStreamConnection {
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
    conn: Connection,
    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u8>>
}
impl MessageStreamConnection {
    pub fn new(socket_addr: String) -> Self {
        let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();
        MessageStreamConnection{
            stop: Arc::new(AtomicBool::new(false)),
            handle: None,
            conn: Connection::new(socket_addr),
            tx,
            rx
        }
    }

    pub fn send_message(&mut self, msg : Vec<u8>) {
        self.tx.send(msg);
    }

    pub fn start_threads(&mut self, rx :Receiver<Vec<u8>>) {
        let stop = self.stop.clone();
        self.handle = Some(thread::spawn(move || {
            // some work here
            loop {
                if !stop.load(Ordering::SeqCst) {
                    break;
                }
                match rx.try_recv() {
                    Ok(msg) => {

                        let len = msg.len() as u32;
                        let len = len.to_be_bytes();
                        self.conn.send(&len);
                        self.conn.send(&msg);
                        // DogClient::send_message(&mut conn, msg);
                    }
                    _ => {}
                }
            }
        }));
    }
}

fn main() {
}
