use clap::Parser;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use bytesize::ByteSize;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Address of the host/redirector
    #[arg()]
    redirector: String,
    
    /// Directory to download the file to.
    /// 
    /// Has to be a directory!
    #[arg(short, long, default_value_t = String::from("./"))]
    directory: String,
}


#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    // parse args
    let args = Args::parse();
    
    // prepare download directory
    let dir = std::path::PathBuf::from(args.directory);
    tokio::fs::create_dir_all(&dir).await?;
    
    // connect
    let mut conn = tokio::net::TcpStream::connect(&args.redirector).await?;
    println!("Connected to {}!", args.redirector);
    
    // check for resume file
    let mut rd_res = tokio::fs::OpenOptions::new()
        .create(true)
        .append(false)
        .truncate(false)
        .write(true)
        .read(true)
        .open(dir.join(".rdres")).await?;
    // send amount
    let mut resume_amt = rd_res.read_u64().await.unwrap_or(0);
    conn.write_u64(resume_amt).await?;
    
    // receive total size
    let total_size = (conn.read_u64().await? + resume_amt).max(1);
    println!("Total File Size: {}", ByteSize(total_size));
    
    // receive name
    let strlen = conn.read_u16().await?;
    let mut buf = vec![0u8; strlen as usize];
    conn.read_exact(&mut buf).await?;
    let name = String::from_utf8(buf).expect("Invalid utf-8 name sent!");
    println!("Filename received: {name}");
    
    // open file
    let file = tokio::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(dir.join(name))
        .await?;
    let mut file = tokio::io::BufWriter::new(file);

    // buffer io
    let mut reader = tokio::io::BufReader::new(conn);
    let mut rd_res = tokio::io::BufWriter::new(rd_res);
    
    // receive bytes
    let start = std::time::Instant::now();
    loop {
        let len = {
            // receive bytes
            let bytes = reader.fill_buf().await?;
            if bytes.is_empty() {
                break
            }
            
            // write bytes
            file.write_all(bytes).await?;
            
            bytes.len()
        };
        
        // progress reader
        reader.consume(len);
        
        // and resume file
        resume_amt += len as u64;
        print!("\r{} ({}%) Bytes read ({}/s)     ", ByteSize(resume_amt), resume_amt * 100 / total_size, ByteSize((resume_amt as f64 / start.elapsed().as_secs_f64()) as u64));
        rd_res.seek(std::io::SeekFrom::Start(0)).await?;
        rd_res.write_u64(resume_amt).await?;
    }
    
    // remove resume file
    tokio::fs::remove_file(dir.join(".rdres")).await?;
    
    // final print
    println!("\nDownload finished in {:?}", start.elapsed());
    Ok(())
}