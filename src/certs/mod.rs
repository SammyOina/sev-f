mod encoders;
mod decoders;
mod crypto;

#[cfg(test)]
mod naples;

pub use self::crypto::PrivateKey;

trait EncodeBuf<T>: codicon::Encoder<T> {
    fn encode_buf(&self, params: T) -> Result<Vec<u8>, Self::Error> {
        let mut buf = Vec::new();
        self.encode(&mut buf, params)?;
        Ok(buf)
    }
}

impl<T, U: codicon::Encoder<T>> EncodeBuf<T> for U {}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    Invalid(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::IoError(ref e) => e.fmt(f),
            Error::Invalid(ref s) => write!(f, "invalid: {}", s),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Unspecified;

impl std::error::Error for Unspecified {}

impl std::fmt::Display for Unspecified {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unspecified")
    }
}

impl From<Error> for Unspecified {
    fn from(_: Error) -> Self {
        Unspecified
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Usage {
    OwnerCertificateAuthority = 2,
    PlatformEndorsementKey = 1,
    PlatformDiffieHellman = 0,
    ChipEndorsementKey = 3,
    AmdRootKey = 5,
    AmdSevKey = 4,
}

impl Usage {
    const ALL: &'static [Usage] = &[ Usage::PlatformDiffieHellman,
        Usage::PlatformEndorsementKey, Usage::OwnerCertificateAuthority,
        Usage::ChipEndorsementKey, Usage::AmdSevKey, Usage::AmdRootKey ];

    pub fn kind(self) -> Kind {
        match self {
            Usage::OwnerCertificateAuthority => Kind::Sev,
            Usage::PlatformEndorsementKey => Kind::Sev,
            Usage::PlatformDiffieHellman => Kind::Sev,
            Usage::ChipEndorsementKey => Kind::Sev,
            Usage::AmdRootKey => Kind::Ca,
            Usage::AmdSevKey => Kind::Ca,
        }
    }
}

impl std::fmt::Display for Usage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", match self {
            Usage::OwnerCertificateAuthority => "OCA",
            Usage::PlatformEndorsementKey => "PEK",
            Usage::PlatformDiffieHellman => "PDH",
            Usage::ChipEndorsementKey => "CEK",
            Usage::AmdRootKey => "ARK",
            Usage::AmdSevKey => "ASK",
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SigAlgo {
    RsaSha256,
    EcdsaSha256,
    RsaSha384,
    EcdsaSha384,
}

impl std::fmt::Display for SigAlgo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SigAlgo::RsaSha256 => write!(f, "RSA(SHA256)"),
            SigAlgo::RsaSha384 => write!(f, "RSA(SHA384)"),
            SigAlgo::EcdsaSha256 => write!(f, "ECDSA(SHA256)"),
            SigAlgo::EcdsaSha384 => write!(f, "ECDSA(SHA384)"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExcAlgo {
    EcdhSha256,
    EcdhSha384,
}

impl std::fmt::Display for ExcAlgo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExcAlgo::EcdhSha256 => write!(f, "ECDH(SHA256)"),
            ExcAlgo::EcdhSha384 => write!(f, "ECDH(SHA384)"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Algo {
    Sig(SigAlgo),
    Exc(ExcAlgo),
}

impl std::fmt::Display for Algo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Algo::Sig(s) => write!(f, "{}", s),
            Algo::Exc(e) => write!(f, "{}", e),
        }
    }
}

impl From<SigAlgo> for Algo {
    fn from(value: SigAlgo) -> Algo {
        Algo::Sig(value)
    }
}

impl From<ExcAlgo> for Algo {
    fn from(value: ExcAlgo) -> Algo {
        Algo::Exc(value)
    }
}

impl PartialEq<SigAlgo> for Algo {
    fn eq(&self, other: &SigAlgo) -> bool {
        *self == Algo::from(*other)
    }
}

impl PartialEq<ExcAlgo> for Algo {
    fn eq(&self, other: &ExcAlgo) -> bool {
        *self == Algo::from(*other)
    }
}

