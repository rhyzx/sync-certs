package main

import (
	"os"

	cdn "github.com/tencentcloud/tencentcloud-sdk-go/tencentcloud/cdn/v20180606"
	"github.com/tencentcloud/tencentcloud-sdk-go/tencentcloud/common"
	"github.com/tencentcloud/tencentcloud-sdk-go/tencentcloud/common/profile"
)

type Client struct{ cdn.Client }

func NewClient() *Client {
	SecretId, SecretKey := os.Getenv("SecretId"), os.Getenv("SecretKey")
	credential := common.NewCredential(SecretId, SecretKey)
	cpf := profile.NewClientProfile()
	cpf.HttpProfile.Endpoint = "cdn.tencentcloudapi.com"
	client, _ := cdn.NewClient(credential, "", cpf)
	return &Client{*client}
}

func (client *Client) UpdateDomainCert(domain, cert, key string) (int, error) {
	request := cdn.NewUpdateDomainConfigRequest()
	request.Domain = &domain

	on := "on"
	request.Https = &cdn.Https{
		Switch: &on,
		CertInfo: &cdn.ServerCert{
			Certificate: &cert,
			PrivateKey:  &key,
		},
	}

	_, err := client.UpdateDomainConfig(request)
	return 1, err
	// if _, ok := err.(*errors.TencentCloudSDKError); ok {
	// 	fmt.Printf("An API error has returned: %s", err)
	// }
	// if err != nil {
	// 	panic(err)
	// }
}
