# UwU-Package Manager

## This is a Package Manager for KawaiiOS and other Linux distros

## Building:
1. Run `rustup target add x86_64-unknown-linux-musl i686-unknown-linux-musl armv7-unknown-linux-musl` (You only need the ones you're actually compiling for)
1. Make sure you have the musl toolchains installed: `sudo pacman -S musl` on Arch and `sudo apt install musl-tools` on Debian and Ubuntu-based stuff
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

#### (IPxxx) = IP / Network / Server error
#### (SHxxx) = Shell / Bash error
#### (FSxxx) = Filesystem error
#### (DWxxx) = Downloading error

### Actual error codes:

- **(IP001)** - Unable to locate server
- **(IP002)** - Unable to add IP to IP list due to it already existing
- **(IP003)** - Unable to remove IP from IP list due to it not existing
- **(IP004)** - Invalid communication protocol (missing http or https in the name)
- **(SH005)** - Invalid command usage
- **(FS006)** - Unable to create necessary files in setup
- **(IP007)** - No available servers to fetch from
- **(FS008)** - A necessary folder doesn't exist
- **(IP009)** - Unable to find a queued package on any servers
- **(DW010)** - A Thread was unable to download a certain package
- **(FS011)** - There's a missing uwupm-install.sh file in a package that's getting installed


### *Warnings* and *Errors*:
Errors are prefixed by an E while warnings are prefixed with W. Warnings can usually be resolved by the program itself without crash while Errors cause a crash

### Common error codes and how to fix them:
1. E(IP004) - Attempt at adding a Server IP to the IP list without http:// or https:// written out. 
- a) Fix: Add http:// or https:// to the beginning of the IP (depending on which one is appropriate)

2. E(IP001) - Unable to reach server when adding it to the IP list
- a) Fix: Add -f or --force to the command
- b) Fix: Wait for the server to turn back on again if it's shut off and verify the IP

3. W(FS008) - Doesn't have to be resolved

4. E(SH005) - You typed an invalid command
- a) Fix: Verify that you're typing the correct command

5. E(IP007) - There aren't any registered servers *or* no servers in the IP list are responding
- a) Fix: Add an IP to your IP list
- b) Fix: Wait and see if a server turn back on

6. E(IP009) - A queued package can't be found on any known servers 
- a) Fix: Try updating
- b) Fix: Add any servers that are know to have the package
