use psoc6_pac::Peripherals;

use crate::{hal::micro_sd, lisp};

pub const DIRECTORY_SECTOR: u32 = 0x100;
const DATA_START_SECTOR: u32 = 0x101;
const DIRECTORY_MAGIC: [u8; 4] = *b"LSPF";
const DATA_MAGIC: [u8; 4] = *b"LSPD";
const VERSION: u32 = 1;
const SECTOR_BYTES: usize = 512;
const DIRECTORY_HEADER_BYTES: usize = 16;
const DIRECTORY_ENTRY_BYTES: usize = 96;
pub const DIRECTORY_MAX_FILES: usize = 5;
const PATH_MAX_BYTES: usize = 64;
const DATA_HEADER_BYTES: usize = 12;

#[derive(Clone, Copy)]
pub enum StoreStatus {
    Ready,
    EmptyPath,
    PathTooLong,
    ContentTooLong,
    NotFound,
    DirectoryFull,
    DirectoryReadFailed,
    DirectoryWriteFailed,
    DataReadFailed,
    DataWriteFailed,
    CorruptData,
    ChecksumMismatch,
}

pub struct WriteReport {
    pub status: StoreStatus,
    pub path_len: u8,
    pub content_len: u8,
    pub directory_sector: u32,
    pub data_sector: u32,
}

pub struct ReadReport {
    pub status: StoreStatus,
    pub path_len: u8,
    pub content_len: u8,
    pub directory_sector: u32,
    pub data_sector: u32,
    pub content: lisp::StringBytes,
}

pub struct ListReport {
    pub status: StoreStatus,
    pub file_count: u8,
    pub directory_sector: u32,
    pub files: [lisp::StringBytes; DIRECTORY_MAX_FILES],
}

#[derive(Clone, Copy)]
struct DirectoryEntry {
    index: usize,
    path_len: u8,
    data_sector: u32,
    content_len: u8,
    checksum: u32,
}

pub fn write_file(
    p: &Peripherals,
    path: lisp::StringBytes,
    content: lisp::StringBytes,
) -> WriteReport {
    let path_len = path.len as usize;
    let content_len = content.len as usize;
    if path_len == 0 {
        return write_report(StoreStatus::EmptyPath, path, content, 0);
    }
    if path_len > PATH_MAX_BYTES {
        return write_report(StoreStatus::PathTooLong, path, content, 0);
    }
    if content_len > lisp::MAX_STRING_BYTES {
        return write_report(StoreStatus::ContentTooLong, path, content, 0);
    }

    let mut directory = [0u8; SECTOR_BYTES];
    let directory_read = micro_sd::read_sector_words(p, DIRECTORY_SECTOR);
    if !matches!(directory_read.status, micro_sd::ReadStatus::Ready) {
        return write_report(StoreStatus::DirectoryReadFailed, path, content, 0);
    }
    words_to_bytes(&directory_read.words, &mut directory);
    if !directory_is_formatted(&directory) {
        initialize_directory(&mut directory);
    }

    let index = match find_entry(&directory, &path.bytes[..path_len])
        .map(|entry| entry.index)
        .or_else(|| find_free_entry(&directory))
    {
        Some(index) => index,
        None => return write_report(StoreStatus::DirectoryFull, path, content, 0),
    };
    let data_sector = DATA_START_SECTOR + index as u32;

    let mut data = [0u8; SECTOR_BYTES];
    data[0..4].copy_from_slice(&DATA_MAGIC);
    write_u32(&mut data, 4, content_len as u32);
    let checksum = checksum(&content.bytes[..content_len]);
    write_u32(&mut data, 8, checksum);
    data[DATA_HEADER_BYTES..DATA_HEADER_BYTES + content_len]
        .copy_from_slice(&content.bytes[..content_len]);

    let data_words = bytes_to_words(&data);
    let data_write = micro_sd::write_sector_words(p, data_sector, &data_words);
    if !matches!(data_write.status, micro_sd::WriteStatus::Ready) {
        return write_report(StoreStatus::DataWriteFailed, path, content, data_sector);
    }

    write_entry(
        &mut directory,
        index,
        path,
        data_sector,
        content.len,
        checksum,
    );
    let directory_words = bytes_to_words(&directory);
    let directory_write = micro_sd::write_sector_words(p, DIRECTORY_SECTOR, &directory_words);
    if !matches!(directory_write.status, micro_sd::WriteStatus::Ready) {
        return write_report(
            StoreStatus::DirectoryWriteFailed,
            path,
            content,
            data_sector,
        );
    }

    write_report(StoreStatus::Ready, path, content, data_sector)
}

