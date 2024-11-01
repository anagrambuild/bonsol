#!/bin/bash
sudo chown -R 1001:121 /workspaces/bonsol
sudo chown -R 1001:121 /usr/local/cargo
sudo exec "$@"