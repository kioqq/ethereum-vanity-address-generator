use tiny_keccak::keccak256;
use ethereum_types::Address as EthAddress;
use serde_json::{
    json,
    Value as JsonValue,
};
use crate::lib::{
    types::Result,
    errors::AppError,
    crypto_utils::generate_random_private_key,
    utils::{
        validate_hex,
        maybe_pad_hex,
        maybe_strip_hex_prefix,
        validate_prefix_hex_length
    },
};
use secp256k1::{
    Secp256k1,
    key::{
        SecretKey,
        PublicKey
    },
};

pub struct EthereumKeys {
    public_key: PublicKey,
    private_key: SecretKey,
    pub address: EthAddress,
    pub address_string: String,
}

impl EthereumKeys {
    fn validate_prefix_hex(prefix_hex: &str) -> Result<String> {
        maybe_strip_hex_prefix(prefix_hex)
            .map(|hex_no_prefix| maybe_pad_hex(&hex_no_prefix))
            .and_then(|padded_hex| validate_prefix_hex_length(&padded_hex))
            .and_then(|correct_length_hex| validate_hex(&correct_length_hex))
    }

    fn get_public_key_from_private_key(private_key: &SecretKey) -> PublicKey {
        PublicKey::from_secret_key(&Secp256k1::new(), private_key)
    }

    fn public_key_to_eth_address(public_key: &PublicKey) -> EthAddress {
        // NOTE: Need the last 20 bytes of the hash of the uncompresed form of the public key, minus it's prefix byte.
        EthAddress::from_slice(&keccak256(&public_key.serialize_uncompressed()[1..])[12..])
    }

    pub fn new_random_address() -> Result<Self> {
        Ok(Self::from_private_key(&generate_random_private_key()?))
    }

    pub fn address_starts_with(&self, prefix: &str) -> bool {
        self.address_string.starts_with(prefix)
    }

    pub fn from_private_key(private_key: &SecretKey) -> Self {
        let public_key = Self::get_public_key_from_private_key(private_key);
        let address = Self::public_key_to_eth_address(&public_key);
        EthereumKeys {
            address,
            public_key,
            private_key: *private_key,
            address_string: hex::encode(&address),
        }
    }

    pub fn to_json(&self) -> JsonValue {
        json!({
            "address": format!("0x{}", self.address_string),
            "private_key": format!("0x{:x}", self.private_key),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::key::SecretKey;
    use crate::lib::crypto_utils::generate_random_private_key;

    fn get_sample_private_key_hex() -> String {
        "decaffb75a41481965e391fb6d4406b6c356d20194c5a88935151f0513c0ffee".to_string()
    }

    fn get_sample_private_key() -> SecretKey {
        SecretKey::from_slice(&hex::decode(&get_sample_private_key_hex()).unwrap()).unwrap()
    }

    fn get_sample_ethereum_keys() -> EthereumKeys {
        EthereumKeys::from_private_key(&get_sample_private_key())
    }

    #[test]
    fn should_generate_new_random_eth_keys() {
        let result = EthereumKeys::new_random_address();
        assert!(result.is_ok());
    }

    #[test]
    fn should_create_etherem_keys_from_private_key() {
        let expected_address = "3eea9f85661bac934637b8407f9361caa14f5163";
        let pk = get_sample_private_key();
        let result = EthereumKeys::from_private_key(&pk);
        assert_eq!(result.address_string, expected_address);
    }

    #[test]
    fn should_return_false_if_address_does_not_start_with_prefix() {
        let prefix = "decaf";
        let keys = get_sample_ethereum_keys();
        let result = keys.address_starts_with(prefix);
        assert!(!result);
    }

    #[test]
    fn should_return_true_if_address_does_not_start_with_prefix() {
        let keys = get_sample_ethereum_keys();
        let prefix: String = keys.address_string.chars().take(5).collect();
        let result = keys.address_starts_with(&prefix);
        assert!(result);
    }

    #[test]
    fn should_convert_ethereum_keys_to_json_correctly() {
        let expected_result = json!({
            "address": "0x3eea9f85661bac934637b8407f9361caa14f5163",
            "private_key": "0xdecaffb75a41481965e391fb6d4406b6c356d20194c5a88935151f0513c0ffee"
        });
        let keys = get_sample_ethereum_keys();
        let result = keys.to_json();
        assert_eq!(result, expected_result);
    }
}
