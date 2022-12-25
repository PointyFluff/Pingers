use std::io::{BufRead, BufReader, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

// spawn me
fn handle_client(stream: TcpStream) {
    let mut read_stream = stream.try_clone().unwrap();
    let mut write_stream = stream.try_clone().unwrap();
    let reader = BufReader::new(&mut read_stream);
    let lines = reader.lines();

    let mut client_msg = |msg: &str| {
        let mut buffer: Vec<u8> = Vec::new();
        let suf: &str;
        if msg == "GOODBYE" {
            suf = "\n\n";
        } else {
            suf = "\n\nPINGERS> ";
        }
        write!(buffer, "{msg} {suf}").unwrap();
        write_stream.write_all(&buffer).unwrap();
        write_stream.flush().unwrap();
    };

    client_msg("");

    for line in lines {
        match line {
            Ok(command) => match command.to_ascii_lowercase().as_str().trim() {
                "ping" => {
                    println!("Recieved a \"ping\" command.");
                    client_msg("PONG");
                }
                "pong" => {
                    println!("Recieved a \"pong\" command.");
                    client_msg("PING");
                }
                "help" => {
                    println!("Recieved a \"help\" command.");
                    client_msg("PING, PONG, WHOAMI, TIME\nTEMPLATE, EXIT, QUIT, HELP");
                }
                "whoami" => {
                    println!("Recieved a \"whoami\" command.");
                    let peer_addr = stream.peer_addr().unwrap().to_string();
                    let local_addr = stream.local_addr().unwrap().to_string();
                    let msg = format!("You: {peer_addr}\nMe: {local_addr}");
                    client_msg(msg.as_str());
                }
                "time" => {
                    println!("Recieved a \"delay\" command.");
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap();
                    // 31557600 @ year
                    // 86400 @ day
                    // 3600 @ hour
                    let msg = format!("1970-01-01 00:00:00 UTC was {} seconds ago!", now.as_secs());
                    client_msg(msg.as_str());
                }
                "template" => {
                    println!("Recieved a \"template\" command.");
                    client_msg("RESPONSE");
                }
                "exit" | "quit" => {
                    println!("Stream close requested");
                    client_msg("GOODBYE");
                    stream.shutdown(Shutdown::Both).unwrap();
                }
                _ => {
                    println!("Recieved UNKNOWN command.");
                    client_msg("???");
                }
            },
            Err(e) => println!("Line error: {}", e),
        }
    }
}

fn main() -> std::io::Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    let arg = |name: &str| {
        args.windows(2)
            .find(|a| a[0] == name)
            .and_then(|a| a[1].to_str())
    };

    let addr = arg("--bind");
    let addr = match addr {
        Some(a) => a,
        None => {
            println!("Usage: pingers --bind address:port");
            "0.0.0.0:9999"
        }
    };

    println!("Binding to: {addr:?}");
    let listener = TcpListener::bind(addr)?;

    for client in listener.incoming() {
        match client {
            Ok(client) => {
                thread::spawn(move || handle_client(client));
            }
            Err(e) => println!("Connection failed {e:?}"),
        }
    }
    Ok(())
}
