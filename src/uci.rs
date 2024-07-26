use std::io;

// read line from stdin and then should interpret it but doesn't do that yet
pub fn uci_main() {
    let mut command = String::new();

    io::stdin()
        .read_line(&mut command)
        .expect("Failed to read line");

    println!("you said {}", command);
}