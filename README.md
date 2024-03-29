## Usage

### Install
```sh
helm install my-sync-certs --repo https://rhyzx.github.io/sync-certs sync-certs --set env.SECRET_ID=$TENCENT_SECRET_ID,env.SECRET_KEY=$TENCENT_SECRET_KEY
```

### Add labels and annotations to TLS Secret(eg. Secrets generated by CertManager)
```yml
kind: Secret
apiVersion: v1
type: kubernetes.io/tls
metadata:
  name: example-tls
  labels:
    sync-certs.io/enable: 'true'
  annotations:
    sync-certs.io/0.adapter: tencent_cloud_cdn
    sync-certs.io/0.domain: test.example.com
```

## Tips
### Multiple domains+API keys

Install with
```sh
--set env.TENCENT_SECRET_ID=$TENCENT_SECRET_ID,env.ALIYUN_ACCESS_KEY_ID=$ALIYUN_ACCESS_KEY_ID,OTHERS…
```

```yaml
metadata:
  annotations:
    sync-certs.io/0.adapter: tencent_cloud_cdn
    sync-certs.io/0.env-prefix: TENCENT_
    sync-certs.io/0.domain: test.example.com
    sync-certs.io/1.adapter: aliyun_cdn
    sync-certs.io/1.env-prefix: ALIYUN_
    sync-certs.io/1.domain: test2.example.com
```

### Enable HTTP2/HSTS for Tencent Cloud CDN
Cause Tencent Cloud will reset HTTPS/HSTS settings after updating, 
an `extra` **JSON** field can be used.

```yaml
metadata:
  annotations:
    sync-certs.io/0.extra: '{"Http2": "on", "Hsts": {"Switch":"on", "MaxAge": 31536000}}'
```

## Adapters
| Name              | Env requires                                     | Extra                                                          |
| ----------------- | ------------------------------------------------ | -------------------------------------------------------------- |
| tencent_cloud_cdn | {PREFIX}SECRET_ID, {PREFIX}SECRET_KEY            | [link](https://cloud.tencent.com/document/api/228/30987#Https) |
| aliyun_cdn        | {PREFIX}ACCESS_KEY_ID, {PREFIX}ACCESS_KEY_SECRET | N/A                                                            |