pub fn read_file(p: &Peripherals, path: lisp::StringBytes) -> ReadReport {
    let path_len = path.len as usize;
    if path_len == 0 {
        return read_report(StoreStatus::EmptyPath, path, 0, 0, empty_string());
    }
    if path_len > PATH_MAX_BYTES {
        return read_report(StoreStatus::PathTooLong, path, 0, 0, empty_string());
    }

    let mut directory = [0u8; SECTOR_BYTES];
    let directory_read = micro_sd::read_sector_words(p, DIRECTORY_SECTOR);
    if !matches!(directory_read.status, micro_sd::ReadStatus::Ready) {
        return read_report(StoreStatus::DirectoryReadFailed, path, 0, 0, empty_string());
    }
    words_to_bytes(&directory_read.words, &mut directory);
    if !directory_is_formatted(&directory) {
        return read_report(StoreStatus::NotFound, path, 0, 0, empty_string());
    }

    let entry = match find_entry(&directory, &path.bytes[..path_len]) {
        Some(entry) => entry,
        None => return read_report(StoreStatus::NotFound, path, 0, 0, empty_string()),
    };

    let data_read = micro_sd::read_sector_words(p, entry.data_sector);
    if !matches!(data_read.status, micro_sd::ReadStatus::Ready) {
        return read_report(
            StoreStatus::DataReadFailed,
            path,
            0,
            entry.data_sector,
            empty_string(),
        );
    }

    let mut data = [0u8; SECTOR_BYTES];
    words_to_bytes(&data_read.words, &mut data);
    if data[0..4] != DATA_MAGIC {
        return read_report(
            StoreStatus::CorruptData,
            path,
            0,
            entry.data_sector,
            empty_string(),
        );
    }

    let content_len = read_u32(&data, 4) as usize;
    let stored_checksum = read_u32(&data, 8);
    if content_len > lisp::MAX_STRING_BYTES || content_len > SECTOR_BYTES - DATA_HEADER_BYTES {
        return read_report(
            StoreStatus::CorruptData,
            path,
            0,
            entry.data_sector,
            empty_string(),
        );
    }
    if content_len != entry.content_len as usize {
        return read_report(
            StoreStatus::CorruptData,
            path,
            0,
            entry.data_sector,
            empty_string(),
        );
    }

    let content_bytes = &data[DATA_HEADER_BYTES..DATA_HEADER_BYTES + content_len];
    if stored_checksum != entry.checksum || checksum(content_bytes) != stored_checksum {
        return read_report(
            StoreStatus::ChecksumMismatch,
            path,
            0,
            entry.data_sector,
            empty_string(),
        );
    }

    let mut content = empty_string();
    content.len = content_len as u8;
    content.bytes[..content_len].copy_from_slice(content_bytes);
    read_report(
        StoreStatus::Ready,
        path,
        content.len,
        entry.data_sector,
        content,
    )
}

pub fn list_files(p: &Peripherals) -> ListReport {
    let mut files = [empty_string(); DIRECTORY_MAX_FILES];
    let mut directory = [0u8; SECTOR_BYTES];
    let directory_read = micro_sd::read_sector_words(p, DIRECTORY_SECTOR);
    if !matches!(directory_read.status, micro_sd::ReadStatus::Ready) {
        return list_report(StoreStatus::DirectoryReadFailed, 0, files);
    }
    words_to_bytes(&directory_read.words, &mut directory);
    if !directory_is_formatted(&directory) {
        return list_report(StoreStatus::Ready, 0, files);
    }

    let mut file_count = 0usize;
    let mut index = 0usize;
    while index < DIRECTORY_MAX_FILES {
        let offset = entry_offset(index);
        if directory[offset] != 0 {
            let path_len = directory[offset + 1] as usize;
            if path_len == 0 || path_len > PATH_MAX_BYTES || path_len > lisp::MAX_STRING_BYTES {
                return list_report(StoreStatus::CorruptData, file_count as u8, files);
            }

            files[file_count].len = path_len as u8;
            files[file_count].bytes[..path_len]
                .copy_from_slice(&directory[offset + 2..offset + 2 + path_len]);
            file_count += 1;
        }
        index += 1;
    }

    list_report(StoreStatus::Ready, file_count as u8, files)
}

fn write_report(
    status: StoreStatus,
    path: lisp::StringBytes,
    content: lisp::StringBytes,
    data_sector: u32,
) -> WriteReport {
    WriteReport {
        status,
        path_len: path.len,
        content_len: content.len,
        directory_sector: DIRECTORY_SECTOR,
        data_sector,
    }
}

fn read_report(
    status: StoreStatus,
    path: lisp::StringBytes,
    content_len: u8,
    data_sector: u32,
    content: lisp::StringBytes,
) -> ReadReport {
    ReadReport {
        status,
        path_len: path.len,
        content_len,
        directory_sector: DIRECTORY_SECTOR,
        data_sector,
        content,
    }
}

