use k8s_openapi::api::core::v1::Secret;
use kube::{api::ListParams, ResourceExt};

mod adapters;
mod applyment;
mod client;

use applyment::Applyment;
use client::SimpleClient;

#[tokio::main]
async fn main() {
    let client = SimpleClient::try_default()
        .await
        .expect("Cluster not found");

    let secrets = client
        .list::<Secret>(
            None,
            &ListParams::default()
                .fields("type=kubernetes.io/tls")
                .labels("sync-certs.io/enable=true"),
        )
        .await
        .expect("Failed to list secrets");

    for secret in &secrets {
        let get_data_str = |key| {
            let bytes = secret.data.as_ref().unwrap().get(key).unwrap();
            String::from_utf8_lossy(&bytes.0)
        };

        let cert = get_data_str("tls.crt");
        let cert_key = get_data_str("tls.key");
        let applyments = Applyment::from_annotations(secret.annotations(), &cert, &cert_key);
        for applyment in applyments {
            if let Some(patch) = applyment.stale_patch() {
                if let Err(err) = adapters::apply(&applyment).await {
                    eprintln!("Failed: {}, due to: {}", applyment.identifier(), err);
                } else {
                    client
                        .patch(secret, patch)
                        .await
                        .expect("Failed to patch secret");

                    println!("Success: {}", applyment.identifier());
                }
            } else {
                println!("Skipped: {}", applyment.identifier());
            }
        }
    }
}
