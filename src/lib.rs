use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::ristretto::{RistrettoPoint, CompressedRistretto};
use curve25519_dalek::constants::{BASEPOINT_ORDER, RISTRETTO_BASEPOINT_TABLE};
use digest::Digest;
use typenum::consts::U64;
use rand_core::{RngCore, CryptoRng};
use rand_os::OsRng;
use subtle::ConstantTimeEq;

/// `Message` is an ElGamal message.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Message([u8; 32]);

impl Message {
    /// `new` creates a new `Message` from a slice of bytes.
    pub fn new(msg: [u8; 32]) -> Message {
        Message(msg)
    }

    /// `random` creates a new random `Message`.
    pub fn random() -> Result<Message, String> {
        let mut rng = OsRng::new()
            .map_err(|e| format!("{}", e))?;

        let msg = Message::from_rng(&mut rng);
        Ok(msg)
    }

    /// `from_rng` creates a new random `Message`, but requires
    /// to specify a random generator.
    pub fn from_rng<R>(mut rng: &mut R) -> Message
        where R: RngCore + CryptoRng
    {
        let point = RistrettoPoint::random(&mut rng).compress();
        Message::from_point(&point)
    }

    /// `from_hash` creates a new `Message` from a 64 bytes hash.
    pub fn from_hash<D>(digest: D) -> Message
        where D: Digest<OutputSize = U64> + Default
    {
        let point = RistrettoPoint::from_hash(digest).compress();
        Message::from_point(&point)
    }

    /// `from_point` creates a new `Message` from a `CompressedRistretto`.
    pub fn from_point(point: &CompressedRistretto) -> Message {
        Message(point.to_bytes())
    }

    /// `to_point` returns the inner `CompressedRistretto` of the `Message`.
    pub fn to_point(&self) -> CompressedRistretto {
        CompressedRistretto::from_slice(&self.0[..])
    }
}

/// `PrivateKey` is an ElGamal private key. It's just a
/// wrapper around `Scalar`. The key is just an integer
/// between 1 and q-1, where q is the order of the group
/// G.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct PrivateKey(Scalar);

impl PrivateKey {
    /// `new` creates a new random `PrivateKey`.
    pub fn new() -> Result<PrivateKey, String> {
        let mut rng = OsRng::new()
            .map_err(|e| format!("{}", e))?;

        PrivateKey::from_rng(&mut rng)
    }

    /// `from_rng` creates a new random `PrivateKey`, but requires
    /// to specify a random generator.
    pub fn from_rng<R>(mut rng: &mut R) -> Result<PrivateKey, String>
        where R: RngCore + CryptoRng
    {
        let mut scalar = Scalar::random(&mut rng);
        while scalar.ct_eq(&Scalar::zero()).unwrap_u8() == 1u8 {
            scalar = Scalar::random(&mut rng);
        }

        let private = PrivateKey(scalar);
        Ok(private)
    }

    /// `from_hash` creates a new `PrivateKey` from a 64 bytes hash.
    pub fn from_hash<D>(digest: D) -> PrivateKey
        where D: Digest<OutputSize = U64>
    {
        let scalar = Scalar::from_hash(digest);
        PrivateKey(scalar)
    }

    /// `from_scalar` creates a new `PrivateKey` from a `Scalar`.
    /// The `Scalar` value cannot be 0.
    pub fn from_scalar(scalar: Scalar) -> Result<PrivateKey, String> {
        if scalar.ct_eq(&Scalar::zero()).unwrap_u8() == 1u8 {
            return Err("0 scalar".into());
        }

        let private = PrivateKey(scalar);
        Ok(private)
    }

    /// `to_scalar` returns the inner `Scalar` of the `PrivateKey`.
    pub fn to_scalar(&self) -> Scalar {
        self.0
    }

    /// `from_slice` creates a new `PrivateKey` from a slice of bytes.
    pub fn from_slice(buf: [u8; 32]) -> Result<PrivateKey, String> {
        if let Some(scalar) = Scalar::from_canonical_bytes(buf) {
            let private = PrivateKey(scalar);
            Ok(private)
        } else {
            Err("not canonical bytes".into())
        }
    }

    /// `to_bytes` returns the `PrivateKey` as an array of bytes.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// `to_public` returns the `PublicKey` of the `PrivateKey`.
    pub fn to_public(&self) -> PublicKey {
        let point = &self.0 * &RISTRETTO_BASEPOINT_TABLE;
        PublicKey(point.compress())
    }
}

/// `PublicKey` is an ElGamal public key. It's just a
/// wrapper around `CompressedRistretto`.
/// The key is computed as g^x, where g is the generator
/// of the group G of order q, and x a `PrivateKey`.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct PublicKey(CompressedRistretto);

impl PublicKey {
    /// `new` creates a new `PublicKey` from a `PrivateKey`.
    pub fn new(private: PrivateKey) -> PublicKey {
        PublicKey::from_private(private)
    }

    /// `from_private` creates a new `PublicKey` from a `PrivateKey`.
    pub fn from_private(private: PrivateKey) -> PublicKey {
        private.to_public()
    }

    /// `from_point` creates a new `PublicKey` from a `CompressedRistretto`.
    pub fn from_point(point: CompressedRistretto) -> PublicKey {
        PublicKey(point)
    }

