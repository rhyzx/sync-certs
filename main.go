package main

import (
	"context"
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/client-go/kubernetes"
	"k8s.io/client-go/rest"
	"k8s.io/client-go/tools/clientcmd"
	"k8s.io/client-go/util/homedir"
)

func InClusterConfig() *rest.Config {
	config, err := rest.InClusterConfig()
	if err != nil {
		panic(err.Error())
	}
	return config
}
func OutClusterConfig() *rest.Config {
	var kubeconfig *string
	if home := homedir.HomeDir(); home != "" {
		kubeconfig = flag.String("kubeconfig", filepath.Join(home, ".kube", "config"), "(optional) absolute path to the kubeconfig file")
	} else {
		kubeconfig = flag.String("kubeconfig", "", "absolute path to the kubeconfig file")
	}
	flag.Parse()

	// use the current context in kubeconfig
	config, err := clientcmd.BuildConfigFromFlags("", *kubeconfig)
	if err != nil {
		panic(err.Error())
	}
	return config
}

func main() {
	var config *rest.Config
	if _, present := os.LookupEnv("KUBERNETES_SERVICE_HOST"); present {
		config = InClusterConfig()
	} else {
		config = OutClusterConfig()
	}
	// create the clientset
	clientset, err := kubernetes.NewForConfig(config)
	if err != nil {
		panic(err.Error())
	}

	secrets, _ := clientset.CoreV1().Secrets("").List(context.TODO(), metav1.ListOptions{
		FieldSelector: "type=kubernetes.io/tls",
	})

	certClient := NewClient()
	for _, secret := range secrets.Items {
		annotation := secret.ObjectMeta.Annotations["cloud.tencent.com/cdn"]
		if annotation != "" {
			crt := secret.Data["tls.crt"]
			key := secret.Data["tls.key"]
			domains := strings.Split(annotation, ",")

			for _, domain := range domains {
				_, err := certClient.UpdateDomainCert(domain, string(crt), string(key))
				if err != nil {
					fmt.Printf("failed update: %s, %s\n", domain, err)
				} else {
					fmt.Printf("success updated: %s\n", domain)
				}
			}
		}
	}
}
