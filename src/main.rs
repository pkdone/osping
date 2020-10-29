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


// Capture type of result from issuing a ping
enum PingResult {
    ConnectionSuccess,
    ConnectionFailure(String),
    DNSIssue(String),
    OSCmndIssue(String),
}


// Main function to call host OS ping executable with a host argument passed to this application
//
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("\nERROR: A host must be provided as an argument\n");
        exit(1);
    }

    let host = &args[1];

    match ping(host) {
        PingResult::ConnectionSuccess => println!("CONNECTION SUCCESS - Network ICMP Ping \
            successful for host '{}'", host),
        PingResult::ConnectionFailure(message) => println!("CONNECTION FAILURE - Network ICMP Ping \
            unsuccessful for host '{}' - error: {}", host, message),
        PingResult::DNSIssue(message) => println!("DNS FAILURE - DNS lookup issue for hostname \
            '{}' - error: {}", host, message),
        PingResult::OSCmndIssue(message) => println!("OS PING COMMAND ISSUE - problem executing \
            OS ping utility - error: {}", message),
    }
}


// Uses the underlying OS ping executable, on the host, to perform a network ICMP ping against a
// host (DNS name or IP address), returning a result typed to indicate success or the type of
// failure
//
fn ping(host: &str) -> PingResult {
    let mut cmd = &mut Command::new(PING_CMD);

    if cfg!(windows) {
        cmd = cmd.arg(WINDOWS_COUNT_ARG_KEY).arg(WINDOWS_COUNT_ARG_VAL);
    } else {
        cmd = cmd.arg(UNIX_COUNT_ARG).arg(UNIX_INTERVAL_ARG);
    }

    match cmd.arg(host).output() {
        Ok(output) => {
            debug_process_output(&output);

            if output.status.success() {
                PingResult::ConnectionSuccess
            } else if !cfg!(windows) && (output.status.code().unwrap_or(-1) == 1) {
                // Unix  (Unix's Ping uses code 1 for connection error & code 2 for other errors)
                PingResult::ConnectionFailure(format!("Host '{}' cannot be reached over a network \
                    ICMP Ping", host))
            } else {
                // Windows for all errors, Unix for non-connection related errors
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if stdout.contains("could not find host") {
                    // Windows
                    PingResult::DNSIssue(format!("Ping returned error indicating no DNS entry for \
                        '{}'. OS output received: '{}'", host, stdout))
                } else if stderr.contains("not known") {
                    // Unix
                    PingResult::DNSIssue(format!("Ping returned error indicating no DNS entry for \
                        '{}'. OS output received: '{}'", host, stderr))
                } else if stderr.contains("associated with hostname") {
                    // Unix
                    PingResult::DNSIssue(format!("Ping returned error indicating the DNS entry is \
                        not a hostname associated with an IP address. OS output received: '{}'",
                        stderr))
                } else if cfg!(windows) {
                    // Windows (Window's Ping uses stdout for errors rather than stderr
                    PingResult::ConnectionFailure(format!("Ping returned error. OS output received \
                        - stdout: '{}' - stderr: '{}'", stdout, stderr))
                } else {
                    // Unix
                    PingResult::ConnectionFailure(format!("Ping returned error. OS output \
                        received: '{}'", stderr))
                }
            }
        }
        Err(e) => {
            // Errors related to not being able to invoke Ping executable both on Windows & Unix
            debug_process_error(&e);
            if e.kind() == ErrorKind::NotFound {
                PingResult::OSCmndIssue("Unable to locate 'ping' executable in the local OS \
                    environment - ensure this executable is on your environment path (check your \
                    PATH environment variable)".to_string())
            } else if e.kind() == ErrorKind::PermissionDenied {
                PingResult::OSCmndIssue("Unable to run the 'ping' executable in the local OS \
                    environment due to lack of permissions - ensure the 'ping' command on your OS \
                    is assigned with executable permissions for your OS user running this \
                    tool".to_string())
            } else {
                PingResult::OSCmndIssue(format!("Unable to invoke the 'ping' executable on the \
                    underlying OS. OS output received: '{}'", e.to_string()))
            }
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
        assert!(if let PingResult::ConnectionSuccess = ping("www.google.com") { true } else { false })
    }


    #[test]
    fn assert_noping_bad_host() {
        assert!(
            if let PingResult::DNSIssue(_) = ping("www.doesnotexistindnshost.com") 
                { true } 
            else
                { false }
        )
    }
}
