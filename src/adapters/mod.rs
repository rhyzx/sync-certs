use crate::applyment::Applyment;

mod tencent_cloud_cdn;

pub async fn apply(applyment: &Applyment<'_>) -> Result<(), Box<dyn std::error::Error>> {
    match applyment.adapter {
        "tencent_cloud_cdn" => tencent_cloud_cdn::apply(applyment).await,
        _ => panic!("Unknown adapter"),
    }
}
