#![no_std]

extern crate panic_halt;

use core::convert::TryFrom;

pub use salty::{
    Error,
    Result,
    constants::{
        FIELD_ELEMENT_LENGTH,
        SCALAR_LENGTH,
        COMPRESSED_Y_LENGTH,
        SECRETKEY_SEED_LENGTH,
        SECRETKEY_NONCE_LENGTH,
        SECRETKEY_SCALAR_LENGTH,
        PUBLICKEY_SERIALIZED_LENGTH,
        SIGNATURE_SERIALIZED_LENGTH,
        SHA512_LENGTH,
    },
};

use salty::{
    Keypair,
    // SecretKey,
    PublicKey,
    Signature,
};

#[no_mangle]
/// Generates a public key from a secret seed. Use to verify signatures.
pub unsafe extern "C" fn salty_public_key(
    seed: &[u8; SECRETKEY_SEED_LENGTH],
    public_key: &mut [u8; PUBLICKEY_SERIALIZED_LENGTH],
) {
    let keypair = Keypair::from(seed);
    public_key.copy_from_slice(keypair.public.compressed.as_bytes());
}

#[no_mangle]
/// Signs the data, based on the keypair generated from the secret seed.
pub unsafe extern "C" fn salty_sign(
    seed: &[u8; SECRETKEY_SEED_LENGTH],
    data_ptr: *const u8,
    data_len: usize,
    signature: &mut [u8; SIGNATURE_SERIALIZED_LENGTH],
) {
    let keypair = Keypair::from(seed);
    let data = core::slice::from_raw_parts(data_ptr, data_len);

    signature.copy_from_slice(
        &keypair.sign(data).to_bytes()
    );
}

#[no_mangle]
pub unsafe extern "C" fn salty_sign_get_first_hash_init_data(
    seed: &[u8; SECRETKEY_SEED_LENGTH],
    first_hash_init: &mut [u8; SECRETKEY_NONCE_LENGTH],
) {
    let keypair = Keypair::from(seed);
    first_hash_init.copy_from_slice(&keypair.sign_get_first_hash_init_data()[..]);
}

#[no_mangle]
pub unsafe extern "C" fn salty_sign_get_second_hash_init_data(
    seed: &[u8; SECRETKEY_SEED_LENGTH],
    first_hash: &[u8; SHA512_LENGTH],
    second_hash_init: &mut [u8; 64],
    secret_r: &mut [u8; SCALAR_LENGTH],
) {
    let keypair = Keypair::from(seed);
    let state = keypair.sign_get_second_hash_init_data(first_hash);
    second_hash_init.copy_from_slice(&state.hash_data);
    secret_r.copy_from_slice(&state.secret_r);
}

#[no_mangle]
pub unsafe extern "C" fn salty_sign_finalize(
    seed: &[u8; SECRETKEY_SEED_LENGTH],
    second_hash: &[u8; SHA512_LENGTH],
    secret_r: &[u8; SCALAR_LENGTH],
    signature: &mut [u8; SIGNATURE_SERIALIZED_LENGTH],
) {
    let keypair = Keypair::from(seed);
    signature.copy_from_slice(&keypair.sign_finalize(second_hash, secret_r).to_bytes())
}

#[no_mangle]
/// Signs the data for a given context, based on the keypair generated
/// from the secret seed.
pub unsafe extern "C" fn salty_sign_with_context(
    seed: &[u8; SECRETKEY_SEED_LENGTH],
    data_ptr: *const u8,
    data_len: usize,
    context_ptr: *const u8,
    context_len: usize,
    signature: &mut [u8; SIGNATURE_SERIALIZED_LENGTH],
) -> Error {
    if context_len > 255 {
        return Error::ContextTooLong;
    }
    let keypair = Keypair::from(seed);
    let data = core::slice::from_raw_parts(data_ptr, data_len);
    let context = core::slice::from_raw_parts(context_ptr, context_len);

    signature.copy_from_slice(
        &keypair.sign_with_context(data, context)
        .to_bytes()
    );
    Error::NoError
}

