use embedded_sdmmc::{
    Block, BlockCount, BlockDevice, BlockIdx, Error as FatError, RawDirectory, RawVolume,
    ShortFileName, TimeSource, Timestamp, VolumeIdx, VolumeManager,
};
use psoc6_pac::Peripherals;

use crate::{hal::micro_sd, lisp};

const MBR_SIGNATURE_OFFSET: usize = 510;
const MBR_SIGNATURE: u16 = 0xaa55;
const PARTITION0_OFFSET: usize = 446;
const PARTITION_STATUS_OFFSET: usize = 0;
const PARTITION_TYPE_OFFSET: usize = 4;
const PARTITION_LBA_OFFSET: usize = 8;
const PARTITION_SECTORS_OFFSET: usize = 12;
const FAT16_SMALL_PARTITION: u8 = 0x04;
const FAT16_PARTITION: u8 = 0x06;
const FAT32_CHS_LBA_PARTITION: u8 = 0x0b;
const FAT32_LBA_PARTITION: u8 = 0x0c;
const FAT16_LBA_PARTITION: u8 = 0x0e;

pub struct InfoReport {
    pub status: FatStatus,
    pub mbr_signature: u16,
    pub partition_status: u8,
    pub partition_type: u8,
    pub partition_lba_start: u32,
    pub partition_sector_count: u32,
    pub root_entry_count: u8,
    pub sample_count: u8,
    pub entries: [lisp::StringBytes; lisp::MAX_STORE_FILES],
}

#[derive(Clone, Copy)]
pub enum FatStatus {
    Ready,
    MbrReadFailed,
    MissingMbrSignature,
    UnsupportedPartition,
    BlockDeviceFailed,
    VolumeOpenFailed,
    RootOpenFailed,
    RootIterateFailed,
    RootCloseFailed,
    VolumeCloseFailed,
}

#[derive(Clone, Copy, Debug)]
enum FatBlockError {
    BlockIndexOverflow,
    ReadFailed,
    WriteFailed,
}

#[derive(Clone, Copy)]
struct PsocBlockDevice<'a> {
    p: &'a Peripherals,
}

#[derive(Clone, Copy)]
struct FixedTime;

