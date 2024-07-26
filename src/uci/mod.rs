use std::io;

pub fn uci_main() {
    let mut command = String::new();

    io::stdin()
        .read_line(&mut command)
        .expect("Failed to read line");

    println!("you said {}", command);
}