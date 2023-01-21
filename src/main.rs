use std::process::{Command, Stdio};
use std::{
    fs::File,
    io::{self, BufReader, BufRead},
};

fn main() -> io::Result<()> {
    println!("Hello, world!");

    let mut node_process = Command::new("node")
        .arg("test.js")
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start node");
    let reader = BufReader::new(node_process.stderr.take().expect("failed to capture stdout"));
    for line in reader.lines() {
        println!("ERROR: {}",line?);
    }
    Ok(())

}
