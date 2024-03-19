# mavftp-cli

`mavftp-cli` is a command-line interface (CLI) tool written in Rust, designed to facilitate communication with devices that utilize the [MAVLink](https://mavlink.io/) protocol, enabling access to their filesystem through [MAVFTP](https://mavlink.io/en/services/ftp.html). This tool aims to streamline the process of interacting with UAVs (Unmanned Aerial Vehicles) and other MAVLink-compatible devices, making file transfers and management both efficient and straightforward.

## Features

```
USAGE:
    mavftp-cli [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --connection <connection>    Connection string [default: tcpout:0.0.0.0:5760]

SUBCOMMANDS:
    crc       Calculate CRC32 for a file
    create    Create a file
    help      Prints this message or the help of the given subcommand(s)
    list      List files in a directory
    mkdir     Create a directory
    read      Read a file
    remove    Remove a file
    reset     Reset sessions
    rmdir     Remove a directory
    write     Write to a file
```

```
$ ./mavftp-cli --connection serial:/dev/ttyACM2:115200 list
Type Name                           Size      
----------------------------------------
F    ./.Trash-1000                  39 B      
F    ./.Trashes                     39 B      
D    ./.fseventsd                             
F    ./.metadata_never_index        39 B      
D    ./APM                                    
F    ./dataman                      61.1 KB   
D    ./log                                    
F    ./param_import_fail.bson       16.0 KB   
F    ./param_import_fail.txt        1.1 KB    
F    ./parameters_backup.bson       454 B 
```

```
$ ./mavftp-cli --connection serial:/dev/ttyACM2:115200 read ./APM/LOGS/00000001.BIN
  [00:00:01] [##############################] 514.02 KiB/514.02 KiB (0.0s)
calculated crc: 0xd33fda9f
```

## Grab it
### Downloads :package:

[Latest builds](https://github.com/patrickelectric/mavftp-cli/releases/latest):
- :computer: [**Windows**](https://github.com/patrickelectric/mavftp-cli/releases/latest/download/mavftp-cli-x86_64-pc-windows-msvc.exe)
- :apple: [**MacOS**](https://github.com/patrickelectric/mavftp-cli/releases/latest/download/mavftp-cli-x86_64-apple-darwin)
- :penguin: [**Linux**](https://github.com/patrickelectric/mavftp-cli/releases/latest/download/mavftp-cli-x86_64-unknown-linux-musl)
- :strawberry: [**Raspberry**](https://github.com/patrickelectric/mavftp-cli/releases/latest/download/mavftp-cli-arm-unknown-linux-musleabihf)
  - [ARMv6 binary](https://github.com/patrickelectric/mavftp-cli/releases/latest/download/mavftp-cli-arm-unknown-linux-musleabihf), [ARMv7](https://github.com/patrickelectric/mavftp-cli/releases/latest/download/mavftp-cli-armv7-unknown-linux-musleabihf) is also available under the project releases.

For others or different releases, check the [releases menu](https://github.com/patrickelectric/mavftp-cli/releases).

## Build it

To install `mavftp-cli`, ensure you have Rust and Cargo installed on your machine. Follow these steps:

1. Clone the repository:
   ```bash
   git clone https://github.com/patrickelectric/mavftp-cli.git
   ```
2. Navigate to the project directory:
   ```bash
   cd mavftp-cli
   ```
3. Build:
   ```bash
   cargo build
   ```
4. Run:
   ```bash
   cargo run -- --help
   ```
