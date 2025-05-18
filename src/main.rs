use sha1::{
    Digest, Sha1, Sha1Core,
    digest::{core_api::CoreWrapper, generic_array::functional::FunctionalSequence},
};
use std::{
    fs::File,
    io::{Error, Read, Write},
};

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
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.inner
            .read(buf)
            .inspect(|&len| self.hasher.update(&buf[0..len]))
    }
}
struct Sha1Writer {
    hasher: CoreWrapper<Sha1Core>,
}
impl Sha1Writer {
    fn new() -> Self {
        Self {
            hasher: Sha1::new(),
        }
    }
}
impl ToHexString for Sha1Writer {
    fn to_hex_string(self) -> String {
        self.hasher.to_hex_string()
    }
}
impl Write for Sha1Writer {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.hasher.update(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let mut r = lz4_flex::frame::FrameDecoder::new(Sha1Reader::new(File::open(
        r"C:\Users\kyle\source\repos\rust\sha1outinlz4\a.txt.lz4",
    )?));
    let mut w = Sha1Writer::new();
    std::io::copy(&mut r, &mut w)?;
    println!("{},{}", r.into_inner().to_hex_string(), w.to_hex_string());
    Ok(())
}
//cargo r; wsl bash -c ' lz4 -c a.txt.lz4 | sha1sum a.txt.lz4 - | cut -d\  -f 1 | paste -sd,'
