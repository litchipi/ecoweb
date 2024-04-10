use base64::{engine::general_purpose::STANDARD as Base64, Engine};
use rsa::pkcs1::DecodeRsaPrivateKey;
use std::{collections::HashMap, path::PathBuf};

#[allow(unused)]
pub fn decode_mailed_data(data: String, private_key_file: PathBuf) -> HashMap<String, String> {
    let privk_data = std::fs::read_to_string(private_key_file)
        .expect("Unable to read privk file")
        .trim()
        .to_string();
    let privk_data = Base64
        .decode(privk_data)
        .expect("Unable to decode private key base64");
    let privk =
        rsa::RsaPrivateKey::from_pkcs1_der(&privk_data).expect("Error while importing private key");

    let mut decrypted = vec![];
    for block in data.split("-----") {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }
        let bind = Base64.decode(block).expect("Error decoding data base64");
        decrypted.extend(
            privk
                .decrypt(rsa::Oaep::new::<sha2::Sha256>(), &bind)
                .expect("Error while decrypting data"),
        );
    }

    bincode::deserialize(&decrypted).expect("Error while deserializing binary data")
}
