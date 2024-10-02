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
Refer to [this documentation](https://github.com/anagrambuild/bonsol/tree/main/charts) on how to install the Helm chart.
