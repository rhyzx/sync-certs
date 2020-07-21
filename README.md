
## Usage


### Install
```
helm install sync-qcloud-cdn-cert https://github.com/yiiz/sync-qcloud-cdn-cert/archive/chart.tar.gz --set secrets.SecretId=<QCloudSecretId>,secrets.SecretKey=<QCloudSecretKey>
```

### Add annotation to TLS Secret(eg. Secrets generated by CertManager)
```
metadata:
  name: example-tls
  annotations:
    cloud.tencent.com/cdn: cdn1.example.com,cdn2.example.com
```
