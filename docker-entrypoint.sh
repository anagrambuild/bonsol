#!/bin/bash
set -e
ulimit -s unlimited
exec "$@"