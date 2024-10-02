# Bonsol prover with tester

Chart deploys bonsol prover and its tester which triggers proof generation

## Introduction

This chart bootstraps a [bonsol prover](https://github.com/anagrambuild/bonsol) deployment on a [Kubernetes](https://kubernetes.io) cluster using the [Helm](https://helm.sh) package manager.

## Prerequisites

- Kubernetes 1.19+
- Helm 3.2.0+
- solana-keygen

## Creating Solana key pair

Generate Solana key pair:

`solana-keygen new -o keypair.json

## Creating Solana RPC endpoint

In order to run Bonsol node you need to have access to Dragons mouth compatible rpc provider endpoint Dragons Mouth Docs.
Click here to get one from [Triton One](https://triton.one/triton-rpc/)

## Installing the Chart

Create my_values.yaml file:

```
signer:
  keypair: <your keypair value from keypair.json file>
rpc:
  url: <your Solana RPC URL>
  token: <your token to access RPC service>

```

To install the chart with the release name `my-release`:

```console
helm repo add zkcharts https://zerocomputing.github.io/helm/
helm install my-release zkcharts/bonsol-node -f my_values.yaml
```

## Uninstalling the Chart

To uninstall/delete the `my-release` deployment:

```console
helm delete my-release
```

The command removes all the Kubernetes components associated with the chart and deletes the release.

## Parameters

### Global parameters

| Name                   | Description                                                 | Value                          |
| ---------------------- | ----------------------------------------------------------- | ------------------------------ |
| `signer.path`          | file path where Solana key will be mounted in the container | `/opt/bonsol/keys/signer.json` |
| `signer.keypair`       | user's Solana key pair                                      | `""`                           |
| `provernode.rpc.url`   | URL of Solana RPC service                                   | `""`                           |
| `provernode.rpc.token` | token granting access to RPC service                        | `""`                           |


### Provernode parameters

| Name                                | Description                                                                                | Value                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| ----------------------------------- | ------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `provernode.nameOverride`           | string to partially override provernode.fullname template (will maintain the release name) | `nil`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `provernode.fullnameOverride`       | string to fully override provernode.fullname template                                      | `nil`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `provernode.image.repository`       | bonsol container image registry                                                            | `ghcr.io/anagrambuild/bonsol-relay@sha256`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `provernode.image.tag`              | provernode image tage                                                                      | `a44f47818cbbc0bafcd27ae46226fdb2271662c3a070df6a1a84d5cc6b031a6d`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `provernode.image.pullPolicy`       | provernode image pull policy                                                               | `IfNotPresent`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `provernode.image.imagePullSecrets` | provernode image pull secrets                                                              | `[]`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `provernode.replicaCount`           | Desired number of prover node replicas                                                     | `1`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `provernode.podAnnotations`         | annotations to add to pod object                                                           | `{}`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `provernode.podLabels`              | labels to add to pod object                                                                | `{}`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `provernode.podSecurityContext`     | podSecurityContext to add to pod object                                                    | `{}`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `provernode.volumes`                | a list of volumes to be added to the pod                                                   | `[]`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `provernode.volumeMounts`           | a list of volume mounts to be added to the pod                                             | `[]`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `provernode.nodeSelector`           | node labels for pod assignment                                                             | `{}`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `provernode.tolerations`            | tolerations for pod assignment                                                             | `{}`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `provernode.affinity`               | affinity for pod assignment                                                                | `{}`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `provernode.resources`              | the resources limits and/or requests for the container                                     | `{}`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `provernode.env`                    | an map to be converted as environment variables for the container                          | `{}`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `provernode.config.filename`        | an absolute path for prover node config file inside the container                          | `/opt/bonsol/Node.toml`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| `provernode.config.values`          | config file content                                                                        | `risc0_image_folder = "/opt/bonsol/risc0_images"
max_input_size_mb = 10
image_download_timeout_secs = 60
input_download_timeout_secs = 60
maximum_concurrent_proofs = 1
max_image_size_mb = 4
image_compression_ttl_hours = 24
env = "dev"

[transaction_sender_config]
  Rpc = { rpc_url = "{{ .Values.provernode.rpc.url }}/{{ .Values.provernode.rpc.token }}" }
[signer_config]
  KeypairFile = { path = "{{ .Values.signer.path }}" }
[ingester_config]
  GrpcSubscription = { grpc_url = "{{ .Values.provernode.rpc.url }}", token = "{{ .Values.provernode.rpc.token }}", connection_timeout_secs = 10, timeout_secs = 10 }
` |


### Tester parameters

| Name                            | Description                                                                            | Value                                   |
| ------------------------------- | -------------------------------------------------------------------------------------- | --------------------------------------- |
| `tester.enabled`                | whether tester should be deployed or not                                               | `true`                                  |
| `tester.nameOverride`           | string to partially override tester.fullname template (will maintain the release name) | `nil`                                   |
| `tester.fullnameOverride`       | string to fully override tester.fullname template                                      | `nil`                                   |
| `tester.image.repository`       | bonsol container image registry                                                        | `docker.io/zerocomputing/bonsol-tester` |
| `tester.image.tag`              | tester image tage                                                                      | `0.2.0`                                 |
| `tester.image.pullPolicy`       | tester image pull policy                                                               | `IfNotPresent`                          |
| `tester.image.imagePullSecrets` | tester image pull secrets                                                              | `[]`                                    |
| `tester.replicaCount`           | Desired number of prover node replicas                                                 | `1`                                     |
| `tester.podAnnotations`         | annotations to add to pod object                                                       | `{}`                                    |
| `tester.podLabels`              | labels to add to pod object                                                            | `{}`                                    |
| `tester.podSecurityContext`     | podSecurityContext to add to pod object                                                | `{}`                                    |
| `tester.volumes`                | a list of volumes to be added to the pod                                               | `[]`                                    |
| `tester.volumeMounts`           | a list of volume mounts to be added to the pod                                         | `[]`                                    |
| `tester.nodeSelector`           | node labels for pod assignment                                                         | `{}`                                    |
| `tester.tolerations`            | tolerations for pod assignment                                                         | `{}`                                    |
| `tester.affinity`               | affinity for pod assignment                                                            | `{}`                                    |
| `tester.resources`              | the resources limits and/or requests for the container                                 | `{}`                                    |
| `tester.env`                    | an map to be converted as environment variables for the container                      | `{}`                                    |


Specify each parameter using the `--set key=value[,key=value]` argument to `helm install`. For example,

```console
helm install my-release \
  --set provider=aws .
```

Alternatively, a YAML file that specifies the values for the parameters can be provided while installing the chart. For example,

```console
helm install my-release -f values.yaml .
```

> **Tip**: You can use the default [values.yaml](values.yaml)

## Configuration and installation details

```console
helm install my-release \
  --set provider=aws \
  --set aws.zoneType=public \
  --set txtOwnerId=HOSTED_ZONE_IDENTIFIER \
  --set domainFilters[0]=HOSTED_ZONE_NAME \
  .
```

