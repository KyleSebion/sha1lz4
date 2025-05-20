use clap::Parser;
use clio::Input;
use sha1::{
    Digest, Sha1, Sha1Core,
    digest::{core_api::CoreWrapper, generic_array::functional::FunctionalSequence},
};
use std::io::{self, Read, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

#[derive(Parser)]
#[command(version)]
struct Opt {
    #[clap(short, long, value_parser)]
    input: Input,
}

trait ToHexString {
    fn to_hex_string(self) -> String;
}
impl ToHexString for CoreWrapper<Sha1Core> {
    fn to_hex_string(self) -> String {
        self.finalize().map(|b| format!("{b:02x}")).join("")
    }
}
struct Sha1Reader<R: Read> {
    inner: R,
    hasher: CoreWrapper<Sha1Core>,
}
impl<R: Read> Sha1Reader<R> {
    fn new(inner: R) -> Self {
        Self {
            inner,
            hasher: Sha1::new(),
        }
    }
}
impl<R: Read> ToHexString for Sha1Reader<R> {
    fn to_hex_string(self) -> String {
        self.hasher.to_hex_string()
    }
}
impl<R: Read> Read for Sha1Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner
            .read(buf)
            .inspect(|&len| self.hasher.update(&buf[0..len]))
    }
}
struct Sha1Writer {
    bytes_tx: Sender<Vec<u8>>,
    result_rx: Receiver<String>,
}
impl Sha1Writer {
    fn new() -> Self {
        let (bytes_tx, bytes_rx) = mpsc::channel();
        let (result_tx, result_rx) = mpsc::channel();
        thread::spawn(move || {
            let mut hasher = Sha1::new();
            loop {
                if let Ok(r) = bytes_rx.recv() {
                    hasher.update(r);
                } else {
                    // caused by drop(self.bytes_tx)
                    break;
                }
            }
            result_tx
                .send(hasher.to_hex_string())
                .expect("result_tx.send");
        });
        Self {
            bytes_tx,
            result_rx,
        }
    }
}
impl ToHexString for Sha1Writer {
    fn to_hex_string(self) -> String {
        drop(self.bytes_tx);
        self.result_rx.recv().expect("result_rx.recv")
    }
}
impl Write for Sha1Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.bytes_tx.send(buf.into()).expect("bytes_tx.send");
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn reader_to_hashes<R: Read>(r: R) -> io::Result<String> {
    let mut r = lz4_flex::frame::FrameDecoder::new(Sha1Reader::new(r));
    let mut w = Sha1Writer::new();
    io::copy(&mut r, &mut w)?;
    Ok(format!(
        "{},{}",
        r.into_inner().to_hex_string(),
        w.to_hex_string()
    ))
}
fn main() -> io::Result<()> {
    let opt = Opt::parse();
    let file = opt.input.path().to_string_lossy().into_owned();
    let hashes = reader_to_hashes(opt.input)?;
    println!("{hashes} {file}");
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn t1() {
        assert_eq!(
            "5e2ecff46cf39bfa09c9f1f40c8fe6636fb97359,fb96549631c835eb239cd614cc6b5cb7d295121a",
            reader_to_hashes(
                &[
                    0x04, 0x22, 0x4d, 0x18, 0x64, 0x40, 0xa7, 0x02, 0x00, 0x00, 0x80, 0x30, 0x30,
                    0x00, 0x00, 0x00, 0x00, 0xfb, 0x97, 0x0f, 0xa1
                ][..]
            )
            .unwrap()
        )
    }
}