    /// `to_point` returns the inner `CompressedRistretto` of the `PublicKey`.
    pub fn to_point(&self) -> CompressedRistretto {
        self.0
    }

    /// `from_hash` creates a new `PublicKey` from a 64 bytes hash.
    pub fn from_hash<D>(digest: D) -> PublicKey
        where D: Digest<OutputSize = U64> + Default
    {
        let point = RistrettoPoint::from_hash(digest);
        PublicKey(point.compress())
    }

    /// `from_slice` creates a new `PublicKey` from a slice of bytes.
    pub fn from_slice(buf: [u8; 32]) -> PublicKey {
        let point = CompressedRistretto::from_slice(&buf[..]);
        PublicKey(point)
    }

    /// `to_bytes` returns the `PublicKey` as an array of bytes.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }
}

/// `KeyPair` is a pair of ElGamal `PublicKey` and `PrivateKey`.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct KeyPair {
    pub public_key: PublicKey,
    pub private_key: PrivateKey,
}

impl KeyPair {
    /// `new` creates a new random `KeyPair`.
    pub fn new() -> Result<KeyPair, String> {
        let private_key = PrivateKey::new()?;
        let public_key = private_key.to_public();

        let keys = KeyPair { public_key, private_key };
        Ok(keys)
    }

    /// `from_rng` creates a new random `KeyPair`, but requires
    /// to specify a random generator.
    pub fn from_rng<R>(mut rng: &mut R) -> Result<KeyPair, String>
        where R: RngCore + CryptoRng
    {
        let private_key = PrivateKey::from_rng(&mut rng)?;
        let public_key = private_key.to_public();

        let keys = KeyPair { public_key, private_key };
        Ok(keys)
    }

    /// `from_hash` creates a new `KeyPair` from a 64 bytes hash.
    pub fn from_hash<D>(digest: D) -> KeyPair
        where D: Digest<OutputSize = U64>
    {
        let private_key = PrivateKey::from_hash(digest);
        let public_key = private_key.to_public();

        KeyPair { public_key, private_key }
    }

    /// `from_scalar` creates a new `KeyPair` from a `Scalar`.
    /// The `Scalar` value cannot be 0.
    pub fn from_scalar(scalar: Scalar) -> Result<KeyPair, String> {
        let private_key = PrivateKey::from_scalar(scalar)?;
        let public_key = private_key.to_public();

        let keys = KeyPair { public_key, private_key };
        Ok(keys)
    }

    /// `from_slice` creates a new `KeyPair` from a slice of bytes.
    pub fn from_slice(buf: [u8; 32]) -> Result<KeyPair, String> {
        let private_key = PrivateKey::from_slice(buf)?;
        let public_key = private_key.to_public();

        let keys = KeyPair { public_key, private_key };
        Ok(keys)
    }
}

/// `CypherText` is the cyphertext generated by ElGamal encryption.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CypherText {
    pub gamma: CompressedRistretto,
    pub delta: CompressedRistretto,
}

/// `encrypt` encrypts a `Message` into a `CypherText`.
pub fn encrypt(msg: Message, pk: PublicKey, sk: PrivateKey) -> Result<CypherText, String> {
    // s  = pk.to_point() * sk.to_scalar()
    // c1 = RISTRETTO_BASEPOINT_TABLE * sk.to_scalar()
    // c2 = m.to_point() * s
    // (c1, c2)
    if sk.to_public().to_point().ct_eq(&pk.to_point()).unwrap_u8() == 1u8 {
        return Err("same private keys".into());
    }

    if let Some(pk_point) = pk.to_point().decompress() {
        if let Some(msg_point) = msg.to_point().decompress() {
            let sk_scalar = sk.to_scalar();
            let shared = pk_point * sk_scalar;
            let gamma_decomp = &RISTRETTO_BASEPOINT_TABLE * &sk_scalar;
            let delta_decomp = msg_point + shared;
            let gamma = gamma_decomp.compress();
            let delta = delta_decomp.compress();

            let cyph = CypherText { gamma, delta };
            Ok(cyph)
        } else {
            Err("invalid message".into())
        }
    } else {
        Err("invalid public key".into())
    }
}

/// `decrypt` decrypts a `CypherText` into a `Message`.
pub fn decrypt(cyph: CypherText, sk: PrivateKey) -> Result<Message, String> {
    // s  = cyph.c1.to_point() * sk.to_scalar() [unused as we use the Lagrange Theorem]
    // s' = cyph.c1.to_point() * (Scalar::from(ORDER)- Scalar::one(sk.to_scalar() - sk.to_scalar())
    // m  = c2.to_point() * s'.to_point()
    // m
    if let Some(gamma_point) = cyph.gamma.decompress() {
        if let Some(delta_point) = cyph.delta.decompress() {
            let sk_scalar = sk.to_scalar();
            let inv_shared = gamma_point * (BASEPOINT_ORDER - Scalar::one() - sk_scalar);
            let msg_point = delta_point - inv_shared;

            let msg = Message::from_point(&msg_point.compress());
            Ok(msg)
        } else {
            Err("invalid delta".into())
        }
    } else {
        Err("invalid gamma".into())
    }
}
