#!/bin/bash

echo "I: Creating /etc/uwupm, packagelist.txt and iplist.txt"

sudo mkdir -p /etc/uwupm
sudo touch /etc/uwupm/iplist.txt
sudo chmod 644 /etc/uwupm/iplist.txt

sudo touch /etc/uwupm/packagelist.txt
sudo chmod 644 /etc/uwupm/packagelist.txt

sudo mkdir ~/uwupm_packages

echo "I: Setup complete."


