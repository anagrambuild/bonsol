# Manually Provision a Bonsol Node
Bonsol has a fully featured docker image and helm chart that can be used to run a Bonsol node on kubernetes. For more information on how to run a Bonsol node on kubernetes check out the [Run a Bonsol Node on Kubernetes](/docs/how-to/run-a-bonsol-node-on-k8s) guide. But for many of use we want to feel the heat of the cpu, and tast the sour air of hot solder. We need to feed the bare metal between our fingertips.

## Prerequisites
* A keypair for the node, you need some SOL to pay for the transactions
* A Dragons mouth compatible rpc provider endpoint [Dragons Mouth Docs](https://docs.triton.one/project-yellowstone/dragons-mouth-grpc-subscriptions) click here to get one from [Triton One](https://triton.one/triton-rpc/)
* Docker on your local machine (not required on the node)
* The node will do better if it has a gpu with cuda installed, which will require nvidia drivers and tools. 

:::note
Ansible role coming soon
:::

## Installing Deps
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain 1.80.0 -y
```
Ensure cargo is on the path

On your local machine you will need to run a docker imahge to get the neede groth16 witness generator and snark binary. Ensure you have a recent version of docker installed. This docker file copies the binaries from the docker image to the local machine.
```bash
docker build -f setup.dockerfile -o relay .
```
You will have a director called snark with the binaries in it. You need to copy these binaries to the node and remember the path to the snark directory.
```bash
# on the node
sudo mkdir -p /opt/bonsol/stark
sudo chown -R ubuntu /opt/bonsol/stark
sudo mkdir -p /opt/bonsol/keys
sudo chown -R ubuntu /opt/bonsol/keys
```

```bash
# on your local computer

scp -i <your_ssh_key> -r stark/* <node_user>@<node ip>:/opt/bonsol/stark
```
You will put the path in the `stark_compression_tools_path` in the config file.

## Upload the keypair to the node 
You will need to upload the keypair to the node. 
```bash
scp -r <keypair path> <node ip>:/opt/bonsol/keys/
```
you will put the path in the below section of the config file.
```toml
[signer_config]
  KeypairFile = { path = "<your keypair path>" }
```


## Installing Bonsol
```bash
git clone --depth=1 https://github.com/anagrambuild/bonsol.git bonsol
cd bonsol/
cargo build -f cuda --release
```

## Configuring the Node
You will need to create a config file for the node. The config file is a toml file that contains the configuration for the node.
```bash
touch Node.toml
```
Here is an example of a config file.

```toml
risc0_image_folder = "/opt/bonsol/risc0_images"
max_input_size_mb = 10
image_download_timeout_secs = 60
input_download_timeout_secs = 60
maximum_concurrent_proofs = 1
max_image_size_mb = 4
image_compression_ttl_hours = 24
env = "dev"
[transaction_sender_config]
  Rpc = { rpc_url = "http://localhost:8899" }
[signer_config]
  KeypairFile = { path = "/opt/bonsol/keys/signer.json" }
[ingester_config]
  RpcBlockSubscription = { wss_rpc_url = "ws://localhost:8900" }

risc0_image_folder = "/opt/bonsol/risc0_images"
max_input_size_mb = 10
image_download_timeout_secs = 60
input_download_timeout_secs = 60
maximum_concurrent_proofs = 1
max_image_size_mb = 4
image_compression_ttl_hours = 24
env = "dev"
stark_compression_tools_path = "<the path to the stark directory>" 
missing_image_strategy = "DownloadAndClaim"
[metrics_config]
  Prometheus = {}
[ingester_config]
  GrpcSubscription = { grpc_url = "<your dragons mouth grpc endpoint>", token = "<your token>", connection_timeout_secs = 10, timeout_secs = 10 }
[transaction_sender_config]
  Rpc = { rpc_url = "<your solana rpc endpoint>" }
[signer_config]
  KeypairFile = { path = "<your keypair path>" }
```

## Running the Node
You can run the node with the following command.
```bash
ulimit -s unlimited //this is required for the c++ groth16 witness generator it will blow your stack without a huge stack size
bonsol -f Node.toml
```
### Runnig the Node with systemd
You can use the following systemd service file to run the node.
```toml
[Unit]
Description=Solana Validator
After=network.target
StartLimitIntervalSec=0

[Service]
Type=simple
User=ubuntu
Restart=always
RestartSec=1
LimitSTACK=infinity
LimitNOFILE=1000000
LogRateLimitIntervalSec=0
WorkingDirectory=/home/ubuntu/bonsol
ExecStart=/home/ubuntu/bonsol/target/release/relay -f Node.toml

# Create BACKTRACE only on panics
Environment="RUST_BACKTRACE=1"
Environment="RUST_LIB_BACKTRACE=0"

[Install]
WantedBy=multi-user.target
```
You will need to copy this file to `/etc/systemd/system/bonsol.service` and then run the following command.
After that you can reload the systemd daemon and start the service with the following command.
```bash
systemctl daemon-reload
systemctl start bonsol
```

## Metrics
The node will expose prometheus metrics on port 9000 by default. You can use a number of tools to scrappe those metrics but here is an example Grafana alloy config.

The full config is verbose but here is the important parts.
```
prometheus.scrape "bonsol" {
  targets    = [{
    __address__ = "127.0.0.1:9000",
  }]
  forward_to = [prometheus.remote_write.metrics_service.receiver]
}
```
The below config will scrape the metrics from the node and forward them to the prometheus remote write endpoint. This will also monitor the linux node.

```
prometheus.exporter.self "alloy_check" { }


prometheus.scrape "bonsol" {
  targets    = [{
    __address__ = "127.0.0.1:9000",
  }]
  forward_to = [prometheus.remote_write.metrics_service.receiver]
}


discovery.relabel "alloy_check" {
  targets = prometheus.exporter.self.alloy_check.targets

  rule {
    target_label = "instance"
    replacement  = constants.hostname
  }

  rule {
    target_label = "alloy_hostname"
    replacement  = constants.hostname
  }

  rule {
    target_label = "job"
    replacement  = "integrations/alloy-check"
  }
}

discovery.relabel "integrations_node_exporter" {
  targets = prometheus.exporter.unix.integrations_node_exporter.targets

  rule {
    target_label = "instance"
    replacement  = constants.hostname
  }

  rule {
    target_label = "job"
    replacement = "integrations/node_exporter"
  }
}

prometheus.exporter.unix "integrations_node_exporter" {
  disable_collectors = ["ipvs", "btrfs", "infiniband", "xfs", "zfs"]

  filesystem {
    fs_types_exclude     = "^(autofs|binfmt_misc|bpf|cgroup2?|configfs|debugfs|devpts|devtmpfs|tmpfs|fusectl|hugetlbfs|iso9660|mqueue|nsfs|overlay|proc|procfs|pstore|rpc_pipefs|securityfs|selinuxfs|squashfs|sysfs|tracefs)$"
    mount_points_exclude = "^/(dev|proc|run/credentials/.+|sys|var/lib/docker/.+)($|/)"
    mount_timeout        = "5s"
  }

  netclass {
    ignored_devices = "^(veth.*|cali.*|[a-f0-9]{15})$"
  }

  netdev {
    device_exclude = "^(veth.*|cali.*|[a-f0-9]{15})$"
  }
}

prometheus.scrape "integrations_node_exporter" {
  targets    = discovery.relabel.integrations_node_exporter.output
  forward_to = [prometheus.relabel.integrations_node_exporter.receiver]
  scrape_interval = "120s"
}

prometheus.relabel "integrations_node_exporter" {
  forward_to = [prometheus.remote_write.metrics_service.receiver]

  rule {
    source_labels = ["__name__"]
    regex         = "up|node_arp_entries|node_boot_time_seconds|node_context_switches_total|node_cpu_seconds_total|node_disk_io_time_seconds_total|node_disk_io_time_weighted_seconds_total|node_disk_read_bytes_total|node_disk_read_time_seconds_total|node_disk_reads_completed_total|node_disk_write_time_seconds_total|node_disk_writes_completed_total|node_disk_written_bytes_total|node_filefd_allocated|node_filefd_maximum|node_filesystem_avail_bytes|node_filesystem_device_error|node_filesystem_files|node_filesystem_files_free|node_filesystem_readonly|node_filesystem_size_bytes|node_intr_total|node_load1|node_load15|node_load5|node_md_disks|node_md_disks_required|node_memory_Active_anon_bytes|node_memory_Active_bytes|node_memory_Active_file_bytes|node_memory_AnonHugePages_bytes|node_memory_AnonPages_bytes|node_memory_Bounce_bytes|node_memory_Buffers_bytes|node_memory_Cached_bytes|node_memory_CommitLimit_bytes|node_memory_Committed_AS_bytes|node_memory_DirectMap1G_bytes|node_memory_DirectMap2M_bytes|node_memory_DirectMap4k_bytes|node_memory_Dirty_bytes|node_memory_HugePages_Free|node_memory_HugePages_Rsvd|node_memory_HugePages_Surp|node_memory_HugePages_Total|node_memory_Hugepagesize_bytes|node_memory_Inactive_anon_bytes|node_memory_Inactive_bytes|node_memory_Inactive_file_bytes|node_memory_Mapped_bytes|node_memory_MemAvailable_bytes|node_memory_MemFree_bytes|node_memory_MemTotal_bytes|node_memory_SReclaimable_bytes|node_memory_SUnreclaim_bytes|node_memory_ShmemHugePages_bytes|node_memory_ShmemPmdMapped_bytes|node_memory_Shmem_bytes|node_memory_Slab_bytes|node_memory_SwapTotal_bytes|node_memory_VmallocChunk_bytes|node_memory_VmallocTotal_bytes|node_memory_VmallocUsed_bytes|node_memory_WritebackTmp_bytes|node_memory_Writeback_bytes|node_netstat_Icmp6_InErrors|node_netstat_Icmp6_InMsgs|node_netstat_Icmp6_OutMsgs|node_netstat_Icmp_InErrors|node_netstat_Icmp_InMsgs|node_netstat_Icmp_OutMsgs|node_netstat_IpExt_InOctets|node_netstat_IpExt_OutOctets|node_netstat_TcpExt_ListenDrops|node_netstat_TcpExt_ListenOverflows|node_netstat_TcpExt_TCPSynRetrans|node_netstat_Tcp_InErrs|node_netstat_Tcp_InSegs|node_netstat_Tcp_OutRsts|node_netstat_Tcp_OutSegs|node_netstat_Tcp_RetransSegs|node_netstat_Udp6_InDatagrams|node_netstat_Udp6_InErrors|node_netstat_Udp6_NoPorts|node_netstat_Udp6_OutDatagrams|node_netstat_Udp6_RcvbufErrors|node_netstat_Udp6_SndbufErrors|node_netstat_UdpLite_InErrors|node_netstat_Udp_InDatagrams|node_netstat_Udp_InErrors|node_netstat_Udp_NoPorts|node_netstat_Udp_OutDatagrams|node_netstat_Udp_RcvbufErrors|node_netstat_Udp_SndbufErrors|node_network_carrier|node_network_info|node_network_mtu_bytes|node_network_receive_bytes_total|node_network_receive_compressed_total|node_network_receive_drop_total|node_network_receive_errs_total|node_network_receive_fifo_total|node_network_receive_multicast_total|node_network_receive_packets_total|node_network_speed_bytes|node_network_transmit_bytes_total|node_network_transmit_compressed_total|node_network_transmit_drop_total|node_network_transmit_errs_total|node_network_transmit_fifo_total|node_network_transmit_multicast_total|node_network_transmit_packets_total|node_network_transmit_queue_length|node_network_up|node_nf_conntrack_entries|node_nf_conntrack_entries_limit|node_os_info|node_sockstat_FRAG6_inuse|node_sockstat_FRAG_inuse|node_sockstat_RAW6_inuse|node_sockstat_RAW_inuse|node_sockstat_TCP6_inuse|node_sockstat_TCP_alloc|node_sockstat_TCP_inuse|node_sockstat_TCP_mem|node_sockstat_TCP_mem_bytes|node_sockstat_TCP_orphan|node_sockstat_TCP_tw|node_sockstat_UDP6_inuse|node_sockstat_UDPLITE6_inuse|node_sockstat_UDPLITE_inuse|node_sockstat_UDP_inuse|node_sockstat_UDP_mem|node_sockstat_UDP_mem_bytes|node_sockstat_sockets_used|node_softnet_dropped_total|node_softnet_processed_total|node_softnet_times_squeezed_total|node_systemd_unit_state|node_textfile_scrape_error|node_time_zone_offset_seconds|node_timex_estimated_error_seconds|node_timex_maxerror_seconds|node_timex_offset_seconds|node_timex_sync_status|node_uname_info|node_vmstat_oom_kill|node_vmstat_pgfault|node_vmstat_pgmajfault|node_vmstat_pgpgin|node_vmstat_pgpgout|node_vmstat_pswpin|node_vmstat_pswpout|process_max_fds|process_open_fds"
    action        = "keep"
  }
}


prometheus.scrape "alloy_check" {
  targets    = discovery.relabel.alloy_check.output
  forward_to = [prometheus.relabel.alloy_check.receiver]

  scrape_interval = "120s"
}

prometheus.relabel "alloy_check" {
  forward_to = [prometheus.remote_write.metrics_service.receiver]

  rule {
    source_labels = ["__name__"]
    regex         = "(prometheus_target_sync_length_seconds_sum|prometheus_target_scrapes_.*|prometheus_target_interval.*|prometheus_sd_discovered_targets|alloy_build.*|prometheus_remote_write_wal_samples_appended_total|process_start_time_seconds)"
    action        = "keep"
  }
}

prometheus.remote_write "metrics_service" {
  endpoint {
    url = "https://<prometheus url>/api/prom/push"

    basic_auth {
      username = "<prometheus username>"
      password = "<prometheus password>"
    }
  }
}

loki.write "grafana_cloud_loki" {
  endpoint {
    url = "https://<loki url>/loki/api/v1/push"

    basic_auth {
      username = "<loki username>"
      password = "<loki password>"
    }
  }
}
```

Installing Alloy is out of the scope of this guide but you can follow the [grafana cloud docs](https://grafana.com/docs/alloy/latest/set-up/install/linux/) to install it.