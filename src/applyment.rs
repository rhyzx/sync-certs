use std::collections::{BTreeMap, HashMap};

use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct Applyment<'a> {
    pub idx: &'a str,
    pub adapter: &'a str,
    pub domain: &'a str,
    pub env_prefix: &'a str,
    pub extra: Option<&'a str>,
    pub last_applied_digest: &'a str,
    pub cert: &'a str,
    pub cert_key: &'a str,
}

impl<'a> Applyment<'a> {
    // from `{"sync-certs.io/0.adapter": "tencent_cloud_cdn", "sync-certs.io/0.domain": "example.com"}`
    pub fn from_annotations(
        map: &'a BTreeMap<String, String>,
        cert: &'a str,
        cert_key: &'a str,
    ) -> Vec<Self> {
        map.iter()
            .filter_map(|(k, v)| {
                let mut parts = k.split('/');
                if parts.next() == Some("sync-certs.io") {
                    let mut parts = parts.next()?.split('.');
                    let idx = parts.next()?;
                    let key = parts.next()?;
                    Some((idx, key, &v[..]))
                } else {
                    None
                }
            })
            .fold(HashMap::new(), |mut acc, (idx, key, value)| {
                let entry = acc.entry(idx).or_insert_with(HashMap::new);
                entry.insert(key, value);
                acc
            })
            .into_iter()
            .filter_map(|(idx, map)| {
                Some(Applyment {
                    idx,
                    // TODO log adapter/domain not exists
                    adapter: map.get("adapter").copied()?,
                    domain: map.get("domain").copied()?,
                    env_prefix: map.get("env-prefix").copied().unwrap_or(""),
                    extra: map.get("extra").copied(),
                    last_applied_digest: map.get("_last-applied-digest").copied().unwrap_or(""),
                    cert,
                    cert_key,
                })
            })
            .collect()
    }

    pub fn digest(&self) -> String {
        let bytes = Sha256::digest(format!(
            "{}.{}.{}.{}.{}.{}",
            self.adapter.replace('.', "\\."),
            self.domain.replace('.', "\\."),
            self.env_prefix.replace('.', "\\."),
            self.extra.unwrap_or("").replace('.', "\\."),
            self.cert.replace('.', "\\."),
            self.cert_key.replace('.', "\\.")
        ));
        format!("{:x}", bytes)
    }

    pub fn stale_patch(&self) -> Option<serde_json::Value> {
        let digest = self.digest();
        if digest != self.last_applied_digest {
            Some(serde_json::json!({
                "metadata": {
                    "annotations": {
                        format!("sync-certs.io/{}._last-applied-digest", self.idx): digest
                    }
                }
            }))
        } else {
            None
        }
    }

    pub fn identifier(&self) -> String {
        format!("{}@{}", self.domain, self.adapter)
    }
}

#[test]
fn test() {
    let map = BTreeMap::from(
        [
            ("sync-certs.io/0.adapter", "tencent_cloud_cdn"),
            ("sync-certs.io/0.domain", "tencent.com"),
            ("sync-certs.io/0.extra", r#"{"Http2": "on"}"#),
            ("sync-certs.io/1.adapter", "aliyun_cdn"),
            ("sync-certs.io/1.domain", "aliyun.com"),
            ("sync-certs.io/1._last-applied-digest", "123"),
            ("sync-certs.io/2.extra", r#"{"Http2": "on"}"#),
            ("sync-certs.io/3.adapter", "aliyun_cdn"),
            ("sync-certs.io/3.domain", "aliyun2.com"),
            (
                "sync-certs.io/3._last-applied-digest",
                "406204f6d91f795e17c0f1bb4a84e56cf909773ce7c3a0d47366bf7c4f43760f",
            ),
        ]
        .map(|(k, v)| (k.to_string(), v.to_string())),
    );

    let applyments = Applyment::from_annotations(&map, "cert_content", "cert_key");

    assert_eq!(applyments.len(), 3, "should filter out invalid applyment");

    let ap0 = applyments
        .iter()
        .find(|a| a.idx == "0")
        .expect("should find applyment with id 0");
    let ap1 = applyments
        .iter()
        .find(|a| a.idx == "1")
        .expect("should find applyment with id 1");
    let ap3 = applyments
        .iter()
        .find(|a| a.idx == "3")
        .expect("should find applyment with id 3");

    assert_eq!(ap0.idx, "0");
    assert_eq!(ap0.adapter, "tencent_cloud_cdn");
    assert_eq!(ap3.idx, "3");

    assert_eq!(
        ap0.stale_patch(),
        Some(serde_json::json!({
            "metadata": {
                "annotations": {
                    "sync-certs.io/0._last-applied-digest": "1335a63a63bd6883d3074247453999f2b37d1f48591c294dc6067a2437b31986"
                }
            }
        }))
    );
    assert_eq!(
        ap1.stale_patch(),
        Some(serde_json::json!({
            "metadata": {
                "annotations": {
                    "sync-certs.io/1._last-applied-digest": "2ef69db9a0977b6fef242dc40caa998ad7b733d6263acb962bf472d206e8b2a0"
                }
            }
        }))
    );
    assert_eq!(ap3.stale_patch(), None);
}
