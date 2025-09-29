use std::convert::TryFrom;
use std::io::{self, BufRead, BufReader, Write, Read};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, Duration};

use aes::{Aes128, Aes192, Aes256};
use ctr::cipher::{NewCipher, StreamCipher};
use hkdf::Hkdf;
use sha2::Sha256;
use x25519_dalek::{StaticSecret, PublicKey};
use hex;
use rand::rngs::OsRng;
use rand::RngCore;
use signal_hook::iterator::Signals;

/// Derive an AES key from the shared secret using HKDF with SHA-256.
fn derive_aes_key(shared: &[u8], key_len: usize) -> Vec<u8> {
    let hk = Hkdf::<Sha256>::new(None, shared);
    let mut okm = vec![0u8; key_len];
    hk.expand(b"aes key", &mut okm).expect("HKDF expand failed");
    okm
}

/// Runs the entire key exchange and streaming procedure.
/// When it returns—due to an error or normal termination—the caller can restart the session.
fn run_streaming() -> io::Result<()> {
    // Connect to the PC server.
    let server_address = "192.168.1.3:5000"; // adjust as needed
    let mut stream = TcpStream::connect(server_address)
        .expect("Failed to connect to server");
    // Disable Nagle's algorithm for low latency.
    stream.set_nodelay(true)?;

    // --- Key Exchange Phase ---
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    // Expected format: "<key_size> <server_public_key_hex>"
    let mut parts = line.trim().split_whitespace();
    let key_size: usize = parts.next().expect("Missing key size")
        .parse().expect("Invalid key size");
    if key_size != 128 && key_size != 192 && key_size != 256 {
        panic!("Server key size must be 128, 192, or 256");
    }
    let server_pub_hex = parts.next().expect("Missing server public key");
    let server_pub_bytes = hex::decode(server_pub_hex)
        .expect("Failed to decode server public key");
    let server_public = PublicKey::from(
        <[u8; 32]>::try_from(&server_pub_bytes[..]).expect("Invalid server public key length")
    );

    // Generate client's X25519 key pair.
    let client_private = StaticSecret::new(OsRng);
    let client_public = PublicKey::from(&client_private);

    // Send client's public key (hex-encoded) to the server.
    let client_pub_hex = hex::encode(client_public.as_bytes());
    stream.write_all(format!("{}\n", client_pub_hex).as_bytes())?;

    // Compute shared secret and derive AES key.
    let shared_secret = client_private.diffie_hellman(&server_public);
    let shared_bytes = shared_secret.as_bytes();
    let aes_key = derive_aes_key(shared_bytes, key_size / 8);
    println!("Derived AES key (hex): {}", hex::encode(&aes_key));

    // --- Improved Nonce Generation ---
    let mut random_part = [0u8; 8];
    OsRng.fill_bytes(&mut random_part);
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let timestamp_bytes = timestamp.to_le_bytes();
    let mut nonce = [0u8; 16];
    nonce[..8].copy_from_slice(&random_part);
    nonce[8..].copy_from_slice(&timestamp_bytes);
    let nonce_hex = hex::encode(&nonce);
    stream.write_all(format!("{}\n", nonce_hex).as_bytes())?;

    // --- Video Capture & Encryption Phase ---
    // Spawn libcamera-vid to capture video.
    let mut cam = Command::new("libcamera-vid")
        .args(&[
            "-t", "0",                // run indefinitely
            "--width", "1280",
            "--height", "720",
            "--framerate", "30",
            "--codec", "h264",
            "--profile", "baseline",
            "--inline",
            "--libav-format", "mpegts", // use MPEG-TS container
            "-o", "-"                 // output to stdout
        ])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn libcamera-vid");
    let mut cam_stdout = cam.stdout.take().expect("Failed to capture stdout from libcamera-vid");

    // Set up signal handling: when a SIGINT or SIGTERM is received, exit immediately.
    let mut signals = Signals::new(&[signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM])?;
    std::thread::spawn(move || {
        for sig in signals.forever() {
            eprintln!("Received signal: {:?}. Exiting immediately.", sig);
            std::process::exit(0);
        }
    });

    // Use an atomic flag for the streaming loop (if needed for graceful shutdown in future).
    let running = Arc::new(AtomicBool::new(true));

    // Set up AES-CTR encryption.
    match key_size {
        128 => {
            use ctr::Ctr128BE;
            type Aes128Ctr = Ctr128BE<Aes128>;
            let mut cipher = Aes128Ctr::new_from_slices(&aes_key, &nonce)
                .expect("Error creating AES-128-CTR cipher");
            let mut buf = [0u8; 4096];
            while running.load(Ordering::SeqCst) {
                let n = cam_stdout.read(&mut buf)?;
                if n == 0 { break; }
                let mut data = buf[..n].to_vec();
                cipher.apply_keystream(&mut data);
                stream.write_all(&data)?;
            }
        },
        192 => {
            use ctr::Ctr128BE;
            type Aes192Ctr = Ctr128BE<Aes192>;
            let mut cipher = Aes192Ctr::new_from_slices(&aes_key, &nonce)
                .expect("Error creating AES-192-CTR cipher");
            let mut buf = [0u8; 4096];
            while running.load(Ordering::SeqCst) {
                let n = cam_stdout.read(&mut buf)?;
                if n == 0 { break; }
                let mut data = buf[..n].to_vec();
                cipher.apply_keystream(&mut data);
                stream.write_all(&data)?;
            }
        },
        256 => {
            use ctr::Ctr128BE;
            type Aes256Ctr = Ctr128BE<Aes256>;
            let mut cipher = Aes256Ctr::new_from_slices(&aes_key, &nonce)
                .expect("Error creating AES-256-CTR cipher");
            let mut buf = [0u8; 4096];
            while running.load(Ordering::SeqCst) {
                let n = cam_stdout.read(&mut buf)?;
                if n == 0 { break; }
                let mut data = buf[..n].to_vec();
                cipher.apply_keystream(&mut data);
                stream.write_all(&data)?;
            }
        },
        _ => unreachable!(),
    }

    // Cleanup: terminate libcamera-vid if still running.
    let _ = cam.kill();
    println!("Streaming session ended. Returning to wait for new connection.");
    Ok(())
}

fn main() -> io::Result<()> {
    loop {
        println!("Starting new streaming session...");
        match run_streaming() {
            Ok(_) => println!("Session ended normally. Restarting..."),
            Err(e) => println!("Session ended with error: {}. Restarting...", e),
        }
        // Wait briefly before restarting.
        std::thread::sleep(Duration::from_secs(1));
    }
}
