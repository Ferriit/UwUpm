use std::ffi::CString;
use std::env;
use std::io::{Result, Write, Read, BufReader, self};
use std::fs;
use std::fs::File;
use std::path::Path;
use reqwest::blocking::get;
use indicatif::{ProgressBar, ProgressStyle};
use std::thread;
use std::cmp::min;
use std::sync::{Arc, Mutex};


const IPLIST_PATH: &str = "/etc/uwupm/iplist.txt";
const PACKAGE_LIST_PATH: &str = "/etc/uwupm/packagelist.txt";
const SAVE_PATH: &str = "/etc/uwupm/packages";
const PROGRESS_BAR_CHARS: &str = "█>-";
const PROGRESS_BAR_STYLE: &str = "[{elapsed_precise}] [{bar:60.default}] {bytes}/{total_bytes} ({eta})";
const THREAD_AMOUNT: i8 = 5;

/*
 * E(IP001)     = Unable to locate server
 * W(IP002)     = Unable to add ip to list due to it already existing
 * E(IP003)     = Unable to remove ip from list due to it not existing
 * E(IP004)     = Invalid communication protocol
 * E(SH005)     = Invalid command usage
 * E(FS006)     = Unable to create necessary files in setup
 * E(IP007)     = No available servers
 * W(FS008)     = A necessary folder doesn't exist
 * E(IP009)     = Unable to find a package on any server
 * E(DW010)     = Error downloading a package
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


fn download(ip: &str, package: &str, save_name: &str) -> io::Result<()> {
    let url = format!("{}/{}", ip.trim_end_matches('/'), package);

    let response = get(&url)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    // Get content length for progress bar (if available)
    let total_size = response
        .content_length()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to get content length"))?;

    let mut reader = BufReader::new(response);
    let mut dest = File::create(format!("{}/{}", SAVE_PATH, save_name))?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::with_template(PROGRESS_BAR_STYLE)
            .unwrap()
            .progress_chars(PROGRESS_BAR_CHARS),
    );

    let mut buffer = [0u8; 8192];
    let mut downloaded = 0;

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break; // EOF
        }
        dest.write_all(&buffer[..n])?;
        downloaded += n as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message("Download complete");
    Ok(())
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
        download(i, "packagelist.txt", "packagelist_partial.txt")?;
        log("", "I", "Writing...");
        let contents = fs::read_to_string(&format!("{}/packagelist_partial.txt", SAVE_PATH))?;
        writeln!(package_list_file, "{}", contents)?;
    }

    log("", "I", "Copying packagelist...");
    fs::copy(&format!("{}/packagelist.txt", SAVE_PATH), PACKAGE_LIST_PATH)?;

    log("", "I", "Update done!");

    Ok(())
}


fn install(arguments: &[String]) -> Result<()> {
    log("", "I", &format!("Installing package(s) {}", arguments.join(", ")));
    let mut packages: Vec<&str> = Vec::new();
    let mut flags: Vec<&str> = Vec::new();

    /* Flags:
     * -s/--skip = Skip unavailable packages
     * */
    // Separate Flags from Packages
    for i in arguments {
        if i.starts_with("-") {
            flags.push(i);
        } else {
            packages.push(i);
        }
    }

    //TODO: Implement more flags
    let skip_unavailable = flags.contains(&"-s") || flags.contains(&"--skip");

    log("", "I", "Reading package list...");
    let package_list_raw = fs::read_to_string(PACKAGE_LIST_PATH)?;

    // Format the raw package list to something actually useful
    let package_list: Vec<(String, String)> = package_list_raw
        .lines()    // Split string into lines
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let name = parts.next()?;     // Filter the name and URL
            let url = parts.next()?;
            Some((name.to_string(), url.to_string()))
        })
        .collect();

    // Check if the queued packages actually exist
    let mut download_queue: Vec<(String, String)> = Vec::new(); // Queue for packages to download if certain packages don't exist
    let mut downloadable = true;
    log("", "I", "Checking for package availability");
    for pkg_name in &packages {
        if !package_list.iter().any(|(_, name)| name == *pkg_name) {
            let error_type = if !skip_unavailable { "E" } else { "W" };
            log("IP009", error_type, &format!("Unable to locate package \"{}\" on any known servers", pkg_name));
            downloadable = false;
        }
        else {
            let package_idx: usize = package_list.iter()
            .position(|(_url, name)| name == *pkg_name)
            .unwrap();

            //let (url, name) = &package_list[package_idx];
            download_queue.push(package_list[package_idx].clone());
        }
    }

    // Exit due to packages being available
    if !downloadable && !skip_unavailable {
        return Ok(());
    }
    
    // List of all packages that already have been downloaded
    let downloaded_packages: Vec<usize> = Vec::new();

    let download_queue = Arc::new(download_queue); // if RO between threads
    let downloaded_packages = Arc::new(Mutex::new(downloaded_packages)); // R/W between threads

    // Download all packages in download queue in multiple threads
    let handles: Vec<_> = (0..min(THREAD_AMOUNT as usize, download_queue.len())).map(|thread_idx| {
        let dq = Arc::clone(&download_queue);
        let dp = Arc::clone(&downloaded_packages);

        thread::spawn(move || -> Result<()> {
            //println!("Hello from thread {}", i);
            

            let mut dp_lock = dp.lock().unwrap();

            for i in thread_idx..dq.len() {
                if !dp_lock.contains(&i) {
                    let (url, name) = &dq[i];
                    dp_lock.push(i);
                    download(&url, &format!("{}.tar.gz", name), &name)?;
                }
            }
            Ok(())
        })
    }).collect();
   
    for handle in handles {
        match handle.join() {
            Ok(thread_result) => {
                if let Err(e) = thread_result {
                    log("DW010", "E", &format!("{:?}", e));
                    return Ok(());
                }
            }
            Err(e) => {
                log("DW010", "E", &format!("{:?}", e));
                return Ok(());
            }
        }
    }

    log("", "I", "Install complete");
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
            "install" => {
                if args.len() > 2{
                    install(&args[2..])
                } else {
                    log("SH005", "E", "Invalid usage. Expected: uwupm install [packages/flags]");
                    Ok(())
                }
            },
            _ => unknown_command(args[1].clone())
        }?;
    }


    Ok(())
}

