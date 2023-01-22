use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use rand::random;
use reqwest::{header, Client, ClientBuilder};
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
use tower_http::cors::{self, Any};

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

    let cors = cors::CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([http::Method::GET, http::Method::POST, http::Method::OPTIONS])
        .allow_headers([http::header::CONTENT_TYPE])
        // allow requests from any origin
        .allow_origin(Any);
    // build our application with a route
    let app = Router::new()
        // `POST /users` goes to `create_user`
        .route("/fix", post(fix_code_endpoint))
        .route("/run", post(run_code_endpoint))
        .layer(cors);

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
    output: String
}

async fn fix_code_endpoint(
    Json(body): Json<FixCodeRequest>,
) -> Result<Json<FixCodeResponse>, StatusCode> {
    let foo = async {
        let code_js = improve_your_code(body.code_js).await?;
        let fixed_filename = write_code_to_file(code_js.as_str())?;
        let post_fixing_output = run_code(fixed_filename.as_path());

        Ok((code_js, post_fixing_output))
    };
    let foo: Result<_, io::Error> = foo.await;
    match foo {
        Ok((code_js, post_fixing_output)) => Ok(Json(FixCodeResponse { code_js, output: post_fixing_output })),
        Err(e) => {
            println!("error: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RanCodeResponse {
    output: String
}

async fn run_code_endpoint(
    Json(FixCodeRequest { code_js }): Json<FixCodeRequest>
) -> Result<Json<RanCodeResponse>, StatusCode> {
    let foo = || -> Result<String, io::Error> {
        let filename = write_code_to_file(code_js.as_str())?;
        Ok(run_code(filename.as_path()))
    };
    match foo() {
        Ok(string) => Ok(Json(RanCodeResponse { output: string })),
        Err(e) => {
            dbg!(e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn improve_your_code(file_contents: String) -> Result<String, io::Error> {
    let filename = write_code_to_file(file_contents.as_str())?;

    let error_string = run_code(filename.as_path());
    if error_string.len() == 0 {
        return Ok(file_contents);
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

fn run_code(filename: &std::path::Path) -> String {
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
    error_string
}

/// Returns the filename
fn write_code_to_file(file_contents: &str) -> Result<std::path::PathBuf, io::Error> {
    let temp_dir = std::env::temp_dir();
    let filename = format!("file_{}.js", random::<usize>());
    let filename = temp_dir.join(filename);
    let mut tempfile = std::fs::File::create(filename.as_path())?;
    tempfile.write(file_contents.as_bytes())?;
    Ok(filename)
}
