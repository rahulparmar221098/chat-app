git clone https://github.com/rahulparmar221098/chat-app.git

cd chat-app

cargo build --release

Server: cargo run --release -p server -- --port 9000
Client: cargo run --release -p client -- --host 127.0.0.1 --port 9000 --username username
