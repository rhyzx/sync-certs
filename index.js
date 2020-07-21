const atob = (base64) => `${Buffer.from(base64, 'base64')}`
const k8s = require('@kubernetes/client-node')
const { updateDomainCert } = require('./client')

const kc = new k8s.KubeConfig()
kc.loadFromDefault()

const watcher = new k8s.Watch(kc)

watcher.watch(
  '/api/v1/secrets',
  {
    // allowWatchBookmarks: true,
    fieldSelector: 'type=kubernetes.io/tls',
    // labelSelector: 'custom-key=custom-value',
  },
  (type, obj) => {
    if (type !== 'ADDED' && type !== 'MODIFIED') return

    const annotation = obj.metadata.annotations['cloud.tencent.com/cdn']
    const crt = atob(obj.data['tls.crt'])
    const key = atob(obj.data['tls.key'])
    if (!annotation || !crt || !key) return

    const domains = annotation.split(',')
    domains.forEach(async (d) => {
      console.log(`start update: ${d}`)
      try {
        await updateDomainCert(d, crt, key)
        console.log(`success update: ${d}`)
      } catch (e) {
        console.error(`failed update: ${d}, reason: ${e.message}`)
      }
    })
  },
  (err) => {
    console.error(err)
  }
)
// .then((req) => {
//   setTimeout(() => req.abort(), 10 * 1000)
// }
