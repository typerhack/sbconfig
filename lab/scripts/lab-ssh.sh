#!/bin/bash
# lab/scripts/lab-ssh.sh
# SSH into the sbconfig lab container
exec ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -p 2222 root@localhost