pub fn info(p: &Peripherals) -> InfoReport {
    let sector_zero = micro_sd::read_sector_words(p, 0);
    if !matches!(sector_zero.status, micro_sd::ReadStatus::Ready) {
        return info_report(FatStatus::MbrReadFailed, 0, 0, 0, 0, 0, 0, empty_entries());
    }

    let mut mbr = [0u8; Block::LEN];
    words_to_bytes(&sector_zero.words, &mut mbr);
    let mbr_signature = read_u16(&mbr, MBR_SIGNATURE_OFFSET);
    let partition = &mbr[PARTITION0_OFFSET..PARTITION0_OFFSET + 16];
    let partition_status = partition[PARTITION_STATUS_OFFSET];
    let partition_type = partition[PARTITION_TYPE_OFFSET];
    let partition_lba_start = read_u32(partition, PARTITION_LBA_OFFSET);
    let partition_sector_count = read_u32(partition, PARTITION_SECTORS_OFFSET);

    if mbr_signature != MBR_SIGNATURE {
        return info_report(
            FatStatus::MissingMbrSignature,
            mbr_signature,
            partition_status,
            partition_type,
            partition_lba_start,
            partition_sector_count,
            0,
            empty_entries(),
        );
    }

    if !is_supported_fat_partition(partition_type) {
        return info_report(
            FatStatus::UnsupportedPartition,
            mbr_signature,
            partition_status,
            partition_type,
            partition_lba_start,
            partition_sector_count,
            0,
            empty_entries(),
        );
    }

    let device = PsocBlockDevice { p };
    let manager: VolumeManager<_, _, 2, 1, 1> =
        VolumeManager::new_with_limits(device, FixedTime, 7000);

    let volume = match manager.open_raw_volume(VolumeIdx(0)) {
        Ok(volume) => volume,
        Err(error) => {
            return info_report(
                open_error_status(error),
                mbr_signature,
                partition_status,
                partition_type,
                partition_lba_start,
                partition_sector_count,
                0,
                empty_entries(),
            )
        }
    };

    let root = match manager.open_root_dir(volume) {
        Ok(root) => root,
        Err(error) => {
            close_volume_ignore_error(&manager, volume);
            return info_report(
                root_error_status(error),
                mbr_signature,
                partition_status,
                partition_type,
                partition_lba_start,
                partition_sector_count,
                0,
                empty_entries(),
            );
        }
    };

    let mut entries = empty_entries();
    let mut root_entry_count = 0u8;
    let mut sample_count = 0u8;
    if manager
        .iterate_dir(root, |entry| {
            root_entry_count = root_entry_count.saturating_add(1);
            if usize::from(sample_count) < entries.len() {
                entries[usize::from(sample_count)] = short_name_string(&entry.name);
                sample_count = sample_count.saturating_add(1);
            }
        })
        .is_err()
    {
        close_dir_ignore_error(&manager, root);
        close_volume_ignore_error(&manager, volume);
        return info_report(
            FatStatus::RootIterateFailed,
            mbr_signature,
            partition_status,
            partition_type,
            partition_lba_start,
            partition_sector_count,
            0,
            empty_entries(),
        );
    }

    if manager.close_dir(root).is_err() {
        close_volume_ignore_error(&manager, volume);
        return info_report(
            FatStatus::RootCloseFailed,
            mbr_signature,
            partition_status,
            partition_type,
            partition_lba_start,
            partition_sector_count,
            root_entry_count,
            entries,
        );
    }

    if manager.close_volume(volume).is_err() {
        return info_report(
            FatStatus::VolumeCloseFailed,
            mbr_signature,
            partition_status,
            partition_type,
            partition_lba_start,
            partition_sector_count,
            root_entry_count,
            entries,
        );
    }

    let mut report = info_report(
        FatStatus::Ready,
        mbr_signature,
        partition_status,
        partition_type,
        partition_lba_start,
        partition_sector_count,
        root_entry_count,
        entries,
    );
    report.sample_count = sample_count;
    report
}

impl BlockDevice for PsocBlockDevice<'_> {
    type Error = FatBlockError;

    fn read(&self, blocks: &mut [Block], start_block_idx: BlockIdx) -> Result<(), Self::Error> {
        let mut index = 0usize;
        while index < blocks.len() {
            let block_index = start_block_idx
                .0
                .checked_add(index as u32)
                .ok_or(FatBlockError::BlockIndexOverflow)?;
            let report = micro_sd::read_sector_words(self.p, block_index);
            if !matches!(report.status, micro_sd::ReadStatus::Ready) {
                return Err(FatBlockError::ReadFailed);
            }
            words_to_bytes(&report.words, &mut blocks[index].contents);
            index += 1;
        }
        Ok(())
    }

    fn write(&self, blocks: &[Block], start_block_idx: BlockIdx) -> Result<(), Self::Error> {
        let mut index = 0usize;
        while index < blocks.len() {
            let block_index = start_block_idx
                .0
                .checked_add(index as u32)
                .ok_or(FatBlockError::BlockIndexOverflow)?;
            let words = bytes_to_words(&blocks[index].contents);
            let report = micro_sd::write_sector_words(self.p, block_index, &words);
            if !matches!(report.status, micro_sd::WriteStatus::Ready) {
                return Err(FatBlockError::WriteFailed);
            }
            index += 1;
        }
        Ok(())
    }

    fn num_blocks(&self) -> Result<BlockCount, Self::Error> {
        let report = micro_sd::read_sector_words(self.p, 0);
        if !matches!(report.status, micro_sd::ReadStatus::Ready) {
            return Err(FatBlockError::ReadFailed);
        }

        let mut mbr = [0u8; Block::LEN];
        words_to_bytes(&report.words, &mut mbr);
        if read_u16(&mbr, MBR_SIGNATURE_OFFSET) != MBR_SIGNATURE {
            return Ok(BlockCount(0));
        }

        let partition = &mbr[PARTITION0_OFFSET..PARTITION0_OFFSET + 16];
        let partition_lba_start = read_u32(partition, PARTITION_LBA_OFFSET);
        let partition_sector_count = read_u32(partition, PARTITION_SECTORS_OFFSET);
        let card_blocks = partition_lba_start
            .checked_add(partition_sector_count)
            .ok_or(FatBlockError::BlockIndexOverflow)?;
        Ok(BlockCount(card_blocks))
    }
}

