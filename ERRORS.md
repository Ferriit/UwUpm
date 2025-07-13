# Error codes:

### (IPxxx) = IP / Network / Server error
### (SHxxx) = Shell / Bash error
### (FSxxx) = Filesystem error

## Actual error codes:
- **(IP001)** - Unable to locate server
- **(IP002)** - Unable to add IP to IP list due to it already existing
- **(IP003)** - Unable to remove IP from IP list due to it not existing
- **(IP004)** - Invalid communication protocol (missing http or https in the name)
- **(SH005)** - Invalid command usage
- **(FS006)** - Unable to create necessary files in setup
- **(IP007)** - No available servers to fetch from
- **(FS008)** - A necessary folder doesn't exist


## *Warnings* and *Errors*:
Errors are prefixed by an E while warnings are prefixed with W. Warnings can usually be resolved by the program itself without crash while Errors cause a crash

## Common error codes and how to fix them:
1. E(IP004) - Attempt at adding a Server IP to the IP list without http:// or https:// written out. 
a) Fix: Add http:// or https:// to the beginning of the IP (depending on which one is appropriate)

2. E(IP001) - Unable to reach server when adding it to the IP list
a) Fix: Add -f or --force to the command
b) Fix: Wait for the server to turn back on again if it's shut off and verify the IP

3. W(FS008) - Doesn't have to be resolved

4. E(SH005) - You typed an invalid command
a) Fix: Verify that you're typing the correct command

5. E(IP007) - There aren't any registered servers *or* no servers in the IP list are responding
a) Fix: Add an IP to your IP list
b) Fix: Wait and see if a server turn back on

