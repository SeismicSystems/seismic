use alloy_consensus::{
    transaction::{RlpEcdsaDecodableTx, RlpEcdsaEncodableTx},
    SignableTransaction,
};
use alloy_eips::eip7702::Authorization;
use alloy_primitives::{
    aliases::U96, hex, keccak256, Address, Bytes, Signature, TxKind, B256, U256,
};
use alloy_rlp::{Decodable, Encodable, Header};
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use seismic_alloy_consensus::{GasPayment, TxSeismic, TxSeismicElements};
use serde_json::json;
use std::str::FromStr;

fn main() {
    let secret = SecretKey::from_slice(&hex!(
        "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    ))
    .unwrap();
    let secp = Secp256k1::new();
    let mut vectors = Vec::new();
    for version in [0, 2] {
        for payment in [
            GasPayment::Auto,
            GasPayment::Native,
            GasPayment::Token(Address::repeat_byte(0x11)),
        ] {
            for nonce in [
                U96::ZERO,
                U96::from_str("0x000101010101010101010101").unwrap(),
                U96::MAX,
            ] {
                for create in [false, true] {
                    let tx = TxSeismic {
                        chain_id: 5124, nonce: 48, gas_price: 360000, gas_limit: 169477,
                        gas_payment: payment,
                        to: if create { TxKind::Create } else { TxKind::Call(Address::from_str("0x3aB946eEC2553114040dE82D2e18798a51cf1e14").unwrap()) },
                        value: U256::from(1000000000000000u64), input: Bytes::from_static(&[1,2]),
                        seismic_elements: TxSeismicElements {
                            encryption_pubkey: PublicKey::from_str("028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0").unwrap(),
                            encryption_nonce: nonce, message_version: version,
                            recent_block_hash: B256::repeat_byte(0xab), expires_at_block: 1000000,
                            signed_read: !create,
                        },
                        authorization_list: if create { vec![Authorization {
                            chain_id: U256::from(5124), address: Address::repeat_byte(0x22), nonce: 7,
                        }.into_signed(Signature::new(U256::from(1), U256::from(2), false))] } else { vec![] },
                    };
                    let digest = tx.signature_hash();
                    let sig = secp.sign_ecdsa_recoverable(&Message::from_digest(digest.0), &secret);
                    let (recovery, compact) = sig.serialize_compact();
                    let sig = Signature::new(
                        U256::from_be_slice(&compact[..32]),
                        U256::from_be_slice(&compact[32..]),
                        i32::from(recovery) != 0,
                    );
                    let mut unsigned = vec![0x4a];
                    tx.encode(&mut unsigned);
                    let mut signed = vec![0x4a];
                    tx.rlp_encode_signed(&sig, &mut signed);
                    // Rust decoding verifies canonical bytes and mandatory nested selector.
                    assert_eq!(
                        TxSeismic::rlp_decode_signed(&mut signed[1..].as_ref())
                            .unwrap()
                            .tx(),
                        &tx
                    );
                    // Build the old raw layout (without a selector); never default it on decode.
                    let mut old_fields = Vec::new();
                    tx.chain_id.encode(&mut old_fields);
                    tx.nonce.encode(&mut old_fields);
                    tx.gas_price.encode(&mut old_fields);
                    tx.gas_limit.encode(&mut old_fields);
                    tx.to.encode(&mut old_fields);
                    tx.value.encode(&mut old_fields);
                    tx.seismic_elements.encode(&mut old_fields);
                    tx.input.encode(&mut old_fields);
                    tx.authorization_list.encode(&mut old_fields);
                    let mut old_raw = Vec::new();
                    Header {
                        list: true,
                        payload_length: old_fields.len(),
                    }
                    .encode(&mut old_raw);
                    old_raw.extend(old_fields);
                    assert!(TxSeismic::decode(&mut old_raw.as_slice()).is_err());
                    let mut old_json = serde_json::to_value(&tx).unwrap();
                    old_json.as_object_mut().unwrap().remove("gasPayment");
                    assert!(serde_json::from_value::<TxSeismic>(old_json).is_err());
                    let typed = tx.eip712_to_type_data();
                    // Typed messages carry only the authorization-list hash, not its tuples.
                    let mut hash_only_roundtrip = tx.clone();
                    hash_only_roundtrip.authorization_list.clear();
                    assert_eq!(
                        TxSeismic::eip712_decode(&typed).unwrap(),
                        hash_only_roundtrip
                    );
                    let mut old = typed.clone();
                    old.message.as_object_mut().unwrap().remove("gasPayment");
                    assert!(TxSeismic::eip712_decode(&old).is_err());
                    for invalid in [
                        json!({"kind":"3","token":Address::ZERO}),
                        json!({"kind":"2","token":Address::ZERO}),
                        json!({"kind":"0","token":Address::repeat_byte(1)}),
                        json!({"kind":"1","token":Address::ZERO,"extra":true}),
                    ] {
                        let mut bad = typed.clone();
                        bad.message["gasPayment"] = invalid;
                        assert!(TxSeismic::eip712_decode(&bad).is_err());
                    }
                    vectors.push(json!({
                        "tx": tx, "typedData": typed,
                        "unsigned": format!("0x{}", hex::encode(unsigned)),
                        "signingHash": digest, "signature": sig,
                        "signed": format!("0x{}", hex::encode(&signed)), "txHash": keccak256(signed)
                    }));
                }
            }
        }
    }
    println!("{}", serde_json::to_string_pretty(&json!({
        "provenance": "Generated by current seismic-alloy-consensus TxSeismic codecs and signature_hash; generator and source SHA-256 recorded in README.",
        "privateKey": format!("0x{}", hex::encode(secret.secret_bytes())),
        "vectors": vectors
    })).unwrap());
}
