use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use rand::random;
use reqwest::{Client, ClientBuilder, header};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::os::unix::prelude::*;
use std::process::{Command, Stdio};
use std::{
    env,
    fs::{self, File},
    io::{self, BufRead, BufReader},
};
use tokio::main;

#[derive(Serialize, Deserialize)]
struct Body {
    input: String,
}

#[derive(Serialize, Deserialize)]
struct SpellbookResponse {
    text: String,
}

//     let filename = env::args().last().unwrap_or("test.js".to_string());
//     let mut file_contents = String::new();
//     File::open(&filename)
//         .unwrap()
//         .read_to_string(&mut file_contents)
//         .expect("could not read the file to string :(((");

//     let result = improve_your_code(file_contents).unwrap();s
//     fs::write(filename + ".new", result).expect("Unable to write file");

//     Ok(())
// }
//     File::open(&filename)
//         .unwrap()
//         .read_to_string(&mut file_contents)
//         .expect("could not read the file to string :(((");

//     let result = improve_your_code(file_contents).unwrap();s
//     fs::write(filename + ".new", result).expect("Unable to write file");

//     Ok(())
// }

#[tokio::main]
async fn main() {
    use std::str::FromStr;
    let port = std::env::var("PORT")
        .ok()
        .and_then(|x| u16::from_str(&x).ok())
        .unwrap_or(8080);
    println!("Listening on port {port}");

    // build our application with a route
    let app = Router::new()
        // `POST /users` goes to `create_user`
        .route("/fix", post(fix_code_endpoint));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FixCodeRequest {
    code_js: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FixCodeResponse {
    code_js: String,
}

async fn fix_code_endpoint(
    Json(body): Json<FixCodeRequest>,
) -> Result<Json<FixCodeResponse>, StatusCode> {
    match improve_your_code(body.code_js).await {
        Ok(code_js) => Ok(Json(FixCodeResponse { code_js })),
        Err(e) => {
            println!("error: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn improve_your_code(file_contents: String) -> Result<String, io::Error> {
    let temp_dir = std::env::temp_dir();
    let filename = format!("file_{}.js", random::<usize>());
    let filename = temp_dir.join(filename);
    let mut tempfile = std::fs::File::create(filename.as_path())?;
    tempfile.write(file_contents.as_bytes())?;

    let mut node_process = Command::new("node")
        .arg(filename)
        .stderr(Stdio::piped())
        .stdout(Stdio::null())
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
    if error_string.len() == 0 {
        return Ok(error_string);
    }
    let request_string = format!(
        "Original File: 
```js
{file_contents}
```

Error: 
```
{error_string}
```
"
    );
    let mut headers = header::HeaderMap::new();
    headers.append("Content-Type", "application/json".parse().unwrap());
    headers.append(
        "Authorization",
        std::env::var("SPELLBOOK_AUTHORIZATION_HEADER")
            .unwrap()
            .parse()
            .unwrap(),
    );
    let client = ClientBuilder::new()
        .default_headers(headers)
        .build()
        .unwrap();
    let result = client
        .post("https://dashboard.scale.com/spellbook/api/app/4q6k2235")
        .body(
            serde_json::to_string(&Body {
                input: request_string,
            })
            .unwrap(),
        )
        .send()
        .await
        .unwrap()
        .json::<SpellbookResponse>()
        .await
        .unwrap();
    let result = result.text.trim();
    let (_js_ticks, result) = result.split_once('\n').unwrap();
    let (result, _ticks) = result.rsplit_once('\n').unwrap();

    Ok(result.into())
}
