use std::io::Write;
use std::process::exit;
use std::time::SystemTime;

use crate::mavftp::*;
use num_traits::FromPrimitive;

use indicatif::{ProgressBar, ProgressState, ProgressStyle};

use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom};

enum OperationStatus {
    ScanningFolder(ScanningFolderStatus),
    OpeningFile(OpeningFileStatus),
    ReadingFile(ReadingFileStatus),
    Reset,
    CalcFileCRC32(CalcFileCRC32Status),
    ClosingSession
}

struct ScanningFolderStatus {
    path: String,
    offset: u8,
}

struct OpeningFileStatus {
    path: String,
}

struct CalcFileCRC32Status {
    path: String,
}

struct ReadingFileStatus {
    path: String,
    offset: u32,
    file_size: u32,
    file: std::fs::File,
}

pub struct Controller {
    session: u8,
    last_time: SystemTime,
    entries: Vec<EntryInfo>,
    status: Option<OperationStatus>,
    waiting: bool,
    progress: Option<ProgressBar>,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            session: 0,
            last_time: SystemTime::now(),
            entries: Vec::new(),
            status: None,
            waiting: false,
            progress: None,
        }
    }

    pub fn list_directory(&mut self, path: String) {
        self.status = Some(OperationStatus::ScanningFolder(ScanningFolderStatus {
            path,
            offset: 0,
        }))
    }

    pub fn read_file(&mut self, path: String) {
        self.status = Some(OperationStatus::OpeningFile(OpeningFileStatus { path }));
    }

    pub fn reset(&mut self) {
        self.status = Some(OperationStatus::Reset);
    }

    pub fn crc(&mut self, path: String) {
        self.status = Some(OperationStatus::CalcFileCRC32(CalcFileCRC32Status { path }));
    }

    pub fn run(&mut self) -> Option<MavlinkFtpPayload> {
        if self.waiting {
            return None;
        }
        self.waiting = true;
        match &self.status {
            Some(OperationStatus::Reset) => {
                return Some(MavlinkFtpPayload::new_reset_sesions(1, self.session));
            }
            Some(OperationStatus::ScanningFolder(status)) => {
                return Some(MavlinkFtpPayload::new_list_directory(
                    1,
                    self.session,
                    status.offset as u32,
                    &status.path,
                ));
            }
            Some(OperationStatus::OpeningFile(status)) => {
                return Some(MavlinkFtpPayload::new_open_file(
                    1,
                    self.session,
                    &status.path,
                ));
            }
            Some(OperationStatus::CalcFileCRC32(status)) => {
                return Some(MavlinkFtpPayload::new_calc_file_crc32(
                    1,
                    self.session,
                    &status.path,
                ));
            }
            Some(OperationStatus::ReadingFile(status)) => {
                return Some(MavlinkFtpPayload::new_read_file(
                    1,
                    self.session,
                    0,
                    usize::MAX,
                ));
            }
            _ => return None,
        }
    }

    pub fn parse_mavlink_message(
        &mut self,
        message: &mavlink::common::FILE_TRANSFER_PROTOCOL_DATA,
    ) -> Option<mavlink::common::MavMessage> {
        self.waiting = false;
        let payload = MavlinkFtpPayload::from_bytes(&message.payload).unwrap();
        match payload.opcode {
            MavlinkFtpOpcode::Ack => {
                match &mut self.status {
                    Some(OperationStatus::Reset) => {
                        if payload.req_opcode == MavlinkFtpOpcode::ResetSessions {
                            self.waiting = false;
                            self.status = None;
                        }
                    }
                    Some(OperationStatus::ScanningFolder(status)) => {
                        let entries: Vec<&[u8]> = payload.data.split(|&byte| byte == 0).collect();

                        if entries.is_empty() {
                            return None;
                        }

                        for entry in entries {
                            if entry.is_empty() {
                                continue;
                            }
                            status.offset += 1;

                            if let Ok(mut result) =
                                parse_directory_entry(&String::from_utf8_lossy(entry))
                            {
                                result.name = format!("{}/{}", status.path, result.name);
                                self.entries.push(result);
                            }
                        }

                        if status.offset != 0 {
                            self.waiting = true;
                            return Some(mavlink::common::MavMessage::FILE_TRANSFER_PROTOCOL(
                                mavlink::common::FILE_TRANSFER_PROTOCOL_DATA {
                                    target_network: 0,
                                    target_system: 1,
                                    target_component: 1,
                                    payload: MavlinkFtpPayload::new_list_directory(
                                        1,
                                        self.session,
                                        status.offset as u32,
                                        &status.path,
                                    )
                                    .to_bytes(),
                                },
                            ));
                        }
                    }
                    Some(OperationStatus::OpeningFile(status)) => {
                        if payload.size != 4 {
                            panic!("Wrong size");
                        }
                        let file_size = u32::from_le_bytes([
                            payload.data[0],
                            payload.data[1],
                            payload.data[2],
                            payload.data[3],
                        ]);

                        self.progress = Some(ProgressBar::new(file_size as u64));
                        if let Some(progress) = &mut self.progress {
                            progress.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                                .unwrap()
                                .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
                                .progress_chars("#>-")
                            );
                        }

                        self.status = Some(OperationStatus::ReadingFile(ReadingFileStatus {
                            path: status.path.clone(),
                            offset: 0,
                            file_size,
                            file: OpenOptions::new()
                                .write(true)
                                .create(true)
                                .open(status.path.split('/').last().unwrap())
                                .unwrap(),
                        }));

                        return None;
                    }
                    Some(OperationStatus::CalcFileCRC32(status)) => {
                        if payload.req_opcode == MavlinkFtpOpcode::CalcFileCRC32 {
                            let crc = u32::from_le_bytes([
                                payload.data[0],
                                payload.data[1],
                                payload.data[2],
                                payload.data[3],
                            ]);
                            println!("crc: 0x{:x?}", crc);
                            exit(0);
                        }
                    }
                    Some(OperationStatus::ReadingFile(status)) => {
                        let chunk = &payload.data;
                        status
                            .file
                            .seek(SeekFrom::Start(payload.offset.into()))
                            .unwrap();
                        status.file.write_all(chunk).unwrap();
                        status.offset = payload.offset + payload.size as u32;
                        if let Some(progress) = &self.progress {
                            progress.set_position(status.offset as u64);
                        }

                        if status.offset < status.file_size {
                            self.waiting = true;
                            
                            if payload.burst_complete == 1 {
                                return Some(mavlink::common::MavMessage::FILE_TRANSFER_PROTOCOL(
                                    mavlink::common::FILE_TRANSFER_PROTOCOL_DATA {
                                        target_network: 0,
                                        target_system: 1,
                                        target_component: 1,
                                        payload: MavlinkFtpPayload::new_read_file(
                                            payload.seq_number + 1,
                                            self.session,
                                            status.offset,
                                            usize::MAX,
                                        )
                                        .to_bytes(),
                                    },
                                ));
                            } else {
                                return None;
                            }
                        } else {
                            if let Some(progress) = &self.progress {
                                progress.finish();
                            }

                            // Lets get the crc
                            let mut buffer = Vec::new();
                            let mut file = std::fs::File::open(status.path.split('/').last().unwrap()).unwrap();
                            file.read_to_end(&mut buffer).unwrap();
                            let crc = mavlink_crc32(&buffer);
                            println!("calculated crc: 0x{:08x}", crc);

                            self.status = Some(OperationStatus::ClosingSession);
                            self.waiting = true;

                            return Some(mavlink::common::MavMessage::FILE_TRANSFER_PROTOCOL(
                                mavlink::common::FILE_TRANSFER_PROTOCOL_DATA {
                                    target_network: 0,
                                    target_system: 1,
                                    target_component: 1,
                                    payload: MavlinkFtpPayload::new_terminate_session(
                                        payload.seq_number + 1,
                                        self.session,
                                    )
                                    .to_bytes(),
                                },
                            ));
                        }
                    }
                    Some(OperationStatus::ClosingSession) => {
                        println!("session closed");
                        exit(0);
                    }
                    None => return None,
                }
            }
            MavlinkFtpOpcode::Nak => {
                let nak_code = MavlinkFtpNak::from_u8(payload.data[0]).unwrap();

                match nak_code {
                    MavlinkFtpNak::EOF => {
                        // We finished the current operation
                        match &payload.req_opcode {
                            MavlinkFtpOpcode::ListDirectory => {
                                println!("{:<4} {:<30} {:<10}", "Type", "Name", "Size");
                                println!("{}", "-".repeat(40));
                                self.entries
                                    .sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
                                for entry in &self.entries {
                                    let item_type = match entry.entry_type {
                                        EntryType::File => 'F',
                                        EntryType::Directory => 'D',
                                        EntryType::Skip => 'S',
                                    };
                                    println!(
                                        "{:<4} {:<30} {:<10}",
                                        item_type,
                                        entry.name,
                                        format_size(entry.size as u64)
                                    );
                                }
                            }
                            _ => {}
                        }
                        exit(0);
                        self.status = None;
                        return None;
                    }
                    MavlinkFtpNak::FailErrno => {
                        return None;
                    }
                    _ => {
                        // Something is wrong... but it'll deal with it in the same way
                        return None;
                    }
                }
            }
            _ => {}
        }

        return None;
    }
}

fn format_size(size: u64) -> String {
    const KILO: u64 = 1024;
    const MEGA: u64 = KILO * 1024;
    const GIGA: u64 = MEGA * 1024;

    match size {
        0 => String::new(),
        1..=KILO => format!("{} B", size),
        KILO..=MEGA => format!("{:.1} KB", (size as f64) / (KILO as f64)),
        MEGA..=GIGA => format!("{:.1} MB", (size as f64) / (MEGA as f64)),
        _ => format!("{:.1} GB", (size as f64) / (GIGA as f64)),
    }
}
