use rkey::Server;

fn main() {

    let mut server = Server::new();

    println!("Starting server...");
    server.listen("127.0.0.1:6379").unwrap();


}