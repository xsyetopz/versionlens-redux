module github.com/xxx/yyy

go 1.21

retract v1.1.0 // Published accidentally.

retract [v1.0.0, v1.0.5] // Build broken on some platforms.

require (
	github.com/docker/buildx v0.14.1
	github.com/docker/cli v26.1.3+incompatible
	github.com/docker/cli-docs-tool v0.7.0
	github.com/docker/docker v26.1.3+incompatible
	github.com/docker/go-connections v0.5.0
	github.com/docker/go-units v0.5.0
)

require golang.org/x/term v0.20.0

require (
	k8s.io/api v0.29.2 // indirect
	k8s.io/apimachinery v0.29.2 // indirect
	k8s.io/apiserver v0.29.2 // indirect
	k8s.io/client-go v0.29.2 // indirect
	k8s.io/klog/v2 v2.110.1 // indirect
	k8s.io/kube-openapi v0.0.0-20231010175941-2dd684a91f00 // indirect
	k8s.io/utils v0.0.0-20230726121419-3b25d923346b // indirect
)

exclude github.com/docker/go-units v0.5.0