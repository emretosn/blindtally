use tfhe::prelude::*;
use tfhe::{generate_keys, set_server_key, ConfigBuilder, FheUint8};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConfigBuilder::default().build();
    let (client_key, server_keys) = generate_keys(config);

    let clear_a: u8 = 7;
    let clear_b: u8 = 3;

    let encrypted_a = FheUint8::try_encrypt(clear_a, &client_key)?;
    let encrypted_b = FheUint8::try_encrypt(clear_b, &client_key)?;

    set_server_key(server_keys);

    let encrypted_sum = &encrypted_a + &encrypted_b;
    let encrypted_product = &encrypted_a * &encrypted_b;

    let decrypted_sum: u8 = encrypted_sum.decrypt(&client_key);
    let decrypted_product: u8 = encrypted_product.decrypt(&client_key);

    println!("7 + 3 = {} (computed while fully encrypted)", decrypted_sum);
    println!("7 * 3 = {} (computed while fully encrypted)", decrypted_product);

    Ok(())
}