#[derive(Copy, Clone)]
pub struct RsaKey {
    pub pubexp: [u8; 4096 / 8],
    pub modulus: [u8; 4096 / 8],
}

impl std::cmp::Eq for RsaKey {}
impl std::cmp::PartialEq for RsaKey {
    fn eq(&self, other: &RsaKey) -> bool {
        self.pubexp[..] == other.pubexp[..]
            && self.modulus[..] == other.modulus[..]
    }
}

impl std::fmt::Debug for RsaKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "RsaKey {{ pubexp: {:?}, modulus: {:?} }}",
            self.pubexp.to_vec(), self.modulus.to_vec())
    }
}

impl RsaKey {
    fn psize(&self) -> usize {
        match self.pubexp.iter().rev().skip_while(|b| **b == 0).count() * 8 {
            0 ... 2048 => 2048 / 8,
            _ => 4096 / 8,
        }
    }

    fn msize(&self) -> usize {
        match self.pubexp.iter().rev().skip_while(|b| **b == 0).count() * 8 {
            0 ... 2048 => 2048 / 8,
            _ => 4096 / 8,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Curve {
    P256,
    P384,
}

impl std::fmt::Display for Curve {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", match self {
            Curve::P256 => "P256",
            Curve::P384 => "P384",
        })
    }
}

#[derive(Copy, Clone)]
pub struct EccKey {
    pub c: Curve,
    pub x: [u8; 576 / 8],
    pub y: [u8; 576 / 8],
}

impl std::cmp::Eq for EccKey {}
impl std::cmp::PartialEq for EccKey {
    fn eq(&self, other: &EccKey) -> bool {
        self.x[..] == other.x[..] && self.y[..] == other.y[..]
    }
}

impl std::fmt::Debug for EccKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "EccKey {{ c: {:?}, x: {:?}, y: {:?} }}",
            self.c, self.x.to_vec(), self.y.to_vec())
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyType {
    Rsa(RsaKey),
    Ecc(EccKey),
}

impl std::fmt::Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            KeyType::Rsa(ref r) => write!(f, "RSA({})", r.msize() * 8),
            KeyType::Ecc(ref e) => write!(f, "ECC({})", e.c),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PublicKey {
    pub usage: Usage,
    pub algo: Algo,
    pub key: KeyType,
    pub id: Option<std::num::NonZeroU128>,
}

impl std::fmt::Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let id = self.id.and_then(|i| Some(i.get())).unwrap_or(0u128);
        write!(f, "{} {} {}: {:039X}", self.usage, self.algo, self.key, id)
    }
}

#[derive(Copy, Clone)]
pub struct Signature {
    pub usage: Usage,
    pub algo: SigAlgo,
    pub sig: [u8; 4096 / 8],
    pub id: Option<std::num::NonZeroU128>,
}

impl std::cmp::Eq for Signature {}
impl std::cmp::PartialEq for Signature {
    fn eq(&self, other: &Signature) -> bool {
        self.usage == other.usage
            && self.algo == other.algo
            && self.sig[..] == other.sig[..]
            && self.id == other.id
    }
}

impl std::fmt::Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Signature {{ usage: {:?}, algo: {:?}, sig: {:?}, id: {:?} }}",
            self.usage, self.algo, self.sig.to_vec(), self.id)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Firmware(pub u8, pub u8);

impl std::fmt::Display for Firmware {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}

#[derive(Copy, Clone, Debug, Eq)]
pub struct Certificate {
    pub version: u32,
    pub firmware: Option<Firmware>,
    pub key: PublicKey,
    pub sigs: [Option<Signature>; 2],
}

impl PartialEq for Certificate {
    fn eq(&self, other: &Certificate) -> bool {
        self.version == other.version
            && self.firmware == other.firmware
            && self.key == other.key
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Sev,
    Ca
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Full;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Body;

pub trait Verifier<'a> {
    fn verify(self) -> Result<&'a Certificate, Unspecified>;
}