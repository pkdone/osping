use std::env;
use std::error::Error;
use std::io::ErrorKind;
use std::process::{exit, Command, Output};


const PING_CMD: &str = "ping";
const UNIX_INTERVAL_ARG: &str = "-i 0.2";
const UNIX_COUNT_ARG: &str = "-c 3";
const WINDOWS_COUNT_ARG_KEY: &str = "-n";
const WINDOWS_COUNT_ARG_VAL: &str = "3";
const LOG_DEBUG: bool = false;


// Main function to call host OS ping executable with a host argument passed to this application
//
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("\nERROR: A host must be provided as an argument\n");
        exit(1);
    }

    let host = &args[1];
    ping(host)?;
    println!("Network ICMP Ping successful for host '{}'", host);
    Ok(())
}


// Uses the underlying OS ping executable, on the host, to perform a network ICMP ping against a
// host (DNS name or IP address), only returning an Error if the destination could not be reached by
// an ICMP ping
//
fn ping(host: &str) -> Result<(), Box<dyn Error>> {
    let mut cmd = &mut Command::new(PING_CMD);

    if cfg!(windows) {
        cmd = cmd.arg(WINDOWS_COUNT_ARG_KEY).arg(WINDOWS_COUNT_ARG_VAL);
    } else {
        cmd = cmd.arg(UNIX_COUNT_ARG).arg(UNIX_INTERVAL_ARG);
    }

    let output_res = cmd.arg(host).output();
    match output_res {
        Ok(output) => {
            debug_process_output(&output);

            if output.status.success() {
                Ok(())
            } else if !cfg!(windows) && (output.status.code().unwrap_or(-1) == 1) {
                // Unix
                Err(format!("Host '{}' cannot be reached over a network ICMP Ping", host).into())
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let err_msg =
                    if stdout.contains("could not find host") {
                        // Windows
                        format!("Ping returned error indicating no DNS entry for '{}'.  OS OUTPUT \
                            RECEIVED: '{}'", host, stdout)
                    } else if stderr.contains("not known") {
                        // Unix
                        format!("Ping returned error indicating no DNS entry for '{}'.  OS OUTPUT \
                            RECEIVED: '{}'", host, stderr)
                    } else if stderr.contains("associated with hostname") {
                        // Unix
                        format!("Ping returned error indicating the DNS entry is not a hostname \
                            associated with an IP address.  OS OUTPUT RECEIVED: '{}'", stderr)
                    } else if cfg!(windows) {
                        // Windows
                        format!("Ping returned error.  OS OUTPUT RECEIVED - stdout: '{}' - stderr: \
                        '{}'", stdout, stderr)
                    } else {
                        // Unix
                        format!("Ping returned error.  OS OUTPUT RECEIVED: '{}'", stderr)
                    };

                Err(err_msg.into())
            }
        }
        Err(e) => {
            debug_process_error(&e);
            let err_msg =
                if e.kind() == ErrorKind::NotFound {
                    "Unable to locate 'ping' executable in the local OS environment - ensure this \
                    executable is on your environment path (check your PATH environment \
                    variable)".to_string()
                } else if e.kind() == ErrorKind::PermissionDenied {
                    "Unable to run the 'ping' executable in the local OS environment due to lack \
                    of permissions - ensure the 'ping' command on your OS is assigned with \
                    executable permissions for your OS user running this tool".to_string()
                } else {
                    format!("Unable to invoke the 'ping' executable on the underlying OS.  \
                        OS OUTPUT RECEIVED: '{}'", e.to_string())
                };
            Err(err_msg.into())
        }
    }
}


// Print out the ping command output if the debug 'constant' is set to true
//
fn debug_process_output(output: &Output) {
    if LOG_DEBUG {
        println!("\n ---------------------");
        println!(" Process result:");
        println!("  * Status: {}", output.status);
        println!("  * Stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("  * Stderr: {}", String::from_utf8_lossy(&output.stderr));
        println!(" ---------------------\n");
    }
}


// Print out the ping command error if the debug 'constant' is set to true
//
fn debug_process_error(error: &dyn Error) {
    if LOG_DEBUG {
        println!("\n ---------------------");
        println!(" Process error:");
        println!("  * Message: {:?}", error);
        println!(" ---------------------\n");
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn assert_ping_good_host() {
        assert!(ping("www.google.com").is_ok());
    }


    #[test]
    fn assert_noping_bad_host() {
        assert!(ping("www.doesnotexistindnshost.com").is_err());
    }
}
