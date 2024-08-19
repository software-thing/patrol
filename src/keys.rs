use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jsonwebtoken::{
    jwk::{
        AlgorithmParameters, CommonParameters, Jwk, JwkSet, KeyAlgorithm, KeyOperations,
        PublicKeyUse, RSAKeyParameters, RSAKeyType,
    },
    DecodingKey, EncodingKey,
};
use rsa::{pkcs8::DecodePublicKey, traits::PublicKeyParts, RsaPublicKey};
use serde_json::Value;
use tokio::{fs, sync::OnceCell};

const RSA_PUBLIC_KEY: &'static str = "keys/patrol_rsa_public.pem";
const RSA_PRIVATE_KEY: &'static str = "keys/patrol_rsa_private.pem";

static DECODING_KEY: OnceCell<DecodingKey> = OnceCell::const_new();
static ENCODING_KEY: OnceCell<EncodingKey> = OnceCell::const_new();

static JWK: OnceCell<Value> = OnceCell::const_new();

pub async fn decoding_key() -> anyhow::Result<&'static DecodingKey> {
    DECODING_KEY
        .get_or_try_init(|| async {
            let public_key = fs::read(RSA_PUBLIC_KEY).await?;
            DecodingKey::from_rsa_pem(&public_key).map_err(anyhow::Error::new)
        })
        .await
}

pub async fn encoding_key() -> anyhow::Result<&'static EncodingKey> {
    ENCODING_KEY
        .get_or_try_init(|| async {
            let private_key = fs::read(RSA_PRIVATE_KEY).await?;
            EncodingKey::from_rsa_pem(&private_key).map_err(anyhow::Error::new)
        })
        .await
}

pub async fn jwk() -> anyhow::Result<&'static Value> {
    JWK.get_or_try_init(|| async {
        let public_key = fs::read_to_string(RSA_PUBLIC_KEY).await?;
        let public_key =
            RsaPublicKey::from_public_key_pem(&public_key).map_err(anyhow::Error::new)?;

        let jwk = Jwk {
            algorithm: AlgorithmParameters::RSA(RSAKeyParameters {
                key_type: RSAKeyType::RSA,
                n: URL_SAFE_NO_PAD.encode(public_key.n().to_bytes_be()),
                e: URL_SAFE_NO_PAD.encode(public_key.e().to_bytes_be()),
            }),
            common: CommonParameters {
                public_key_use: Some(PublicKeyUse::Signature),
                key_operations: Some(vec![KeyOperations::Verify]),
                key_algorithm: Some(KeyAlgorithm::RS384),
                key_id: Some("main".to_string()),
                x509_url: None,
                x509_chain: None,
                x509_sha1_fingerprint: None,
                x509_sha256_fingerprint: None,
            },
        };

        let jwk_set = JwkSet { keys: vec![jwk] };

        Ok(serde_json::to_value(jwk_set).map_err(anyhow::Error::new)?)
    })
    .await
}
