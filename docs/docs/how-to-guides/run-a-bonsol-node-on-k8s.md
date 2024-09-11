# How to Run a Bonsol Node on Kubernetes
Bonsol has a fully featured docker image and helm chart that can be used to run a Bonsol node on kubernetes. This guide will show you how to run a Bonsol node on kubernetes using the helm chart.

## Prerequisites
* A kubernetes cluster
* Helm installed
* A keypair for the node, you need some SOL to pay for the transactions
* A Dragons mouth compatible rpc provider endpoint [Dragons Mouth Docs](https://docs.triton.one/project-yellowstone/dragons-mouth-grpc-subscriptions) click here to get one from [Triton One](https://triton.one/triton-rpc/)


## Kublet/Docker requirments
The bonsol code makes use of a c++ groth16 witness generator, this thing can easily blow your stack. You will need to make sure the the kublet will respect changing the containers maximum ulimit settings such as the stack size.
This will vary from k8s provider to k8s provider, but you can check the kubelet docs for your provider to see how to up the maxiumum ulimits.
I know it sounds crazy but `ulimit -s unlimited` is the best bet right now. We hope to remove this requirement in the future.

## Installing the Helm Chart
First we need to install the helm chart. You can do this with the following command. But you will first need a values file.

```bash
helm install --set-file signer_config.local_signer_keypair_content=<your node keypair> bonsol ./charts/bonsol-node -f ./charts/bonsol-node/secret-values.yaml -n default
```
 
### Values File
The values file is a yaml file that contains the configuration for the helm chart. Here is an example of a values file (annotated with comments).

```yaml
image:
  repository: ghcr.io/anagrambuild/bonsol
  pullPolicy: IfNotPresent
  tag: <version you want to use> //you should run the cuda line of versions to take advantage of the gpu
replicaCount: 1 //number of pods to run feel free to run more than one but you should have a different keypair for each pod we need clarity on how multiple sunscribers work on the grpc stream before we reccomend this
max_input_size_mb: 10 //max size of the input in mb, if greater than this the node will reject the request
image_download_timeout_secs: 60 //timeout for downloading the image from a deployment, if the image is not available in the time specified the node will reject the request
input_download_timeout_secs: 60 //timeout for downloading the input from a url based input type, if the input is not available in the time specified the node will reject the request
maximum_concurrent_proofs: 100 // max number of proofs to generate at once, if this is exceeded the node will reject the request this will be basedon your capacity
max_image_size_mb: 4 //max size of the image in mb, if greater than this the node will reject the deployment
image_compression_ttl_hours: 24 // how long to keep any individual image in hot memory cache before purging it and having to load from disk
env: "devnet"
transaction_sending_config:
  type: "Rpc"
  rpc_url: "http://{your solana rpc}"
ingester_config:
  type: "GrpcSubscription" //we also support block subscriptions friom a configred validator but we reccomend grpc subscriptions
  grpc_url: "http://{your yellow stone grpc}"
  token: "your token here"
  connection_timeout_secs: 10
  timeout_secs: 10
signer_config:
  type: "KeyPairFile"
  path: "/opt/bonsol/keys/signer.json" 
risc0_image_folder: "/opt/bonsol/risc0_images"
metrics: true // enables prometheus metrics on port 9000
```
With all that configured your node will log a few startup lines and then start listenting for requests.