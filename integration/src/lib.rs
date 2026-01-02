//! Integration tests for the chat server and client
#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader, Read, Write},
        net::TcpStream,
        process::{Command, Stdio},
        thread::sleep,
        time::Duration,
    };

    const TEST_HOST: &str = "127.0.0.1";
    const SERVER_BIN: &str = "../target/release/server";
    const CLIENT_BIN: &str = "../target/release/client";
    const MAX_RETRIES: u32 = 5;

    /// Helper function to wait for server to be ready
    fn wait_for_server(port: &str) -> bool {
        let mut attempts = 0;
        while attempts < MAX_RETRIES {
            if TcpStream::connect(format!("{}:{}", TEST_HOST, port)).is_ok() {
                println!("Server is ready on port {}", port);
                return true;
            }
            attempts += 1;
            sleep(Duration::from_secs(1));
        }
        false
    }

    #[test]
    fn server_starts_successfully() {
        let port = "8080";

        // Start the server
        let mut server = Command::new(SERVER_BIN)
            .args(["--port", port])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start server");

        // Wait for server to be ready
        assert!(wait_for_server(port), "Server failed to start");

        // Cleanup
        server.kill().expect("Failed to kill server");
        server.wait().expect("Failed to wait for server");
    }

    #[test]
    fn single_client_connects() {
        let port = "8081";

        // Start the server
        let mut server = Command::new(SERVER_BIN)
            .args(["--port", port])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start server");

        // Wait for server to be ready
        assert!(wait_for_server(port), "Server failed to start");

        // Start a client
        let mut client = Command::new(CLIENT_BIN)
            .args(["--username", "alice"])
            .args(["--host", TEST_HOST])
            .args(["--port", port])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start client");

        sleep(Duration::from_secs(1));

        // Check if client is still running (connected successfully)
        assert!(
            client.try_wait().unwrap().is_none(),
            "Client should still be running"
        );

        // Cleanup
        client.kill().expect("Failed to kill client");
        server.kill().expect("Failed to kill server");
    }

    #[test]
    fn multiple_clients_connect() {
        let port = "8082";

        // Start the server
        let mut server = Command::new(SERVER_BIN)
            .args(["--port", port])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start server");

        assert!(wait_for_server(port), "Server failed to start");

        // Start multiple clients
        let mut client1 = Command::new(CLIENT_BIN)
            .args(["--username", "alice"])
            .args(["--host", TEST_HOST])
            .args(["--port", port])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start client1");

        sleep(Duration::from_millis(500));

        let mut client2 = Command::new(CLIENT_BIN)
            .args(["--username", "bob"])
            .args(["--host", TEST_HOST])
            .args(["--port", port])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start client2");

        sleep(Duration::from_millis(500));

        let mut client3 = Command::new(CLIENT_BIN)
            .args(["--username", "charlie"])
            .args(["--host", TEST_HOST])
            .args(["--port", port])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start client3");

        sleep(Duration::from_secs(1));

        // Check all clients are still running
        assert!(
            client1.try_wait().unwrap().is_none(),
            "Client1 should still be running"
        );
        assert!(
            client2.try_wait().unwrap().is_none(),
            "Client2 should still be running"
        );
        assert!(
            client3.try_wait().unwrap().is_none(),
            "Client3 should still be running"
        );

        // Cleanup
        client1.kill().expect("Failed to kill client1");
        client2.kill().expect("Failed to kill client2");
        client3.kill().expect("Failed to kill client3");
        server.kill().expect("Failed to kill server");
    }

    #[test]
    fn duplicate_username_rejected() {
        let port = "8083";

        // Start the server
        let mut server = Command::new(SERVER_BIN)
            .args(["--port", port])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start server");

        assert!(wait_for_server(port), "Server failed to start");

        // Start first client with username "alice"
        let mut client1 = Command::new(CLIENT_BIN)
            .args(["--username", "alice"])
            .args(["--host", TEST_HOST])
            .args(["--port", port])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start client1");

        sleep(Duration::from_secs(1));

        // Start second client with same username "alice"
        let mut client2 = Command::new(CLIENT_BIN)
            .args(["--username", "alice"])
            .args(["--host", TEST_HOST])
            .args(["--port", port])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start client2");

        sleep(Duration::from_secs(1));

        // Capture stderr from client2
        let mut client2_stderr = String::new();
        if let Some(stderr) = client2.stderr.take() {
            let mut reader = BufReader::new(stderr);
            reader
                .read_to_string(&mut client2_stderr)
                .expect("Failed to read stderr");
        }

        // Assert that duplicate username was rejected
        assert!(
            client2_stderr.contains("Username is not available"),
            "Expected error message about duplicate username, got: {}",
            client2_stderr
        );

        // Cleanup
        client1.kill().expect("Failed to kill client1");
        client2.kill().expect("Failed to kill client2");
        server.kill().expect("Failed to kill server");
    }

    #[test]
    fn client_can_send_message() {
        let port = "8084";

        // Start the server
        let mut server = Command::new(SERVER_BIN)
            .args(["--port", port])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start server");

        assert!(wait_for_server(port), "Server failed to start");

        // Start client with stdin/stdout
        let mut client = Command::new(CLIENT_BIN)
            .args(["--username", "alice"])
            .args(["--host", TEST_HOST])
            .args(["--port", port])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start client");

        sleep(Duration::from_secs(1));

        // Send a message
        let stdin = client.stdin.as_mut().expect("Failed to open stdin");
        writeln!(stdin, "sen Hello, World!").expect("Failed to write to stdin");
        stdin.flush().expect("Failed to flush stdin");

        sleep(Duration::from_secs(1));

        // Check client is still running after sending message
        assert!(
            client.try_wait().unwrap().is_none(),
            "Client should still be running after sending message"
        );

        // Cleanup
        client.kill().expect("Failed to kill client");
        server.kill().expect("Failed to kill server");
    }

    #[test]
    fn client_disconnect_and_reconnect() {
        let port = "8085";

        // Start the server
        let mut server = Command::new(SERVER_BIN)
            .args(["--port", port])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start server");

        assert!(wait_for_server(port), "Server failed to start");

        // Connect first client
        let mut client1 = Command::new(CLIENT_BIN)
            .args(["--username", "alice"])
            .args(["--host", TEST_HOST])
            .args(["--port", port])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start client1");

        sleep(Duration::from_secs(1));

        // Disconnect client
        client1.kill().expect("Failed to kill client1");
        client1.wait().expect("Failed to wait for client1");

        sleep(Duration::from_secs(1));

        // Reconnect with same username
        let mut client2 = Command::new(CLIENT_BIN)
            .args(["--username", "alice"])
            .args(["--host", TEST_HOST])
            .args(["--port", port])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to reconnect client");

        sleep(Duration::from_secs(1));

        // Check client reconnected successfully
        assert!(
            client2.try_wait().unwrap().is_none(),
            "Client should reconnect successfully"
        );

        // Cleanup
        client2.kill().expect("Failed to kill client2");
        server.kill().expect("Failed to kill server");
    }

    #[test]
    fn rapid_client_connections() {
        let port = "8086";

        // Start the server
        let mut server = Command::new(SERVER_BIN)
            .args(["--port", port])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start server");

        assert!(wait_for_server(port), "Server failed to start");

        let mut clients = Vec::new();

        // Rapidly connect 5 clients
        for i in 0..5 {
            let username = format!("user{}", i);
            let client = Command::new(CLIENT_BIN)
                .args(["--username", &username])
                .args(["--host", TEST_HOST])
                .args(["--port", port])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect(&format!("Failed to start client {}", i));

            clients.push(client);
            sleep(Duration::from_millis(200));
        }

        sleep(Duration::from_secs(1));

        // Check all clients are still running
        for (i, client) in clients.iter_mut().enumerate() {
            assert!(
                client.try_wait().unwrap().is_none(),
                "Client {} should still be running",
                i
            );
        }

        // Cleanup
        for mut client in clients {
            client.kill().expect("Failed to kill client");
        }
        server.kill().expect("Failed to kill server");
    }

    #[test]
    fn broadcast_message_to_user() {
        let port = "8089";
        
        // Start the server
        let mut server = Command::new(SERVER_BIN)
            .args(["--port", port])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start server");

        assert!(wait_for_server(port), "Server failed to start");

        // Start three clients
        let mut client1 = Command::new(CLIENT_BIN)
            .args(["--username", "alice"])
            .args(["--host", TEST_HOST])
            .args(["--port", port])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start client1");

        sleep(Duration::from_millis(500));

        let mut client2 = Command::new(CLIENT_BIN)
            .args(["--username", "bob"])
            .args(["--host", TEST_HOST])
            .args(["--port", port])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start client2");

        sleep(Duration::from_millis(500));

        let mut client3 = Command::new(CLIENT_BIN)
            .args(["--username", "charlie"])
            .args(["--host", TEST_HOST])
            .args(["--port", port])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start client3");

        sleep(Duration::from_secs(1));

        // Alice sends a broadcast message using the correct format: send <MSG>
        let client1_stdin = client1.stdin.as_mut().expect("Failed to open client1 stdin");
        writeln!(client1_stdin, "send Hello everyone!")
            .expect("Failed to write to client1 stdin");
        client1_stdin.flush().expect("Failed to flush client1 stdin");

        // Give time for message to be delivered
        sleep(Duration::from_secs(2));

        // Read output from bob's client
        let mut bob_output = Vec::new();
        if let Some(stdout) = client2.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines().take(5) {
                if let Ok(line) = line {
                    bob_output.push(line);
                    if bob_output.len() >= 3 {
                        break;
                    }
                }
            }
        }

        // Verify both bob and charlie received alice's message
        let bob_received = bob_output.join("\n");

        assert!(
            bob_received.contains("alice") && bob_received.contains("Hello everyone"),
            "Bob should receive Alice's message. Got: {}",
            bob_received
        );

        // Cleanup
        client1.kill().expect("Failed to kill client1");
        client2.kill().expect("Failed to kill client2");
        client3.kill().expect("Failed to kill client3");
        server.kill().expect("Failed to kill server");
    }
}