impl TimeSource for FixedTime {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 56,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

fn info_report(
    status: FatStatus,
    mbr_signature: u16,
    partition_status: u8,
    partition_type: u8,
    partition_lba_start: u32,
    partition_sector_count: u32,
    root_entry_count: u8,
    entries: [lisp::StringBytes; lisp::MAX_STORE_FILES],
) -> InfoReport {
    InfoReport {
        status,
        mbr_signature,
        partition_status,
        partition_type,
        partition_lba_start,
        partition_sector_count,
        root_entry_count,
        sample_count: 0,
        entries,
    }
}

fn is_supported_fat_partition(partition_type: u8) -> bool {
    matches!(
        partition_type,
        FAT16_SMALL_PARTITION
            | FAT16_PARTITION
            | FAT32_CHS_LBA_PARTITION
            | FAT32_LBA_PARTITION
            | FAT16_LBA_PARTITION
    )
}

fn open_error_status(error: FatError<FatBlockError>) -> FatStatus {
    match error {
        FatError::DeviceError(_) => FatStatus::BlockDeviceFailed,
        FatError::FormatError(_) => FatStatus::VolumeOpenFailed,
        _ => FatStatus::VolumeOpenFailed,
    }
}

fn root_error_status(error: FatError<FatBlockError>) -> FatStatus {
    match error {
        FatError::DeviceError(_) => FatStatus::BlockDeviceFailed,
        _ => FatStatus::RootOpenFailed,
    }
}

fn close_dir_ignore_error(
    manager: &VolumeManager<PsocBlockDevice<'_>, FixedTime, 2, 1, 1>,
    root: RawDirectory,
) {
    manager.close_dir(root).ok();
}

fn close_volume_ignore_error(
    manager: &VolumeManager<PsocBlockDevice<'_>, FixedTime, 2, 1, 1>,
    volume: RawVolume,
) {
    manager.close_volume(volume).ok();
}

fn short_name_string(name: &ShortFileName) -> lisp::StringBytes {
    let mut out = empty_string();
    let mut len = 0usize;
    for &byte in name.base_name() {
        push_byte(&mut out, &mut len, byte);
    }
    let extension = name.extension();
    if !extension.is_empty() {
        push_byte(&mut out, &mut len, b'.');
        for &byte in extension {
            push_byte(&mut out, &mut len, byte);
        }
    }
    out.len = len as u8;
    out
}

fn push_byte(out: &mut lisp::StringBytes, len: &mut usize, byte: u8) {
    if *len < out.bytes.len() {
        out.bytes[*len] = byte;
        *len += 1;
    }
}

fn empty_entries() -> [lisp::StringBytes; lisp::MAX_STORE_FILES] {
    [empty_string(); lisp::MAX_STORE_FILES]
}

fn empty_string() -> lisp::StringBytes {
    lisp::StringBytes {
        len: 0,
        bytes: [0; lisp::MAX_STRING_BYTES],
    }
}

fn words_to_bytes(words: &[u32; micro_sd::SD_BLOCK_WORDS], bytes: &mut [u8; Block::LEN]) {
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

fn bytes_to_words(bytes: &[u8; Block::LEN]) -> [u32; micro_sd::SD_BLOCK_WORDS] {
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

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    (bytes[offset] as u16) | ((bytes[offset + 1] as u16) << 8)
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    (bytes[offset] as u32)
        | ((bytes[offset + 1] as u32) << 8)
        | ((bytes[offset + 2] as u32) << 16)
        | ((bytes[offset + 3] as u32) << 24)
}
