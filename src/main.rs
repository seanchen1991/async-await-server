#![feature(async_await, await_macro, futures_api)]

use std::io::prelude::*;

use rand::Rng;

use futures::StreamExt;
use futures::executor::{self, ThreadPool};
use futures::io::AsyncWriteExt;
use futures::task::{SpawnExt};

use romio::{TcpListener, TcpStream};

fn main() -> io::Result<()> {
    executor::block_on(async {
        let mut threadpool = ThreadPool::new()?;

        let listener = TcpListener::bind(&"127.0.0.1:7878".parse().unwrap())?;
        let mut incoming = listener.incoming();

        println!("Listening on 127.0.0.1:7878");

        while let Some(stream) = await!(incoming.next()) {
            let stream = stream?;

            threadpool.spawn(async move {
                await!(handle_request(stream)).unwrap();
            }).unwrap();
        }

        Ok(())
    })
}

async fn handle_request(mut stream: TcpStream) -> io::Result<()> {
    let mut buffer = [0; 512];
    await!(stream.read(&mut buffer)?);

    let get_req = b"GET /";
    let d20_req = b"GET /d20";
    let post_req = b"POST /save";

    if buffer.starts_with(d20_req) {
        await!(get_d20(stream)?);
    } else if buffer.starts(get_req) {
        await!(get_file(stream, &mut buffer)?);
    } else if buffer.starts_with(post_req) {
        await!(post_save(stream, &mut buffer)?);
    } else {
        await!(get_404(stream)?);
    }

    Ok(())
}

async fn get_404(stream: TcpStream) -> io::Result<()> {
    let status_line = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
    let mut file = File::open("404.html")?;
    let mut contents = Stream::new();
    
    file.read_to_stream(&mut contents)?;

    await!(send_response(stream, status_line, &contents)?);

    Ok(())
}

async fn send_response(mut stream: TcpStream, status_line: &str, contents: &str) -> io::Result<()> {
    let response = format!("{}{}", status_line, contents);

    await!(stream.write(response.as_bytes())?);
    await!(stream.flush()?);

    Ok(())
}