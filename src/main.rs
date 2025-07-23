use std::ffi::CString;
use std::env;
use std::io::{Result, Write};
use std::fs;
use std::path::Path;


const IPLIST_PATH: &str = "/etc/uwupm/iplist.txt";
const PACKAGE_LIST_PATH: &str = "/etc/uwupm/packagelist.txt";
const SAVE_PATH: &str = "/etc/uwupm/packages";

/*
 * E(IP001)     = Unable to locate server
 * W(IP002)     = Unable to add ip to list due to it already existing
 * E(IP003)     = Unable to remove ip from list due to it not existing
 * E(IP004)     = Invalid communication protocol
 * E(SH005)     = Invalid command usage
 * E(FS006)     = Unable to create necessary files in setup
 * E(IP007)     = No available servers
 * W(FS008)     = A necessary folder doesn't exist
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

fn download(ip: &str, package: &str, save_name: &str) -> i8 {
    // Download package from server "ip" and save it to [SAVE_PATH]/[save_name]
    command(&format!("curl -o {}/{} {}/{}", SAVE_PATH, save_name, ip, package));
    return 0;
}


fn log(error_code: &str, logging_type: &str, message: &str) {
    if logging_type != "I".to_string() {
        // Logging for warnings and errors
        // 1 for bold, 31 for red and 38;5;208 for orange
        let ansi_escape_code = if logging_type == "E" {
           "\x1b[1;31m" // red for errors
        } else {
            "\x1b[1;38;5;208m" // orange for warnings
        };

        println!("{}{}\x1b[22m({}):\x1b[0m {} :3", ansi_escape_code, logging_type, error_code, message);
    }
    else {
        // Whoops, almost forgot about that one
        println!("\x1b[1;34mI:\x1b[0m {} :3", message);
    }
}


fn add_server(ip: String, force_flag: bool) -> Result<()> {
    let mut ip_list: String = fs::read_to_string(IPLIST_PATH)?;

    let found = ip_list.lines().any(|line| line.trim() == ip.trim());

    let clean_ip = ip.trim_start_matches("http://").trim_start_matches("https://")
                    .split(':').next().unwrap_or("");
    let ping_status = command(&format!("ping -c 1 {}", clean_ip));


    // Checking to make sure the ip isn't already in the list, is available and contains either http:// or https://
    if !found && (force_flag || ping_status == 0) && (ip.contains("http://") || ip.contains("https://")) {
        ip_list += &format!("\n{}", ip);
        fs::write(IPLIST_PATH, ip_list)?;
        log("", "I", "Wrote IP to IP list");
    }
    else if !ip_list.contains(&ip) {
        log("IP002", "W", "IP already in IP list - Not adding");
    }
    else if !(ip.contains("http://") || ip.contains("https://")) {
        log("IP004", "E", "Unable to add IP due to it missing http:// or https:// at the beginning");
    }
    else {
        log("IP001", "E", "Unable to reach server. Refusing to add IP");
    }

    Ok(())
}


fn remove_server(ip: String) -> Result<()> {
    let mut ip_list: String = fs::read_to_string(IPLIST_PATH)?;

    if ip_list.contains(&ip) {
        ip_list = String::from(ip_list.replace(&ip, ""));
        fs::write(IPLIST_PATH, ip_list)?;
        log("", "I", "Removed IP from IP list");
    }
    else {
        log("IP003", "E", "IP not yet in list - Can't remove");
    }
    
    Ok(())
}


fn update() -> Result<()>{
    log("", "I", "Reading IP list...");
    let ip_list_string: String = fs::read_to_string(IPLIST_PATH)?;

    // Split the list into a vector and filter it for empty indices
    let ip_list: Vec<&str> = ip_list_string.split("\n").filter(|s| !s.trim().is_empty()).collect();

    log("", "I", "Checking for valid IPs");
    if ip_list.is_empty() {
        log("IP007", "E", "No servers in IP list - cannot update. Make sure to run \"uwupm addip [http://, https://][SERVER_IP]:[PORT]\" before doing anything else ");
        return Ok(());
    }

    log("", "I", "Opening local packagelist...");

    if !Path::new(SAVE_PATH).exists() {
        log("FS008", "W", "Necessary folder for packages doesn't exist. Creating...");
        fs::create_dir_all(SAVE_PATH)?;
    }

    if !Path::new(&format!("{}/packagelist.txt", SAVE_PATH)).exists() {
        log("FS008", "W", "Necessary packagelist file doesn't exist. Creating...");
        fs::File::create(format!("{}/packagelist.txt", SAVE_PATH))?;
    }

    if !Path::new(&format!("{}/packagelist_partial.txt", SAVE_PATH)).exists() {
        log("FS008", "W", "Necessary packagelist_partial file doesn't exist. Creating...");
        fs::File::create(format!("{}/packagelist_partial.txt", SAVE_PATH))?;
    }

    fs::File::create(&format!("{}/packagelist.txt", SAVE_PATH))?;

    let mut package_list_file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(format!("{}/packagelist.txt", SAVE_PATH))?;

    for i in ip_list{
        log("", "I", &format!("Adding packagelist from {}...", i));
        download(i, "packagelist.txt", "packagelist_partial.txt");
        log("", "I", "Writing...");
        let contents = fs::read_to_string(&format!("{}/packagelist_partial.txt", SAVE_PATH))?;
        writeln!(package_list_file, "{}", contents)?;
    }

    log("", "I", "Copying packagelist...");
    fs::copy(&format!("{}/packagelist.txt", SAVE_PATH), PACKAGE_LIST_PATH)?;

    log("", "I", "Update done!");

    Ok(())
}


fn unknown_command(arg: String) -> Result<()>{
    log("SH005", "E", &format!("Unknown command \"{}\"", arg));
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
                    log("SH005", "E", "Invalid usage. Expected: uwupm addip [IP] [--force]");
                    Ok(())
                }
            },
            "removeip" => {
                if args.len() == 3 {
                    remove_server(args[2].clone())
                }
                else {
                    log("SH005", "E", "Invalid usage. Expected: uwupm removeip [IP]");
                    Ok(())
                }
            },
            "update" => update(),
            _ => unknown_command(args[1].clone())
        }?;
    }


    Ok(())
}

