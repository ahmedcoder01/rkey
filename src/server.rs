// TODO: create a multi-threaded web server that accepts redis client requests and runs them as commands.

use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex, MutexGuard,
    },
    thread::JoinHandle,
};

use anyhow::{Context};

use crate::{CommandHandler, Resp, RespType, Storage};

pub struct Server {
    pool: Option<ThreadPool>,
    running: Arc<AtomicBool>,
    address: Option<SocketAddr>,
    storage: Arc<Mutex<Storage>>
}

impl Server {
    pub fn new() -> Self {
        let running = Arc::new(AtomicBool::new(false));
        Self {
            pool: None,
            running: Arc::clone(&running),
            address: None,
            storage: Arc::new(Mutex::new(Storage::new()))
        }
    }
    pub fn listen<A: ToSocketAddrs>(&mut self, addr: A) -> anyhow::Result<()> {
        let socket_addr = addr
            .to_socket_addrs()?
            .next()
            .context("Failed to resolve address")?;
        println!("socket addr: {socket_addr}");
        self.address = Some(socket_addr);
        let listener = TcpListener::bind(addr).context("Failed to bind address")?;
        self.running.store(true, Ordering::SeqCst);
        self.pool = Some(ThreadPool::build(10, Arc::clone(&self.running)));

        for stream in listener.incoming() {
            let storage = Arc::clone(&self.storage);
            match stream {
                Ok(stream) => {
                    self.pool.as_ref().unwrap().execute(Box::new(move || {
                        let res = handle_client(stream, storage);
                        // let res = stream.write_all("+OK\r\n".as_bytes());
                        if let Err(e) = res {
                            println!("Failed to write to client: {e}");
                        }
                    }));
                }
                Err(e) => {
                    println!("Failed to connect to client {e}");
                }
            }
        }

        Ok(())
    }

    pub fn close(&mut self) {
        if !self.running.load(Ordering::SeqCst) {
            println!("Server is not running");
        }

        self.running.store(false, Ordering::SeqCst);
    }
}

fn handle_client(mut stream: TcpStream, storage: Arc<Mutex<Storage>>) -> anyhow::Result<()> {
    let mut buf = [0; 1024];

    loop {
        let read_buf: Vec<u8> = match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let mut valid = Vec::new();
                valid.extend_from_slice(&buf[..n]); // Append only the received bytes

                // Debug print: Show raw bytes and their escaped representation
                println!("Raw received: {:?}", &valid);
                println!("Received (escaped): {:?}", String::from_utf8_lossy(&valid));

                valid
            },
            Err(e) => {
                anyhow::bail!(format!("Failed to read from client {e}"));
            }
        };

        if !read_buf.ends_with(b"\r\n") {
            let c = String::from_utf8_lossy(&read_buf);
            println!("skipping current read as the cmd delim is not reached..");
            println!("read: {}", c.escape_debug());
            continue;
        };

        let cmd: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&read_buf);
        let mut cmd_handler = CommandHandler::new(Arc::clone(&storage));
        let req_resp = Resp::new().parse_line(&cmd);
        match req_resp {
            Ok(parsed) => {
                let res = cmd_handler.handle_cmd(parsed);
                match res {
                    Ok(resp_response) => {
                        println!("Writing to client {:?}" , resp_response);
                         stream.write_all(resp_response.serialize().as_bytes())?;
                    },
                    Err(e) => {
                        println!("Error: Writing to client {:?}" , e);
                        //  stream.write_all(format!("Failed to exec command {e}").as_bytes())?; 
                        stream.write_all(RespType::Err(format!("Failed to exec command {e}")).serialize().as_bytes())?;
                    },
                }
                
            },
            Err(e) => {
                println!("Failed to parse cmd {}", e);
            } 
        
        }
        
    }

    Ok(())
}

struct ThreadPool {
    workers: Vec<Worker>,
    pool_size: usize,
    running: Arc<AtomicBool>,
    sender: Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    fn build(pool_size: usize, running: Arc<AtomicBool>) -> Self {
        // std::thread::spawn(f)
        let (tx, rx): (Sender<Job>, Receiver<Job>) = channel();

        let rx = Arc::new(Mutex::new(rx));

        let workers: Vec<Worker> = vec![0; pool_size]
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let rx = Arc::clone(&rx);
                let running = Arc::clone(&running);
                return Worker::new(i, rx, Arc::clone(&running));
            })
            .collect();

        println!("spawned {} workers", workers.len());

        Self {
            workers,
            sender: tx,
            pool_size,
            running,
        }
    }

    fn execute(&self, task: Job) {
        if let Err(e) = self.sender.send(task) {
            eprintln!("Failed to send task to worker thread: {}", e);
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, rx: Arc<Mutex<Receiver<Job>>>, running: Arc<AtomicBool>) -> Self {
        let thread = std::thread::spawn(move || {
            println!("Thread {id}: being spawned...");
            while running.load(Ordering::SeqCst) {
                // if ! {break;}
                let msg = rx.lock().unwrap().recv();
                println!("Thread {id}: Recived a msg");

                if let Err(e) = msg {
                    println!("Thread {id}: failed to recieve msg: {e}");
                    break;
                }

                msg.unwrap()();
            }
        });

        Self {
            id,
            thread: Some(thread),
        }
    }
}

impl Drop for ThreadPool {
    /// Gracefully shut down all workers when the pool is dropped.
    fn drop(&mut self) {
        println!("Shutting down thread pool...");

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().expect("Worker thread failed to join");
            }
        }
    }
}
