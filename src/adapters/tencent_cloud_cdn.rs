use hmac::digest::generic_array::{typenum::consts::U32, GenericArray};
use hmac::{Hmac, Mac};
use json_patch::merge;
use sha2::{Digest, Sha256};
use time::macros::format_description;
use time::OffsetDateTime;

use crate::applyment::Applyment;

fn hmac_sha256(key: &[u8], data: &[u8]) -> GenericArray<u8, U32> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(data);
    mac.finalize().into_bytes()
}

// https://cloud.tencent.com/document/product/1278/55260
// https://cloud.tencent.com/document/api/228/41116
fn get_authorization(
    secret_id: &str,
    secret_key: &str,
    date: &str,
    timestamp: &str,
    payload: &str,
) -> String {
    let payload_digest = Sha256::digest(payload);
    let canonical_request = format!("POST\n/\n\ncontent-type:application/json\nhost:cdn.tencentcloudapi.com\n\ncontent-type;host\n{payload_digest:x}");

    let raw_sign = format!(
        "TC3-HMAC-SHA256\n{timestamp}\n{date}/cdn/tc3_request\n{canonical_request_digest:x}",
        canonical_request_digest = Sha256::digest(canonical_request)
    );

    let secret_date = hmac_sha256(format!("TC3{secret_key}").as_bytes(), date.as_bytes());
    let secret_service = hmac_sha256(&secret_date, b"cdn");
    let secret_signing = hmac_sha256(&secret_service, b"tc3_request");
    let signature = hmac_sha256(&secret_signing, raw_sign.as_bytes());

    format!("TC3-HMAC-SHA256 Credential={secret_id}/{date}/cdn/tc3_request, SignedHeaders=content-type;host, Signature={signature:x}")
}

pub async fn apply(applyment: &Applyment<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let secret_id_env = format!("{}SECRET_ID", applyment.env_prefix);
    let secret_key_env = format!("{}SECRET_KEY", applyment.env_prefix);
    let secret_id = std::env::var(&secret_id_env)
        .unwrap_or_else(|_| panic!("Failed to retrieve env: {}", &secret_id_env));
    let secret_key = std::env::var(&secret_key_env)
        .unwrap_or_else(|_| panic!("Failed to retrieve env: {}", &secret_key_env));

    let time = OffsetDateTime::now_utc();
    let date = time
        .format(format_description!("[year]-[month]-[day]"))
        .unwrap();
    let timestamp = time.unix_timestamp().to_string();

    let mut payload = serde_json::json!({
        "Https": applyment.extra.map_or(serde_json::Value::Null, |str| {
            serde_json::from_str(str).expect("parse `extra` failed")
        })
    });
    merge(
        &mut payload,
        &serde_json::json!({
            "Domain": applyment.domain,
            "Https": {
                "Switch": "on",
                "CertInfo": {
                    "Certificate": applyment.cert,
                    "PrivateKey": applyment.cert_key,
                }
            }
        }),
    );
    let payload = payload.to_string();

    let authorization = get_authorization(&secret_id, &secret_key, &date, &timestamp, &payload);

    let client = reqwest::Client::new();
    let res = client
        .post("https://cdn.tencentcloudapi.com")
        // .post("https://httpbin.org/anything")
        .header("Authorization", &authorization)
        .header("X-TC-Action", "UpdateDomainConfig")
        .header("X-TC-Version", "2018-06-06")
        .header("X-TC-Timestamp", &timestamp)
        // .header("X-TC-Language", "zh-CN")
        // use `send_string` instead of `send_json` to avoid repeated serialization
        .header("Content-Type", "application/json")
        .body(payload)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    if let Some(err) = res["Response"]["Error"]["Message"].as_str() {
        Err(err.into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth() {
        assert_eq!(
            get_authorization("aaa", "bbb", "2022-12-09", "1670600000", r#"{"Domain":"example.com","Https":{"Switch":"on","Http2":"on"}}"#),
            "TC3-HMAC-SHA256 Credential=aaa/2022-12-09/cdn/tc3_request, SignedHeaders=content-type;host, Signature=e46fe351359b7d0fb72739d4c9c216ab21059ae5a54c2e371ddfe9c7f55a1fdc"
        )
    }

    #[tokio::test]
    async fn test_apply() {
        let domain = &std::env::var("TEST_TENCENT_DOMAIN").unwrap();
        let cert = &std::env::var("TEST_TENCENT_CERT")
            .unwrap()
            .replace("\\n", "\n"); // workaround for direnv not supporting multiline
        let cert_key = &std::env::var("TEST_TENCENT_CERT_KEY")
            .unwrap()
            .replace("\\n", "\n");

        apply(&Applyment {
            idx: "0",
            adapter: "tencent_cloud_cdn",
            domain,
            env_prefix: "TEST_TENCENT_",
            extra: Some(r#"{"Http2": "on", "Hsts": {"Switch":"on", "MaxAge": 31536000}}"#),
            last_applied_digest: "",
            cert,
            cert_key,
        })
        .await
        .unwrap();
    }
}
