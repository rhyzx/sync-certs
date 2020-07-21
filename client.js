const assert = require('assert')
const tencentcloud = require('tencentcloud-sdk-nodejs')
const { SecretId, SecretKey } = process.env
const { Client, Models } = tencentcloud.cdn.v20180606
const { Credential, ClientProfile, HttpProfile } = tencentcloud.common

assert(SecretId, 'SecretId is not provided')
assert(SecretKey, 'SecretKey is not provided')


const credential = new Credential(SecretId, SecretKey)
const httpProfile = new HttpProfile()
httpProfile.endpoint = 'cdn.tencentcloudapi.com'
const clientProfile = new ClientProfile()
clientProfile.httpProfile = httpProfile
const client = new Client(credential, '', clientProfile)

const updateDomainCert = async (domain, crt, key) => {
  const req = new Models.UpdateDomainConfigRequest()

  req.deserialize({
    Domain: domain,
    Https: {
      Switch: 'on',
      CertInfo: {
        Certificate: crt,
        PrivateKey: key,
      },
    },
  })

  await new Promise((resolve, reject) => {
    client.UpdateDomainConfig(req, (err, response) => {
      if (err) reject(err)
      else resolve(response)
    })
  })
}

exports.updateDomainCert = updateDomainCert
