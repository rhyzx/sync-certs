use std::fmt::Debug;

use k8s_openapi::NamespaceResourceScope;
use kube::{
    api::{ListParams, Patch, PatchParams},
    Api, Client, Resource, ResourceExt,
};

// https://github.com/kube-rs/kube/issues/1030
// https://github.com/kube-rs/kube/issues/1032
pub struct SimpleClient(Client);
impl SimpleClient {
    pub async fn try_default() -> Result<Self, kube::Error> {
        let client = Client::try_default().await?;
        Ok(Self(client))
    }

    fn api<K>(&self, ns: Option<&str>) -> Api<K>
    where
        K: Resource<Scope = NamespaceResourceScope>,
        <K as Resource>::DynamicType: Default,
    {
        match ns {
            Some(ns) => Api::<K>::namespaced(self.0.clone(), ns),
            None => Api::<K>::all(self.0.clone()),
        }
    }

    pub async fn list<K>(
        &self,
        ns: Option<&str>,
        lp: &ListParams,
    ) -> Result<kube::core::ObjectList<K>, kube::Error>
    where
        K: Resource<Scope = NamespaceResourceScope>,
        <K as Resource>::DynamicType: Default,
        K: k8s_openapi::serde::de::DeserializeOwned,
        K: Clone,
        K: Debug,
    {
        self.api(ns).list(lp).await
    }

    pub async fn patch<K>(&self, resource: &K, patch: serde_json::Value) -> Result<K, kube::Error>
    where
        K: Resource<Scope = NamespaceResourceScope>,
        <K as Resource>::DynamicType: Default,
        K: k8s_openapi::serde::de::DeserializeOwned,
        K: Clone,
        K: Debug,
    {
        let name = resource.name_unchecked();
        let ns = resource.namespace().unwrap();

        let patch = Patch::Merge(patch);
        self.api(Some(&ns))
            .patch(&name, &PatchParams::default(), &patch)
            .await
    }
}
