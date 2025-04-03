#![allow(unused)]

use std::{io::{Read, Write}, net::{SocketAddr, TcpListener, TcpStream}, thread};

use crate::{blockchain::BlockChain, transaction::TransactionPool};

pub struct Node {
    pub address: SocketAddr,
    pub connected_nodes: Vec<SocketAddr>,
    pub blockchain: BlockChain,
    pub mempool: TransactionPool,
}

impl Node {
    pub fn new(address: SocketAddr) -> Node {
        
        // 서버만 생성, 필요할 때 
        // 부트스트랩 노드한테서 블록체인 값 받아오기
        // 노드 생성

        Node {
            address,
            connected_nodes: Vec::<SocketAddr>::new(),
            blockchain: BlockChain::new(),
            mempool: TransactionPool::new(),
        }
    }

    pub fn connect(&mut self, peer_addr: SocketAddr) {
        if !self.connected_nodes.contains(&peer_addr) {
            self.connected_nodes.push(peer_addr);
            println!("Connected to: {}", peer_addr);
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 512];

    loop {
        match stream.read(&mut buffer) {
            Ok(size) if size > 0 => {
                let message = String::from_utf8_lossy(&buffer[..size]);
                println!("Received: {}", message);

                let response = format!("Message received: {}", message);
                stream.write(response.as_bytes()).unwrap();
            },
            Ok(_) => {
                println!("Client Disconnected.");
                break;
            }
            Err(e) => {
                println!("Failed to read from stream: {e}");
                break;
            }
        }
    }
}

// p2p에서는 수신용
fn start_server(address: SocketAddr) {
    let listener = TcpListener::bind(address).unwrap();
    println!("Listening on {}", address);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move ||{
                    handle_client(stream);
                });
            },
            Err(e) => {
                println!("Connectiond failed: {}", e);
            }
        }
    }
}

// p2p에서는 전송용
fn start_client(address: SocketAddr, message: &str) {
    match TcpStream::connect(address) {
        Ok(mut stream) => {
            stream.write(message.as_bytes()).unwrap();

            let mut buf = [0; 512];

            match stream.read(&mut buf) {
                Ok(size) => {
                    let response = String::from_utf8_lossy(&buf[..size]);
                    println!("Server response: {}", response);
                },
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Falied to connect: {}", e);
        }
    }
}

fn broadcast_message(peers: Vec<SocketAddr>, msg: &str) {
    for peer in peers {
        start_client(peer, msg);
    }
}