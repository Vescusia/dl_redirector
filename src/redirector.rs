use std::net::SocketAddr;
use bytesize::ByteSize;

use clap::Parser;
use futures_util::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// url to redirect
    #[arg()]
    url: String,

    /// Socket to bind to
    #[arg(short, long, default_value_t = { "0.0.0.0:4444".parse().unwrap() })]
    socket_addr: SocketAddr
}


#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // bind to socket
    let listener = tokio::net::TcpListener::bind(args.socket_addr).await?;
    println!("Server is listening!");

    // accept clients
    while let Ok((c_stream, c_socket)) = listener.accept().await {
        println!("Client {c_socket:?} arrived!");
        redirect(c_stream, &args.url).await?
    }

    Ok(())
}

async fn redirect(mut c_stream: tokio::net::TcpStream, url: &str) -> anyhow::Result<()> {
    // receive previously
    let rd_resume = c_stream.read_u64().await?;

    // connect to download site
    let client = reqwest::Client::new();
    let response = client.request(reqwest::Method::GET, url)
        .header("Range", format!("bytes={rd_resume}-"))
        .send().await?;
    // get total length
    let total_length = response.headers().get("Content-Length").map(|v| v.to_str().unwrap()).unwrap_or("0");
    let total_length: u64 = total_length.parse()?;
    // convert to stream
    let mut bytes_stream = response.bytes_stream();
    
    // send total size to client
    println!("Total length: {}", ByteSize(total_length));
    c_stream.write_u64(total_length).await?;
    
    // write file name
    let name = url.split('/').last().unwrap();
    c_stream.write_u16(name.len() as u16).await?;
    c_stream.write_all(name.as_bytes()).await?;

    // redirect bytes
    let mut total_sent = rd_resume as usize;
    let start = tokio::time::Instant::now();
    while let Some(Ok(bytes)) = bytes_stream.next().await {
        c_stream.write_all(&bytes).await?;

        // increment total sent
        total_sent += bytes.len();
        print!("\r{} Bytes sent ({}/s)     ", ByteSize(total_sent as u64), ByteSize((total_sent as f64 / start.elapsed().as_secs_f64()) as u64))
    }

    println!("\nFinished.");
    Ok(())
}
