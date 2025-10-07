use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter};

use bincode;
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;

use tfhe::prelude::*;
use tfhe::{
    generate_keys, set_server_key, ConfigBuilder, FheUint32, ServerKey, ClientKey,
};

const DATA_DIR: &str = "data";
const SERVER_KEY_PATH: &str = "data/server_key.bin";
const CLIENT_KEY_PATH: &str = "data/client_key.bin";
const CT_A_PATH: &str = "data/ct_a.bin";
const CT_B_PATH: &str = "data/ct_b.bin";
const SUM_PATH: &str = "data/sum.bin";

const SAVE_DEFAULT: bool = true;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let save_flag = read_save_flag();
    let output_mode = std::env::var("OUTPUT_MODE").unwrap_or_else(|_| "base64".to_string());

    if save_flag {
        eprintln!("SAVE=true: generating keys & ciphertexts (123, 456), verifying 579, saving all files.");
        gen_and_save()?;
        eprintln!("Saved: {SERVER_KEY_PATH}, {CLIENT_KEY_PATH}, {CT_A_PATH}, {CT_B_PATH}");
    } else {
        eprintln!("SAVE=false: loading keys/ciphertexts, performing addition, verifying 579, output in '{output_mode}' mode.");
        output_sum(output_mode)?;
    }

    Ok(())
}

fn read_save_flag() -> bool {
    match std::env::var("SAVE") {
        Ok(v) => {
            let v = v.to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "t" | "yes" | "y")
        }
        Err(_) => SAVE_DEFAULT,
    }
}

fn gen_and_save() -> Result<(), Box<dyn std::error::Error>> {
	let config = ConfigBuilder::default().build();
    //let config = ConfigBuilder::default().use_custom_parameters(MB_PARAMS.with_deterministic_execution()).build();
	
	let (client_key, server_key) = generate_keys(config);

    let m1: u32 = 123;
    let m2: u32 = 456;

    let ct_a = FheUint32::encrypt(m1, &client_key);
    let ct_b = FheUint32::encrypt(m2, &client_key);

    // Verify locally
    set_server_key(server_key.clone());
    let ct_sum = &ct_a + &ct_b;
    let plain_sum: u32 = ct_sum.decrypt(&client_key);
    eprintln!(
        "Local verification: expected 579, got {} -> {}",
        plain_sum,
        if plain_sum == 579 { "OK" } else { "FAILED" }
    );

    // Save everything
    create_dir_all(DATA_DIR)?;
    bincode::serialize_into(BufWriter::new(File::create(SERVER_KEY_PATH)?), &server_key)?;
    bincode::serialize_into(BufWriter::new(File::create(CLIENT_KEY_PATH)?), &client_key)?;
    bincode::serialize_into(BufWriter::new(File::create(CT_A_PATH)?), &ct_a)?;
    bincode::serialize_into(BufWriter::new(File::create(CT_B_PATH)?), &ct_b)?;
    Ok(())
}

fn output_sum(output_mode: String) -> Result<(), Box<dyn std::error::Error>> {
    let server_key: ServerKey =
        bincode::deserialize_from(BufReader::new(File::open(SERVER_KEY_PATH)?))?;
    let client_key: ClientKey =
        bincode::deserialize_from(BufReader::new(File::open(CLIENT_KEY_PATH)?))?;
    let ct_a: FheUint32 =
        bincode::deserialize_from(BufReader::new(File::open(CT_A_PATH)?))?;
    let ct_b: FheUint32 =
        bincode::deserialize_from(BufReader::new(File::open(CT_B_PATH)?))?;

    set_server_key(server_key);

    let sum = ct_a + ct_b;

    // Verify correctness again
    let plain_sum: u32 = sum.decrypt(&client_key);
    eprintln!(
        "Reload verification: expected 579, got {} -> {}",
        plain_sum,
        if plain_sum == 579 { "OK" } else { "FAILED" }
    );

    // Save result
    create_dir_all(DATA_DIR)?;
    bincode::serialize_into(BufWriter::new(File::create(SUM_PATH)?), &sum)?;

    match output_mode.as_str() {
        "int64" => {
            // Convert ciphertext to raw bytes, then print as i64
	
			let (radix_ct, _degree, _carry_modulus, _noises) = sum.into_raw_parts();
			println!("FheUint32 has {} blocks", radix_ct.blocks.len());
			for (i, block) in radix_ct.blocks.iter().enumerate() {

				let lwe = &block.ct;
				let mask = lwe.get_mask();
				let body = lwe.get_body();
				println!("--- block #{i} ---");
				println!("A = {:?}", mask.as_ref());
				println!("b        = {:?}", body.data);
		
			}
		}
        _ => {
            let bytes = bincode::serialize(&sum)?;
            let b64 = B64.encode(bytes);
            println!("{b64}");
        }
    }

    Ok(())
}

