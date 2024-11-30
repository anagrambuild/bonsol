#!/bin/bash
chown -R root /workspaces/bonsol
chown -R root /usr/local/cargo
exec "$@"