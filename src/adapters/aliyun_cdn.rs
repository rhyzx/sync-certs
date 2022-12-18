use chrono::{Timelike, Utc};
use sha2::{Digest, Sha256};

use crate::applyment::Applyment;

use super::common::hmac_sha256;

// Outdated https://help.aliyun.com/document_detail/315526.htm
// https://help.aliyun.com/document_detail/448069.htm
// https://cdn.jsdelivr.net/npm/@alicloud/openapi-client@0.4.4/src/client.ts
// https://cdn.jsdelivr.net/npm/@alicloud/openapi-util@0.3.0/src/client.ts
fn get_authorization(
    access_key_id: &str,
    access_key_secret: &str,
    headers: &[(&str, &str)],
    payload_digest: &str,
) -> String {
    let header_names = headers
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join(";");

    let canonical_request = format!(
        "POST\n/\n\n{headers}\n\n{header_names}\n{payload_digest}",
        headers = headers
            .iter()
            .map(|(name, value)| format!("{}:{}", name, value))
            .collect::<Vec<_>>()
            .join("\n"),
    );

    let string_to_sign = format!("ACS3-HMAC-SHA256\n{:x}", Sha256::digest(canonical_request));
    let signature = hmac_sha256(access_key_secret.as_bytes(), string_to_sign.as_bytes());

    format!("ACS3-HMAC-SHA256 Credential={access_key_id},SignedHeaders={header_names},Signature={signature:x}")
}

pub async fn apply(applyment: &Applyment<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let access_key_id_env = format!("{}ACCESS_KEY_ID", applyment.env_prefix);
    let access_key_secret_env = format!("{}ACCESS_KEY_SECRET", applyment.env_prefix);
    let access_key_id = std::env::var(&access_key_id_env)
        .unwrap_or_else(|_| panic!("Failed to retrieve env: {}", &access_key_id_env));
    let access_key_secret = std::env::var(&access_key_secret_env)
        .unwrap_or_else(|_| panic!("Failed to retrieve env: {}", &access_key_secret_env));

    let dt = format!("{:?}", Utc::now().with_nanosecond(0).unwrap());
    let nonce = format!("{:x}", rand::random::<u128>());

    let payload = serde_urlencoded::to_string([
        ("DomainName", applyment.domain),
        ("PrivateKey", applyment.cert_key),
        ("ServerCertificate", applyment.cert),
        ("ServerCertificateStatus", "on"),
    ])
    .unwrap();
    let payload_digest = format!("{:x}", Sha256::digest(payload.as_bytes()));

    let headers = [
        // attention: lexical order
        ("content-type", "application/x-www-form-urlencoded"),
        ("host", "cdn.aliyuncs.com"),
        ("x-acs-action", "SetDomainServerCertificate"),
        ("x-acs-content-sha256", &payload_digest),
        ("x-acs-date", &dt),
        ("x-acs-signature-nonce", &nonce),
        ("x-acs-version", "2018-05-10"),
    ];

    let authorization = get_authorization(
        &access_key_id,
        &access_key_secret,
        &headers,
        &payload_digest,
    );

    let client = reqwest::Client::new();
    let mut req = client
        .post("https://cdn.aliyuncs.com")
        // .post("https://httpbin.org/anything")
        .header("Authorization", authorization)
        .header("Accept", "application/json")
        .body(payload);

    for (name, value) in &headers {
        req = req.header(*name, *value);
    }

    let res = req.send().await?.json::<serde_json::Value>().await?;

    if let Some(err) = res["Message"].as_str() {
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
        let headers = [
            // attention: lexical order
            ("content-type", "application/x-www-form-urlencoded"),
            ("host", "cdn.aliyuncs.com"),
            ("x-acs-action", "SetDomainServerCertificate"),
            (
                "x-acs-content-sha256",
                "66fc6ffea69e7280d1ea30552a96f16ca148081d2e879bbff84a9e594723ba5f",
            ),
            ("x-acs-date", "2022-12-18T05:45:41Z"),
            ("x-acs-signature-nonce", "250ddd97577a5443c7691433bd88b6b9"),
            ("x-acs-version", "2018-05-10"),
        ];

        assert_eq!(
          get_authorization(
            "ACCESS_KEY_ID",
            "ACCESS_KEY_SECRET",
            &headers,
            "66fc6ffea69e7280d1ea30552a96f16ca148081d2e879bbff84a9e594723ba5f",
        ),
            "ACS3-HMAC-SHA256 Credential=ACCESS_KEY_ID,SignedHeaders=content-type;host;x-acs-action;x-acs-content-sha256;x-acs-date;x-acs-signature-nonce;x-acs-version,Signature=f1382927235c67253a0786219989709867f0b396eedaed1329b1613d0734f2f6"
        )
    }

    #[tokio::test]
    async fn test_apply() {
        let domain = &std::env::var("TEST_ALIYUN_DOMAIN").unwrap();
        let cert = &std::env::var("TEST_ALIYUN_CERT")
            .unwrap()
            .replace("\\n", "\n"); // workaround for direnv not supporting multiline
        let cert_key = &std::env::var("TEST_ALIYUN_CERT_KEY")
            .unwrap()
            .replace("\\n", "\n");

        apply(&Applyment {
            idx: "0",
            adapter: "aliyun_cdn",
            domain,
            env_prefix: "TEST_ALIYUN_",
            extra: None,
            last_applied_digest: "",
            cert,
            cert_key,
        })
        .await
        .unwrap();
    }
}
