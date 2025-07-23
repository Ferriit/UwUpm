# UwU-Package Manager

## This is a Package Manager for KawaiiOS and other Linux distros

## Building:
1. Run `make ARCH=[arm/x86/x64]` to choose whether to compile for Arm, x86 (32-bit) or x64 (x86_64, 64-bit)
1. Run `make` in the root of the repository
1. Run `chmod +x target/debug/uwupm`
1. Run `./src/setup.sh`
1. Move *target/debug/uwupm* to */bin*

## Usage:
Currently UwUPM just supports updating and adding and removing server IPs. But this is a small guide of how it's meant to work:
- Add server IPs to fetch packages from     `uwupm addip [http/https]://[IP]:[PORT (optional)] [--force/-f]` *Note:* It's possible that uwupm refuses if the server is unreachable currently. If that's the case and you still want to add the server, add `--force` or `-f` 
- Remove untrusted or unwanted servers      `uwupm removeip [same ip you used to register it]`
*Note:* The servers are stored in /etc/uwupm/iplist.txt
- Update package list                       `uwupm update`
- Install packages                          `uwupm install [package]`   *Note:* It scans *all* listed servers. So sometimes it might find multiple and you get to choose which one to download from
- Remove packages                           `uwupm remove [package]`
- Upgrade packages                          `uwupm upgrade`

## Errors:
[Guide to error codes](ERRORS.md)
