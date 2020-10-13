use std::convert::TryInto;
use std::env;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;
use std::str;
use std::thread;

use byteorder::{ByteOrder, NativeEndian};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(target_family = "unix")]
use std::os::unix::net::UnixStream as SocketStream;
#[cfg(target_family = "windows")]
use windows_named_pipe::PipeStream as SocketStream;

const DELIMITER: [u8; 1] = [12]; // node-ipc "\f"

fn get_socket_path() -> String {
    let exe_path = env::current_exe().unwrap();
    let exe_path = Path::new(&exe_path);
    // Cargo.toml name
    let filename = exe_path.file_stem().unwrap();
    let filename = filename.to_str().unwrap();

    if cfg!(target_family = "unix") {
        format!("/tmp/app.{}", filename)
    } else {
        format!("\\\\.\\pipe\\tmp-app.{}", filename)
    }
}

fn log(buf: &[u8]) {
    let mut path = env::current_exe().unwrap();
    path.set_file_name(".log");
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .expect("Couldn't open log file");
    file.write_all(format!("{:?}\n", buf).as_bytes()).unwrap();
    file.write_all(buf).unwrap();
    file.write_all(b"\n").unwrap();
}

#[derive(Serialize, Deserialize)]
struct JsMessage<'a> {
    r#type: &'a str,
    data: Value,
}

fn read_stdin_pipe_to(ipc_writable: &mut SocketStream) {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    let mut disconnect = false;
    let mut len_buf = [0; 4];
    let mut content = vec![0; 1024 * 1024]; // webextension limit 1M
    if cfg!(debug_assertions) {
        let msg = br#"{"type":"app.message","data":"rust debug"}"#;
        ipc_writable.write_all(msg).unwrap();
        log(msg);
    }

    while !disconnect {
        stdin.read_exact(&mut len_buf).unwrap();
        let len = NativeEndian::read_u32(&len_buf).try_into().unwrap();
        if len == 0 {
            disconnect = true;
        } else {
            stdin.read_exact(&mut content[..len]).unwrap();
            let js_msg = [&content[..len], &DELIMITER].concat();
            if cfg!(debug_assertions) {
                log(&js_msg);
            }
            ipc_writable.write_all(&js_msg).unwrap();
        }
    }
}

fn write_stdout(buf: &[u8]) {
    let mut len_buf = [0; 4];
    NativeEndian::write_u32(&mut len_buf, buf.len().try_into().unwrap());
    let content = [&len_buf, buf].concat();
    // _log(&content);
    io::stdout().write_all(&content).unwrap();
    io::stdout().flush().unwrap();
}

fn read_ipc_pipe_to(ipc_readable: &mut SocketStream) {
    let mut disconnect = false;
    let mut stream = BufReader::new(ipc_readable);
    while !disconnect {
        let mut buf = vec![];
        let len = stream.read_until(DELIMITER[0], &mut buf).unwrap();
        if len == 0 {
            disconnect = true;
        } else {
            write_stdout(&buf[..buf.len() - 1]);
        }
    }
}

fn notify() {
    let msg: JsMessage = JsMessage {
        r#type: "connected",
        data: Value::String(String::new()),
    };
    write_stdout(serde_json::to_string(&msg).unwrap().as_bytes());
}

fn create_socket() -> (SocketStream, SocketStream) {
    let writable = SocketStream::connect(get_socket_path()).expect("Couldn't connect socket");
    let readable = SocketStream::connect(get_socket_path()).expect("Couldn't connect socket");
    (writable, readable)
}

fn main() {
    let (mut ipc_writable, mut ipc_readable) = create_socket();

    notify();

    let thr = thread::spawn(move || read_stdin_pipe_to(&mut ipc_writable));
    let thr2 = thread::spawn(move || read_ipc_pipe_to(&mut ipc_readable));
    thr.join().expect("read stdin thread failed");
    thr2.join().expect("read ipc thread failed");
}
