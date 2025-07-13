use std::ffi::CString;
use std::env;
use std::io::{Result, Write};
use std::fs;
use std::path::Path;


const IPLISTPATH: &str = "/etc/uwupm/iplist.txt";
const PACKAGE_LIST_PATH: &str = "/etc/uwupm/packagelist.txt";

/*
 * E(IP001)     = Unable to locate server
 * W(IP002)     = Unable to add ip to list due to it already existing
 * E(IP003)     = Unable to remove ip from list due to it not existing
 * E(IP004)     = Invalid communication protocol
 * E(SH005)     = Invalid command usage
 * E(FS006)     = Unable to create necessary files in setup
 * E(IP007)     = No available servers
 * W(FS008)
 * */


unsafe extern "C" {
    fn Cpp_Command(cmd: *const libc::c_char) -> i32;
}


fn command(cmd: &str) -> i32{
    unsafe {
        let c_cmd = CString::new(cmd).expect("CString::new failed");
        Cpp_Command(c_cmd.as_ptr())
    }
}


fn add_server(ip: String, force_flag: bool) -> Result<()> {
    let mut ip_list: String = fs::read_to_string(IPLISTPATH)?;

    let found = ip_list.lines().any(|line| line.trim() == ip.trim());

    let clean_ip = ip.trim_start_matches("http://").trim_start_matches("https://")
                    .split(':').next().unwrap_or("");
    let ping_status = command(&format!("ping -c 1 {}", clean_ip));


    // Checking to make sure the ip isn't already in the list, is available and contains either http:// or https://
    if !found && (force_flag || ping_status == 0) && (ip.contains("http://") || ip.contains("https://")) {
        ip_list += &format!("\n{}", ip);
        fs::write(IPLISTPATH, ip_list)?;
        println!("I: Wrote IP to IP list");
    }
    else if !ip_list.contains(&ip) {
        println!("W(IP002): IP already in IP list - Not adding");
    }
    else if !(ip.contains("http://") || ip.contains("https://")) {
        println!("E(IP004): Unable to add IP due to it missing http:// or https:// at the beginning")
    }
    else {
        println!("E(IP001): Unable to reach server. Refusing to add IP");
    }

    Ok(())
}


fn remove_server(ip: String) -> Result<()> {
    let mut ip_list: String = fs::read_to_string(IPLISTPATH)?;

    if ip_list.contains(&ip) {
        ip_list = String::from(ip_list.replace(&ip, ""));
        fs::write(IPLISTPATH, ip_list)?;
        println!("I: Removed IP from IP list");
    }
    else {
        println!("E(IP003): IP not yet in list - Can't remove");
    }
    
    Ok(())
}


fn update() -> Result<()>{
    // curl -o /path/to/directory/desired_filename.ext http://example.com/path/to/file
    println!("I: Reading IP list...");
    let ip_list_string: String = fs::read_to_string(IPLISTPATH)?;

    // Split the list into a vector and filter it for empty indices
    let ip_list: Vec<&str> = ip_list_string.split("\n").filter(|s| !s.trim().is_empty()).collect();

    println!("I: Checking for valid IPs");
    if ip_list.is_empty() {
        println!("E(IP007): No servers in iplist - cannot update. Make sure to run \"uwupm addip [http://, http://][SERVER_IP]:[PORT]\" before doing anything else");
        return Ok(());
    }

    println!("I: Opening local packagelist...");

    if !Path::new(&format!("/home/{}/uwupm_packages/", whoami::username())).exists() {
        println!("W(FS008): Necessary folder for packages doesn't exist. Creating... (Create the folder ~/uwupm_packages if this fails)");
        fs::create_dir_all(&format!("/home/{}/uwupm_packages/", whoami::username()))?;
    }

    if !Path::new(&format!("/home/{}/uwupm_packages/packagelist.txt", whoami::username())).exists() {
        println!("W(FS008): Necessary packagelist file doesn't exist. Creating...");
        fs::File::create(format!("/home/{}/uwupm_packages/packagelist.txt", whoami::username()))?;
    }

    if !Path::new(&format!("/home/{}/uwupm_packages/packagelist_partial.txt", whoami::username())).exists() {
        println!("W(FS008): Necessary packagelist_partial file doesn't exist. Creating...");
        fs::File::create(format!("/home/{}/uwupm_packages/packagelist_partial.txt", whoami::username()))?;
    }

    fs::File::create("/home/root/uwupm_packages/packagelist.txt")?;

    let mut package_list_file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(format!("/home/{}/uwupm_packages/packagelist.txt", whoami::username()))?;

    for i in ip_list{
        println!("I: Adding packagelist from {}...", i);
        command(&format!("curl -o /home/{}/uwupm_packages/packagelist_partial.txt {}/packagelist.txt", whoami::username(), i));
        println!("I: Writing...");
        let contents = fs::read_to_string("/home/root/uwupm_packages/packagelist_partial.txt")?;
        writeln!(package_list_file, "{}", contents)?;
    }

    println!("I: Copying packagelist...");
    fs::copy("/home/root/uwupm_packages/packagelist.txt", PACKAGE_LIST_PATH)?;

    println!("I: Update done!");

    Ok(())
}


fn unknown_command(arg: String) -> Result<()>{
    println!("Unknown command \"{}\"", arg);
    Ok(())
}


fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match &args[1][..] {
            "addip" => {
                if args.len() == 3 {
                    add_server(args[2].clone(), false)
                } else if args.len() == 4 && args.iter().any(|s| s == "--force" || s == "-f") {
                    add_server(args[2].clone(), true)
                } else {
                    println!("E(SH005): Invalid usage. Expected: uwupm addip [IP] [--force]"); 
                    Ok(())
                }
            },
            "removeip" => {
                if args.len() == 3 {
                    remove_server(args[2].clone())
                }
                else {
                    println!("E(SH005): Invalid usage. Expected: uwupm removeip [IP]");
                    Ok(())
                }
            },
            "update" => update(),
            _ => unknown_command(args[1].clone())
        }?;
    }


    Ok(())
}