#[no_mangle]
/// Signs the prehashed data, based on the keypair generated from the secret seed.
/// An optional context can also be passed (this is recommended).
pub unsafe extern "C" fn salty_sign_prehashed(
    seed: &[u8; SECRETKEY_SEED_LENGTH],
    prehashed_data: &[u8; SHA512_LENGTH],
    context_ptr: *const u8,
    context_len: usize,
    signature: &mut [u8; SIGNATURE_SERIALIZED_LENGTH],
) -> Error {
    if context_len > 255 {
        return Error::ContextTooLong;
    }
    let keypair = Keypair::from(seed);
    let context = core::slice::from_raw_parts(context_ptr, context_len);

    signature.copy_from_slice(
        &keypair.sign_prehashed(prehashed_data, Some(context))
        .to_bytes()
    );

    Error::NoError
}

#[no_mangle]
/// Verify a presumed signature on the given data.
pub unsafe extern "C" fn salty_verify(
    public_key: &[u8; PUBLICKEY_SERIALIZED_LENGTH],
    data_ptr: *const u8,
    data_len: usize,
    signature: &[u8; SIGNATURE_SERIALIZED_LENGTH],
) -> Error {
    let maybe_public_key = PublicKey::try_from(public_key);
    if maybe_public_key.is_err() {
        return maybe_public_key.err().unwrap()
    }
    let public_key = maybe_public_key.unwrap();

    let data = core::slice::from_raw_parts(data_ptr, data_len);
    let signature = Signature::from(signature);
    let verification = public_key.verify(data, &signature);

    if verification.is_err() {
        return verification.err().unwrap()
    }
    return Error::NoError;
}

#[no_mangle]
/// Verify a presumed signature on the given data.
pub unsafe extern "C" fn salty_verify_with_context(
    public_key: &[u8; PUBLICKEY_SERIALIZED_LENGTH],
    data_ptr: *const u8,
    data_len: usize,
    signature: &[u8; SIGNATURE_SERIALIZED_LENGTH],
    context_ptr: *const u8,
    context_len: usize,
) -> Error {
    if context_len > 255 {
        return Error::ContextTooLong;
    }
    let maybe_public_key = PublicKey::try_from(public_key);
    if maybe_public_key.is_err() {
        return maybe_public_key.err().unwrap()
    }
    let public_key = maybe_public_key.unwrap();

    let data = core::slice::from_raw_parts(data_ptr, data_len);
    let signature = Signature::from(signature);
    let context = core::slice::from_raw_parts(context_ptr, context_len);
    let verification = public_key.verify_with_context(data, &signature, context);
    if verification.is_err() {
        return verification.err().unwrap()
    }
    return Error::NoError;
}

#[no_mangle]
/// Verify a presumed signature on the given data.
pub unsafe extern "C" fn salty_verify_prehashed(
    public_key: &[u8; PUBLICKEY_SERIALIZED_LENGTH],
    prehashed_data: &[u8; SHA512_LENGTH],
    signature: &[u8; SIGNATURE_SERIALIZED_LENGTH],
    context_ptr: *const u8,
    context_len: usize,
) -> Error {
    if context_len > 255 {
        return Error::ContextTooLong;
    }
    let maybe_public_key = PublicKey::try_from(public_key);
    if maybe_public_key.is_err() {
        return maybe_public_key.err().unwrap()
    }
    let public_key = maybe_public_key.unwrap();
    let signature = Signature::from(signature);
    let context = core::slice::from_raw_parts(context_ptr, context_len);
    let verification = public_key.verify_prehashed(prehashed_data, &signature, Some(context));
    if verification.is_err() {
        return verification.err().unwrap()
    }
    return Error::NoError;
}

#[no_mangle]
/// Perform X25519 key agreement.
pub unsafe extern "C" fn salty_agree(
    scalar: &[u8; SECRETKEY_SEED_LENGTH],
    input_u: &[u8; FIELD_ELEMENT_LENGTH],
    output_u: &mut [u8; FIELD_ELEMENT_LENGTH],
) {
    let shared_secret = salty::agreement::x25519(*scalar, *input_u);
    output_u.copy_from_slice(&shared_secret);
}

