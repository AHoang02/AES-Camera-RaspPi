use std::env;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

use aes::{Aes128, Aes192, Aes256};
use ctr::cipher::{NewCipher, StreamCipher};
use hkdf::Hkdf;
use sha2::Sha256;
use x25519_dalek::{StaticSecret, PublicKey};
use hex;
use rand::rngs::OsRng;

fn derive_aes_key(shared: &[u8], key_len: usize) -> Vec<u8> {
    let hk = Hkdf::<Sha256>::new(None, shared);
    let mut okm = vec![0u8; key_len];
    hk.expand(b"aes key", &mut okm).expect("HKDF expand failed");
    okm
}

fn main() -> io::Result<()> {
    // --- Command‐Line Argument for Key Size ---
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <key_size>", args[0]);
        std::process::exit(1);
    }
    let key_size: usize = args[1].parse().expect("Invalid key size");
    if key_size != 128 && key_size != 192 && key_size != 256 {
        eprintln!("Key size must be 128, 192, or 256.");
        std::process::exit(1);
    }
    eprintln!("Using AES key size: {}", key_size);

    // --- Key Exchange Phase ---
    let listener = TcpListener::bind("0.0.0.0:5000")?;
    eprintln!("Server listening on port 5000... Waiting for connection.");
    let (mut stream, addr) = listener.accept()?;
    eprintln!("Connection established from {}", addr);

    // Send key size + server public key
    let server_private = StaticSecret::new(OsRng);
    let server_public = PublicKey::from(&server_private);
    let server_pub_hex = hex::encode(server_public.as_bytes());
    stream.write_all(format!("{} {}\n", key_size, server_pub_hex).as_bytes())?;

    // Receive client public key
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut client_pub_hex = String::new();
    reader.read_line(&mut client_pub_hex)?;
    let client_pub_bytes = hex::decode(client_pub_hex.trim()).expect("Failed to decode client public key");
    let client_public = PublicKey::from(
        <[u8; 32]>::try_from(&client_pub_bytes[..]).expect("Invalid client public key length")
    );

    // Derive shared secret & AES key
    let shared_secret = server_private.diffie_hellman(&client_public);
    let aes_key = derive_aes_key(shared_secret.as_bytes(), key_size / 8);
    eprintln!("Derived AES key (hex): {}", hex::encode(&aes_key));

    // --- Nonce Reception ---
    let mut nonce_hex = String::new();
    reader.read_line(&mut nonce_hex)?;
    let nonce_bytes = hex::decode(nonce_hex.trim()).expect("Failed to decode nonce");

    // --- Decrypt & Forward MPEG-TS ---
    stream.set_nodelay(true)?;
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut last_log = Instant::now();

    match key_size {
        128 => {
            use ctr::Ctr128BE;
            type AesCtr = Ctr128BE<Aes128>;
            let mut cipher = AesCtr::new_from_slices(&aes_key, &nonce_bytes)
                .expect("Error creating AES-128-CTR cipher");
            let mut buf = [0u8; 4096];
            loop {
                let n = stream.read(&mut buf)?;
                if n == 0 { break; }
                if last_log.elapsed() >= Duration::from_secs(1) {
                    eprintln!("Encrypted data (first 32 bytes): {:02x?}", &buf[..n.min(32)]);
                    last_log = Instant::now();
                }
                let mut data = buf[..n].to_vec();
                cipher.apply_keystream(&mut data);
                out.write_all(&data)?;
                out.flush()?;
            }
        },
        192 => {
            use ctr::Ctr128BE;
            type AesCtr = Ctr128BE<Aes192>;
            let mut cipher = AesCtr::new_from_slices(&aes_key, &nonce_bytes)
                .expect("Error creating AES-192-CTR cipher");
            let mut buf = [0u8; 4096];
            loop {
                let n = stream.read(&mut buf)?;
                if n == 0 { break; }
                if last_log.elapsed() >= Duration::from_secs(1) {
                    eprintln!("Encrypted data (first 32 bytes): {:02x?}", &buf[..n.min(32)]);
                    last_log = Instant::now();
                }
                let mut data = buf[..n].to_vec();
                cipher.apply_keystream(&mut data);
                out.write_all(&data)?;
                out.flush()?;
            }
        },
        256 => {
            use ctr::Ctr128BE;
            type AesCtr = Ctr128BE<Aes256>;
            let mut cipher = AesCtr::new_from_slices(&aes_key, &nonce_bytes)
                .expect("Error creating AES-256-CTR cipher");
            let mut buf = [0u8; 4096];
            loop {
                let n = stream.read(&mut buf)?;
                if n == 0 { break; }
                if last_log.elapsed() >= Duration::from_secs(1) {
                    eprintln!("Encrypted data (first 32 bytes): {:02x?}", &buf[..n.min(32)]);
                    last_log = Instant::now();
                }
                let mut data = buf[..n].to_vec();
                cipher.apply_keystream(&mut data);
                out.write_all(&data)?;
                out.flush()?;
            }
        },
        _ => unreachable!(),
    }

    Ok(())
}