fn list_report(
    status: StoreStatus,
    file_count: u8,
    files: [lisp::StringBytes; DIRECTORY_MAX_FILES],
) -> ListReport {
    ListReport {
        status,
        file_count,
        directory_sector: DIRECTORY_SECTOR,
        files,
    }
}

fn empty_string() -> lisp::StringBytes {
    lisp::StringBytes {
        len: 0,
        bytes: [0; lisp::MAX_STRING_BYTES],
    }
}

fn initialize_directory(directory: &mut [u8; SECTOR_BYTES]) {
    directory.fill(0);
    directory[0..4].copy_from_slice(&DIRECTORY_MAGIC);
    write_u32(directory, 4, VERSION);
}

fn directory_is_formatted(directory: &[u8; SECTOR_BYTES]) -> bool {
    directory[0..4] == DIRECTORY_MAGIC && read_u32(directory, 4) == VERSION
}

fn find_free_entry(directory: &[u8; SECTOR_BYTES]) -> Option<usize> {
    let mut index = 0usize;
    while index < DIRECTORY_MAX_FILES {
        let offset = entry_offset(index);
        if directory[offset] == 0 {
            return Some(index);
        }
        index += 1;
    }
    None
}

fn find_entry(directory: &[u8; SECTOR_BYTES], path: &[u8]) -> Option<DirectoryEntry> {
    let mut index = 0usize;
    while index < DIRECTORY_MAX_FILES {
        let offset = entry_offset(index);
        if directory[offset] != 0 {
            let path_len = directory[offset + 1] as usize;
            if path_len == path.len()
                && path_len <= PATH_MAX_BYTES
                && directory[offset + 2..offset + 2 + path_len] == *path
            {
                return Some(DirectoryEntry {
                    index,
                    path_len: path_len as u8,
                    data_sector: read_u32(directory, offset + 68),
                    content_len: read_u32(directory, offset + 72) as u8,
                    checksum: read_u32(directory, offset + 76),
                });
            }
        }
        index += 1;
    }
    None
}

fn write_entry(
    directory: &mut [u8; SECTOR_BYTES],
    index: usize,
    path: lisp::StringBytes,
    data_sector: u32,
    content_len: u8,
    checksum: u32,
) {
    let offset = entry_offset(index);
    let path_len = path.len as usize;
    directory[offset..offset + DIRECTORY_ENTRY_BYTES].fill(0);
    directory[offset] = 1;
    directory[offset + 1] = path.len;
    directory[offset + 2..offset + 2 + path_len].copy_from_slice(&path.bytes[..path_len]);
    write_u32(directory, offset + 68, data_sector);
    write_u32(directory, offset + 72, content_len as u32);
    write_u32(directory, offset + 76, checksum);
}

fn entry_offset(index: usize) -> usize {
    DIRECTORY_HEADER_BYTES + index * DIRECTORY_ENTRY_BYTES
}

fn checksum(bytes: &[u8]) -> u32 {
    let mut value = 0x811c_9dc5u32;
    for &byte in bytes {
        value ^= byte as u32;
        value = value.wrapping_mul(0x0100_0193);
    }
    value
}

fn words_to_bytes(words: &[u32; micro_sd::SD_BLOCK_WORDS], bytes: &mut [u8; SECTOR_BYTES]) {
    let mut index = 0usize;
    while index < words.len() {
        let word = words[index];
        let offset = index * 4;
        bytes[offset] = word as u8;
        bytes[offset + 1] = (word >> 8) as u8;
        bytes[offset + 2] = (word >> 16) as u8;
        bytes[offset + 3] = (word >> 24) as u8;
        index += 1;
    }
}

fn bytes_to_words(bytes: &[u8; SECTOR_BYTES]) -> [u32; micro_sd::SD_BLOCK_WORDS] {
    let mut words = [0u32; micro_sd::SD_BLOCK_WORDS];
    let mut index = 0usize;
    while index < words.len() {
        let offset = index * 4;
        words[index] = (bytes[offset] as u32)
            | ((bytes[offset + 1] as u32) << 8)
            | ((bytes[offset + 2] as u32) << 16)
            | ((bytes[offset + 3] as u32) << 24);
        index += 1;
    }
    words
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    (bytes[offset] as u32)
        | ((bytes[offset + 1] as u32) << 8)
        | ((bytes[offset + 2] as u32) << 16)
        | ((bytes[offset + 3] as u32) << 24)
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset] = value as u8;
    bytes[offset + 1] = (value >> 8) as u8;
    bytes[offset + 2] = (value >> 16) as u8;
    bytes[offset + 3] = (value >> 24) as u8;
}
