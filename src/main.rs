use std::{fs::File, io::{Error, Read}};
use sha1::{digest::generic_array::functional::FunctionalSequence, Digest, Sha1};

fn main() -> Result<(), Error> {
    let mut h = Sha1::new();
    h.update(b"00");
    println!("{}", h.finalize().map(|b| format!("{b:02x}")).join(""));


    let mut f = File::open("C:\\a.txt")?;
    let mut b =  [0u8; 256 * 1024];
    let mut oh = Sha1::new();
    loop {
        let l = f.read(&mut b)?;
        if l == 0 { break; }
        oh.update(&b[0..l]);
    }
    println!("{}", oh.finalize().map(|b| format!("{b:02x}")).join(""));
    Ok(())
}
