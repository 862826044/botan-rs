extern crate botan;

#[test]
fn test_version() {
    let version = botan::Version::new();

    assert_eq!(version.major, 2);
    assert!(version.release_date == 0 || version.release_date >= 20170000);
    assert!(version.ffi_api >= 20150000);

    println!("{:?}", version);
}

#[test]
fn test_hash() {
    let hash = botan::HashFunction::new("SHA-384").unwrap();

    assert_eq!(hash.output_length(), 48);

    assert!(hash.update(&[97,98]).is_ok());

    let hash_dup = hash.duplicate().unwrap();

    assert!(hash.update(&[99]).is_ok());
    assert!(hash_dup.update(&[100]).is_ok());

    let digest = hash.finish().unwrap();

    assert_eq!(digest[0], 0xCB);
    assert_eq!(digest[1], 0x00);
    assert_eq!(digest[47], 0xA7);

    let digest_dup = hash_dup.finish().unwrap();

    assert_eq!(digest_dup[0], 0x5D);
    assert_eq!(digest_dup[1], 0x15);
    assert_eq!(digest_dup[47], 0xF7);

    let bad_hash = botan::HashFunction::new("BunnyHash9000");

    assert_eq!(bad_hash.is_err(), true);
    assert_eq!(*bad_hash.as_ref().unwrap_err(), botan::Error::NotImplemented);
}


#[test]
fn test_mac() {
    let mac = botan::MsgAuthCode::new("HMAC(SHA-384)").unwrap();

    mac.set_key(&vec![0xAA; 20]).unwrap();

    mac.update(&vec![0xDD; 1]).unwrap();
    mac.update(&vec![0xDD; 29]).unwrap();
    mac.update(&vec![0xDD; 20]).unwrap();

    let r = mac.finish().unwrap();

    println!("{:?}", r);

    assert_eq!(r[0], 0x88);
    assert_eq!(r[1], 0x06);
    assert_eq!(r[47], 0x27);

}

#[test]
fn test_block_cipher() {
    let bc = botan::BlockCipher::new("AES-128").unwrap();

    bc.set_key(&vec![0; 16]).unwrap();

    let input = vec![0; 16];

    let ctext = bc.encrypt_blocks(&input).unwrap();

    let expected = vec![0x66, 0xe9, 0x4b, 0xd4, 0xef, 0x8a, 0x2c, 0x3b, 0x88, 0x4c, 0xfa, 0x59, 0xca, 0x34, 0x2b, 0x2e];
    assert_eq!(ctext, expected);

    let ptext = bc.decrypt_blocks(&ctext).unwrap();

    assert_eq!(ptext, input);
}

#[test]
fn test_cipher() {
    let cipher = botan::Cipher::new("AES-128/GCM", botan::CipherDirection::Encrypt).unwrap();

    assert_eq!(cipher.tag_length(), 16);

    let zero16 = vec![0; 16];
    let zero12 = vec![0; 12];

    cipher.set_key(&zero16).unwrap();

    let ctext = cipher.process(&zero12, &zero16).unwrap();

    assert_eq!(ctext, botan::hex_decode("0388DACE60B6A392F328C2B971B2FE78AB6E47D42CEC13BDF53A67B21257BDDF").unwrap());

    let cipher = botan::Cipher::new("AES-128/GCM", botan::CipherDirection::Decrypt).unwrap();
    cipher.set_key(&zero16).unwrap();

    let ptext = cipher.process(&zero12, &ctext).unwrap();

    assert_eq!(ptext, zero16);
}


#[test]
fn test_kdf() {

    let salt = botan::hex_decode("000102030405060708090A0B0C").unwrap();
    let label = botan::hex_decode("F0F1F2F3F4F5F6F7F8F9").unwrap();
    let secret = botan::hex_decode("0B0B0B0B0B0B0B0B0B0B0B0B0B0B0B0B0B0B0B0B0B0B").unwrap();
    let expected_output = botan::hex_decode("3CB25F25FAACD57A90434F64D0362F2A2D2D0A90CF1A5A4C5DB02D56ECC4C5BF34007208D5B887185865").unwrap();

    let output = botan::kdf("HKDF(SHA-256)", expected_output.len(), &secret, &salt, &label).unwrap();

    assert_eq!(output, expected_output);
}

#[test]
fn test_pbkdf() {

    let salt = botan::hex_decode("0001020304050607").unwrap();
    let iterations = 10000;
    let passphrase = "xyz";
    let expected_output = botan::hex_decode("DEFD2987FA26A4672F4D16D98398432AD95E896BF619F6A6B8D4ED").unwrap();

    let output = botan::pbkdf("PBKDF2(SHA-256)", expected_output.len(), passphrase, &salt, iterations).unwrap();

    assert_eq!(output, expected_output);
}

#[test]
fn test_hex() {
    let raw = vec![1,2,3,255,42,23];
    assert_eq!(botan::hex_encode(&raw).unwrap(), "010203FF2A17");
    assert_eq!(botan::hex_decode("010203FF2A17").unwrap(), raw);
}

#[test]
fn test_rng() {
    let rng = botan::RandomNumberGenerator::new_system().unwrap();

    let read1 = rng.read(10).unwrap();
    let read2 = rng.read(10).unwrap();

    assert!(read1 != read2);
}

#[test]
fn test_bcrypt() {
    let pass = "password";
    let rng = botan::RandomNumberGenerator::new_system().unwrap();

    let bcrypt1 = botan::bcrypt_hash(pass, &rng, 10).unwrap();

    assert_eq!(bcrypt1.len(), 60);

    let bcrypt2 = botan::bcrypt_hash(pass, &rng, 10).unwrap();

    assert_eq!(bcrypt2.len(), 60);

    assert!(bcrypt1 != bcrypt2);

    assert!(botan::bcrypt_verify(pass, &bcrypt1).unwrap());
    assert!(botan::bcrypt_verify(pass, &bcrypt2).unwrap());

    assert_eq!(botan::bcrypt_verify("passwurd", &bcrypt2).unwrap(), false);
}

