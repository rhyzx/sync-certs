use crate::applyment::Applyment;

mod common;

mod aliyun_cdn;
mod tencent_cloud_cdn;

pub async fn apply(applyment: &Applyment<'_>) -> Result<(), Box<dyn std::error::Error>> {
    match applyment.adapter {
        "aliyun_cdn" => aliyun_cdn::apply(applyment).await,
        "tencent_cloud_cdn" => tencent_cloud_cdn::apply(applyment).await,
        _ => panic!("Unknown adapter"),
    }
}
