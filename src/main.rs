use reqwest::{blocking::Client, blocking::ClientBuilder, header};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::process::{Command, Stdio};
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

#[derive(Serialize, Deserialize)]
struct Body {
    input: String,
}

#[derive(Serialize, Deserialize)]
struct SpellbookResponse {
    text: String,
}

fn main() -> io::Result<()> {
    println!("Hello, world!");

    let filename = "test.js";

    let mut file_contents = String::new();
    File::open(filename)
        .unwrap()
        .read_to_string(&mut file_contents)
        .expect("could not read the file to string :(((");

    let mut node_process = Command::new("node")
        .arg(filename)
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start node");
    let reader = BufReader::new(
        node_process
            .stderr
            .take()
            .expect("failed to capture stderr"),
    );
    let error_string = reader
        .lines()
        .fold(String::new(), |acc, el| acc + el.unwrap().as_str() + "\n");

    let request_string = format!(
        "Original File: 
```js
{file_contents}
```

Error: 
```
{error_string}
```"
    );

    let mut headers = header::HeaderMap::new();
    headers.append("Content-Type", "application/json".parse().unwrap());
    headers.append(
        "Authorization",
            std::env::var("SPELLBOOK_AUTHORIZATION_HEADER").unwrap().parse().unwrap(),
    );

    let client = ClientBuilder::new()
        .default_headers(headers)
        .build()
        .unwrap();
    let result = client
        .post("https://dashboard.scale.com/spellbook/api/app/o7n22jb")
        .body(
            serde_json::to_string(&Body {
                input: request_string,
            })
            .unwrap(),
        )
        .send()
        .unwrap()
        .json::<SpellbookResponse>()
        .unwrap();

    println!("{}", result.text);


    Ok(())
}