#[test]
fn test_pubkey() {
    let rng = botan::RandomNumberGenerator::new_system().unwrap();

    let ecdsa_key = botan::Privkey::create("ECDSA", "secp256r1", &rng).unwrap();

    assert!(ecdsa_key.check_key(&rng).unwrap(), true);
    assert_eq!(ecdsa_key.algo_name().unwrap(), "ECDSA");

    let pub_key = ecdsa_key.pubkey().unwrap();

    assert_eq!(pub_key.algo_name().unwrap(), "ECDSA");

    let bits = ecdsa_key.der_encode().unwrap();
    let pem = ecdsa_key.pem_encode().unwrap();
    assert!(pem.starts_with("-----BEGIN PRIVATE KEY-----\n"));
    assert!(pem.ends_with("-----END PRIVATE KEY-----\n"));

    let pub_bits = pub_key.der_encode().unwrap();
    let pub_pem = pub_key.pem_encode().unwrap();
    assert!(pub_pem.starts_with("-----BEGIN PUBLIC KEY-----\n"));
    assert!(pub_pem.ends_with("-----END PUBLIC KEY-----\n"));

    let loaded_key = botan::Privkey::load_der(&bits).unwrap();
    assert!(loaded_key.check_key(&rng).unwrap(), true);

    let loaded_bits = loaded_key.der_encode().unwrap();
    let loaded_pub_key = loaded_key.pubkey().unwrap();
    assert_eq!(loaded_pub_key.algo_name().unwrap(), "ECDSA");
    let loaded_pub_bits = loaded_pub_key.der_encode().unwrap();

    assert_eq!(bits, loaded_bits);
    assert_eq!(pub_bits, loaded_pub_bits);
}

#[test]
fn test_pubkey_sign() {
    let msg = vec![1,23,42];

    let rng = botan::RandomNumberGenerator::new_system().unwrap();

    let ecdsa_key = botan::Privkey::create("ECDSA", "secp256r1", &rng).unwrap();
    assert!(ecdsa_key.key_agreement_key().is_err());

    let signer = botan::Signer::new(&ecdsa_key, "EMSA1(SHA-256)").unwrap();

    signer.update(&msg).unwrap();
    let signature = signer.finish(&rng).unwrap();

    let pub_key = ecdsa_key.pubkey().unwrap();

    let verifier = botan::Verifier::new(&pub_key, "EMSA1(SHA-256)").unwrap();

    verifier.update(&[1]).unwrap();
    verifier.update(&[23, 42]).unwrap();

    assert_eq!(verifier.finish(&signature).unwrap(), true);

    verifier.update(&[1]).unwrap();
    assert_eq!(verifier.finish(&signature).unwrap(), false);

    verifier.update(&[1]).unwrap();
    verifier.update(&[23, 42]).unwrap();

    assert_eq!(verifier.finish(&signature).unwrap(), true);

}

#[test]
fn test_pubkey_encrypt() {
    let msg = vec![1,23,42];

    let rng = botan::RandomNumberGenerator::new_system().unwrap();

    let priv_key = botan::Privkey::create("RSA", "2048", &rng).unwrap();
    assert!(priv_key.key_agreement_key().is_err());
    let pub_key = priv_key.pubkey().unwrap();

    let encryptor = botan::Encryptor::new(&pub_key, "OAEP(SHA-256)").unwrap();

    let ctext = encryptor.encrypt(&msg, &rng).unwrap();
    assert_eq!(ctext.len(), 2048/8);

    let decryptor = botan::Decryptor::new(&priv_key, "OAEP(SHA-256)").unwrap();

    let ptext = decryptor.decrypt(&ctext).unwrap();

    assert_eq!(ptext, msg);
}

#[test]
fn test_pubkey_key_agreement() {

    let rng = botan::RandomNumberGenerator::new_system().unwrap();

    let a_priv = botan::Privkey::create("ECDH", "secp384r1", &rng).unwrap();
    let b_priv = botan::Privkey::create("ECDH", "secp384r1", &rng).unwrap();

    let a_pub = a_priv.key_agreement_key().unwrap();
    let b_pub = b_priv.key_agreement_key().unwrap();

    let a_ka = botan::KeyAgreement::new(&a_priv, "KDF2(SHA-384)").unwrap();
    let b_ka = botan::KeyAgreement::new(&b_priv, "KDF2(SHA-384)").unwrap();

    let salt = rng.read(16).unwrap();

    let a_key = a_ka.agree(&b_pub, &salt).unwrap();
    let b_key = b_ka.agree(&a_pub, &salt).unwrap();

    assert_eq!(a_key, b_key);
}

#[test]
fn test_ct_compare() {
    let a = vec![1,2,3];

    assert_eq!(botan::const_time_compare(&a, &[1,2,3]), true);
    assert_eq!(botan::const_time_compare(&a, &[1,2,3,4]), false);
    assert_eq!(botan::const_time_compare(&a, &[1,2,4]), false);
    assert_eq!(botan::const_time_compare(&a, &a), true);
    assert_eq!(botan::const_time_compare(&a, &vec![1,2,3]), true);
}

#[test]
fn test_scrub_mem() {
    let mut v = vec![1,2,3];
    botan::scrub_mem(&mut v);
    assert_eq!(v, vec![0,0,0]);

    let mut a = [1u32, 2u32, 3u32, 2049903u32];
    botan::scrub_mem(&mut a);
    assert_eq!(a, [0,0,0,0]);
}
