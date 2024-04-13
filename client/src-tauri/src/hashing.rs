pub fn xor_encrypt(string: &str, key: i32) -> String {
    // Convert string to bytes
    let bytes_string = string.as_bytes();

    // Convert key to bytes
    let bytes_key = key.to_be_bytes();

    // Encrypt byte by byte using XOR
    let encrypted_bytes: Vec<u8> = bytes_string
        .iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ bytes_key[i as usize % bytes_key.len()])
        .collect();

    // Convert encrypted bytes back to string
    let encrypted_string = String::from_utf8(encrypted_bytes).unwrap();

    encrypted_string
}

pub fn modular_pow(b: u64, mut e: u64, m: u64) -> u64 {
    if m == 1 {
        return 0;
    }
    let mut r = 1u128;
    let mut b = (b % m) as u128;

    while e > 0 {
        if e % 2 == 1 {
            r = (r * b as u128) % m as u128;
        }

        e >>= 1;
        b = (b * b) % m as u128;
    }

    r as u64
}
