use core::str;

use embedded_sdmmc::{
    Block, BlockCount, BlockDevice, BlockIdx, Error as FatError, Mode, RawDirectory, RawVolume,
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
const FAT32_SECTORS_PER_CLUSTER: u8 = 64;
const FAT32_RESERVED_SECTORS: u16 = 32;
const FAT32_FAT_COUNT: u8 = 2;
const FAT32_ROOT_CLUSTER: u32 = 2;
const FAT32_FSINFO_SECTOR: u16 = 1;
const FAT32_BACKUP_BOOT_SECTOR: u16 = 6;
const FAT32_MIN_CLUSTERS: u32 = 65_525;
const FAT32_MAX_CLUSTERS: u32 = 0x0fff_ffef;
const VOLUME_ID: u32 = 0x2026_0623;
const VOLUME_LABEL: &[u8; 11] = b"LISPPSOC6  ";
const FORMAT_PROGRESS_CHUNK_SECTORS: u32 = 256;
const FAT_LISP_EXTENSION: &[u8; 3] = b"lsp";
const LISP_EXTENSION: &[u8; 4] = b"lisp";
const MAX_SHORT_PATH_TEXT_BYTES: usize = 12;

pub trait FormatProgress {
    fn report(&mut self, phase: &'static [u8], written_sector_count: u32, total_sectors: u32);
}

struct SilentProgress;

impl FormatProgress for SilentProgress {
    fn report(&mut self, _phase: &'static [u8], _written_sector_count: u32, _total_sectors: u32) {}
}

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

pub struct FormatReport {
    pub status: FatStatus,
    pub mbr_signature: u16,
    pub partition_status: u8,
    pub partition_type_before: u8,
    pub partition_type_after: u8,
    pub partition_lba_start: u32,
    pub partition_sector_count: u32,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub fat_count: u8,
    pub fat_size_sectors: u32,
    pub data_cluster_count: u32,
    pub root_cluster: u32,
    pub written_sector_count: u32,
    pub failed_sector: u32,
}

pub struct FileWriteReport {
    pub status: FatStatus,
    pub path_len: u8,
    pub content_len: u8,
}

pub struct FileReadReport {
    pub status: FatStatus,
    pub path_len: u8,
    pub content_len: u8,
    pub content: lisp::StringBytes,
}

pub struct FileListReport {
    pub status: FatStatus,
    pub file_count: u8,
    pub files: [lisp::StringBytes; lisp::MAX_STORE_FILES],
}

#[derive(Clone, Copy)]
pub enum FatStatus {
    Ready,
    EmptyPath,
    PathTooLong,
    ContentTooLong,
    InvalidPath,
    NotFound,
    MbrReadFailed,
    MissingMbrSignature,
    UnsupportedPartition,
    BlockDeviceFailed,
    FormatGeometryInvalid,
    FormatMbrWriteFailed,
    FormatBootWriteFailed,
    FormatFsInfoWriteFailed,
    FormatFatClearFailed,
    FormatFatHeaderWriteFailed,
    FormatRootClearFailed,
    VolumeOpenFailed,
    RootOpenFailed,
    RootIterateFailed,
    RootCloseFailed,
    VolumeCloseFailed,
    FileOpenFailed,
    FileReadFailed,
    FileWriteFailed,
    FileCloseFailed,
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

#[derive(Clone, Copy)]
struct Fat32Layout {
    partition_status: u8,
    partition_type_before: u8,
    partition_lba_start: u32,
    partition_sector_count: u32,
    fat_size_sectors: u32,
    data_cluster_count: u32,
}

struct ShortPathText {
    len: usize,
    bytes: [u8; MAX_SHORT_PATH_TEXT_BYTES],
}

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

pub fn format_fat32(p: &Peripherals) -> FormatReport {
    let mut progress = SilentProgress;
    format_fat32_with_progress(p, &mut progress)
}

pub fn write_file(
    p: &Peripherals,
    path: lisp::StringBytes,
    content: lisp::StringBytes,
) -> FileWriteReport {
    let short_name = match path_short_name(path) {
        Ok(short_name) => short_name,
        Err(status) => return file_write_report(status, path, content),
    };
    if content.len as usize > lisp::MAX_STRING_BYTES {
        return file_write_report(FatStatus::ContentTooLong, path, content);
    }

    let (manager, volume, root) = match open_root(p) {
        Ok(opened) => opened,
        Err(status) => return file_write_report(status, path, content),
    };

    let file = match manager.open_file_in_dir(root, short_name, Mode::ReadWriteCreateOrTruncate) {
        Ok(file) => file,
        Err(error) => {
            close_dir_ignore_error(&manager, root);
            close_volume_ignore_error(&manager, volume);
            return file_write_report(file_open_error_status(error), path, content);
        }
    };

    let content_bytes = &content.bytes[..content.len as usize];
    if manager.write(file, content_bytes).is_err() {
        manager.close_file(file).ok();
        close_dir_ignore_error(&manager, root);
        close_volume_ignore_error(&manager, volume);
        return file_write_report(FatStatus::FileWriteFailed, path, content);
    }

    if manager.close_file(file).is_err() {
        close_dir_ignore_error(&manager, root);
        close_volume_ignore_error(&manager, volume);
        return file_write_report(FatStatus::FileCloseFailed, path, content);
    }

    let close_status = close_root(manager, volume, root);
    file_write_report(close_status, path, content)
}

pub fn read_file(p: &Peripherals, path: lisp::StringBytes) -> FileReadReport {
    let short_name = match path_short_name(path) {
        Ok(short_name) => short_name,
        Err(status) => return file_read_report(status, path, empty_string()),
    };

    let (manager, volume, root) = match open_root(p) {
        Ok(opened) => opened,
        Err(status) => return file_read_report(status, path, empty_string()),
    };

    let file = match manager.open_file_in_dir(root, short_name, Mode::ReadOnly) {
        Ok(file) => file,
        Err(error) => {
            close_dir_ignore_error(&manager, root);
            close_volume_ignore_error(&manager, volume);
            return file_read_report(file_open_error_status(error), path, empty_string());
        }
    };

    let mut content = empty_string();
    let mut content_len = 0usize;
    let mut chunk = [0u8; 16];
    loop {
        match manager.file_eof(file) {
            Ok(true) => break,
            Ok(false) => {}
            Err(_) => {
                manager.close_file(file).ok();
                close_dir_ignore_error(&manager, root);
                close_volume_ignore_error(&manager, volume);
                return file_read_report(FatStatus::FileReadFailed, path, empty_string());
            }
        }

        if content_len >= content.bytes.len() {
            manager.close_file(file).ok();
            close_dir_ignore_error(&manager, root);
            close_volume_ignore_error(&manager, volume);
            return file_read_report(FatStatus::ContentTooLong, path, empty_string());
        }

        let capacity = core::cmp::min(chunk.len(), content.bytes.len() - content_len);
        let read = match manager.read(file, &mut chunk[..capacity]) {
            Ok(read) => read,
            Err(_) => {
                manager.close_file(file).ok();
                close_dir_ignore_error(&manager, root);
                close_volume_ignore_error(&manager, volume);
                return file_read_report(FatStatus::FileReadFailed, path, empty_string());
            }
        };
        if read == 0 {
            break;
        }
        content.bytes[content_len..content_len + read].copy_from_slice(&chunk[..read]);
        content_len += read;
    }
    content.len = content_len as u8;

    if manager.close_file(file).is_err() {
        close_dir_ignore_error(&manager, root);
        close_volume_ignore_error(&manager, volume);
        return file_read_report(FatStatus::FileCloseFailed, path, empty_string());
    }

    let close_status = close_root(manager, volume, root);
    if matches!(close_status, FatStatus::Ready) {
        file_read_report(FatStatus::Ready, path, content)
    } else {
        file_read_report(close_status, path, empty_string())
    }
}

pub fn list_files(p: &Peripherals) -> FileListReport {
    let (manager, volume, root) = match open_root(p) {
        Ok(opened) => opened,
        Err(status) => return file_list_report(status, 0, empty_entries()),
    };

    let mut entries = empty_entries();
    let mut file_count = 0u8;
    if manager
        .iterate_dir(root, |entry| {
            if usize::from(file_count) < entries.len() {
                entries[usize::from(file_count)] = short_name_string(&entry.name);
            }
            file_count = file_count.saturating_add(1);
        })
        .is_err()
    {
        close_dir_ignore_error(&manager, root);
        close_volume_ignore_error(&manager, volume);
        return file_list_report(FatStatus::RootIterateFailed, 0, empty_entries());
    }

    let close_status = close_root(manager, volume, root);
    let sample_count = core::cmp::min(file_count, lisp::MAX_STORE_FILES as u8);
    file_list_report(close_status, sample_count, entries)
}

pub fn format_fat32_with_progress(
    p: &Peripherals,
    progress: &mut dyn FormatProgress,
) -> FormatReport {
    let sector_zero = micro_sd::read_sector_words(p, 0);
    if !matches!(sector_zero.status, micro_sd::ReadStatus::Ready) {
        return format_report(FatStatus::MbrReadFailed, empty_layout(), 0, 0);
    }

    let mut mbr = [0u8; Block::LEN];
    words_to_bytes(&sector_zero.words, &mut mbr);
    let mbr_signature = read_u16(&mbr, MBR_SIGNATURE_OFFSET);
    if mbr_signature != MBR_SIGNATURE {
        let layout = layout_from_mbr(0, 0, 0, 0, 0);
        return format_report(FatStatus::MissingMbrSignature, layout, 0, 0);
    }

    let partition = &mbr[PARTITION0_OFFSET..PARTITION0_OFFSET + 16];
    let partition_status = partition[PARTITION_STATUS_OFFSET];
    let partition_type_before = partition[PARTITION_TYPE_OFFSET];
    let partition_lba_start = read_u32(partition, PARTITION_LBA_OFFSET);
    let partition_sector_count = read_u32(partition, PARTITION_SECTORS_OFFSET);
    let layout = match compute_fat32_layout(
        partition_status,
        partition_type_before,
        partition_lba_start,
        partition_sector_count,
    ) {
        Some(layout) => layout,
        None => {
            return format_report(
                FatStatus::FormatGeometryInvalid,
                layout_from_mbr(
                    partition_status,
                    partition_type_before,
                    partition_lba_start,
                    partition_sector_count,
                    0,
                ),
                0,
                0,
            )
        }
    };

    let total_format_sectors = format_total_sectors(&layout);
    let mut written_sector_count = 0u32;
    let hidden_mbr = mbr_block(&layout, 0);
    if write_block(p, 0, &hidden_mbr).is_err() {
        return format_report(
            FatStatus::FormatMbrWriteFailed,
            layout,
            written_sector_count,
            0,
        );
    }
    written_sector_count += 1;
    progress.report(b"hide-mbr", written_sector_count, total_format_sectors);

    let first_fat_sector = layout.partition_lba_start + u32::from(FAT32_RESERVED_SECTORS);
    let fat_clear_count = layout.fat_size_sectors * u32::from(FAT32_FAT_COUNT);
    if let Err(failed_sector) = clear_range_with_progress(
        p,
        first_fat_sector,
        fat_clear_count,
        b"clear-fat",
        &mut written_sector_count,
        total_format_sectors,
        progress,
    ) {
        return format_report(
            FatStatus::FormatFatClearFailed,
            layout,
            written_sector_count,
            failed_sector,
        );
    }

    let fat_header = fat_header_block();
    if write_block(p, first_fat_sector, &fat_header).is_err() {
        return format_report(
            FatStatus::FormatFatHeaderWriteFailed,
            layout,
            written_sector_count,
            first_fat_sector,
        );
    }
    written_sector_count += 1;
    progress.report(b"fat-header", written_sector_count, total_format_sectors);

    let second_fat_sector = first_fat_sector + layout.fat_size_sectors;
    if write_block(p, second_fat_sector, &fat_header).is_err() {
        return format_report(
            FatStatus::FormatFatHeaderWriteFailed,
            layout,
            written_sector_count,
            second_fat_sector,
        );
    }
    written_sector_count += 1;
    progress.report(b"fat-header", written_sector_count, total_format_sectors);

    let first_data_sector = first_fat_sector + fat_clear_count;
    if let Err(failed_sector) = clear_range_with_progress(
        p,
        first_data_sector,
        u32::from(FAT32_SECTORS_PER_CLUSTER),
        b"clear-root",
        &mut written_sector_count,
        total_format_sectors,
        progress,
    ) {
        return format_report(
            FatStatus::FormatRootClearFailed,
            layout,
            written_sector_count,
            failed_sector,
        );
    }

    let boot = boot_sector_block(&layout);
    if write_block(p, layout.partition_lba_start, &boot).is_err() {
        return format_report(
            FatStatus::FormatBootWriteFailed,
            layout,
            written_sector_count,
            layout.partition_lba_start,
        );
    }
    written_sector_count += 1;
    progress.report(b"boot", written_sector_count, total_format_sectors);

    let fs_info = fs_info_block(&layout);
    let fs_info_sector = layout.partition_lba_start + u32::from(FAT32_FSINFO_SECTOR);
    if write_block(p, fs_info_sector, &fs_info).is_err() {
        return format_report(
            FatStatus::FormatFsInfoWriteFailed,
            layout,
            written_sector_count,
            fs_info_sector,
        );
    }
    written_sector_count += 1;
    progress.report(b"fsinfo", written_sector_count, total_format_sectors);

    let reserved_blank = Block::new();
    let reserved_sector = layout.partition_lba_start + 2;
    if write_block(p, reserved_sector, &reserved_blank).is_err() {
        return format_report(
            FatStatus::FormatBootWriteFailed,
            layout,
            written_sector_count,
            reserved_sector,
        );
    }
    written_sector_count += 1;
    progress.report(b"reserved", written_sector_count, total_format_sectors);

    let backup_boot_sector = layout.partition_lba_start + u32::from(FAT32_BACKUP_BOOT_SECTOR);
    if write_block(p, backup_boot_sector, &boot).is_err() {
        return format_report(
            FatStatus::FormatBootWriteFailed,
            layout,
            written_sector_count,
            backup_boot_sector,
        );
    }
    written_sector_count += 1;
    progress.report(b"backup-boot", written_sector_count, total_format_sectors);

    let backup_fs_info_sector = backup_boot_sector + 1;
    if write_block(p, backup_fs_info_sector, &fs_info).is_err() {
        return format_report(
            FatStatus::FormatFsInfoWriteFailed,
            layout,
            written_sector_count,
            backup_fs_info_sector,
        );
    }
    written_sector_count += 1;
    progress.report(b"backup-fsinfo", written_sector_count, total_format_sectors);

    let backup_reserved_sector = backup_boot_sector + 2;
    if write_block(p, backup_reserved_sector, &reserved_blank).is_err() {
        return format_report(
            FatStatus::FormatBootWriteFailed,
            layout,
            written_sector_count,
            backup_reserved_sector,
        );
    }
    written_sector_count += 1;
    progress.report(
        b"backup-reserved",
        written_sector_count,
        total_format_sectors,
    );

    let final_mbr = mbr_block(&layout, FAT32_LBA_PARTITION);
    if write_block(p, 0, &final_mbr).is_err() {
        return format_report(
            FatStatus::FormatMbrWriteFailed,
            layout,
            written_sector_count,
            0,
        );
    }
    written_sector_count += 1;
    progress.report(b"final-mbr", written_sector_count, total_format_sectors);

    format_report(FatStatus::Ready, layout, written_sector_count, 0)
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

fn format_report(
    status: FatStatus,
    layout: Fat32Layout,
    written_sector_count: u32,
    failed_sector: u32,
) -> FormatReport {
    FormatReport {
        status,
        mbr_signature: MBR_SIGNATURE,
        partition_status: layout.partition_status,
        partition_type_before: layout.partition_type_before,
        partition_type_after: if matches!(status, FatStatus::Ready) {
            FAT32_LBA_PARTITION
        } else {
            0
        },
        partition_lba_start: layout.partition_lba_start,
        partition_sector_count: layout.partition_sector_count,
        sectors_per_cluster: FAT32_SECTORS_PER_CLUSTER,
        reserved_sectors: FAT32_RESERVED_SECTORS,
        fat_count: FAT32_FAT_COUNT,
        fat_size_sectors: layout.fat_size_sectors,
        data_cluster_count: layout.data_cluster_count,
        root_cluster: FAT32_ROOT_CLUSTER,
        written_sector_count,
        failed_sector,
    }
}

fn clear_range_with_progress(
    p: &Peripherals,
    start_sector: u32,
    sector_count: u32,
    phase: &'static [u8],
    written_sector_count: &mut u32,
    total_format_sectors: u32,
    progress: &mut dyn FormatProgress,
) -> Result<(), u32> {
    let mut cleared = 0u32;
    while cleared < sector_count {
        let chunk = core::cmp::min(FORMAT_PROGRESS_CHUNK_SECTORS, sector_count - cleared);
        let sector = start_sector + cleared;
        let report = micro_sd::write_sector_range_fill(p, sector, chunk, 0);
        *written_sector_count += report.sectors_written;
        progress.report(phase, *written_sector_count, total_format_sectors);
        if !matches!(report.status, micro_sd::WriteStatus::Ready) {
            return Err(report.failed_sector);
        }
        cleared += chunk;
    }
    Ok(())
}

fn format_total_sectors(layout: &Fat32Layout) -> u32 {
    let fat_clear_count = layout.fat_size_sectors * u32::from(FAT32_FAT_COUNT);
    2 + fat_clear_count + 2 + u32::from(FAT32_SECTORS_PER_CLUSTER) + 6
}

fn compute_fat32_layout(
    partition_status: u8,
    partition_type_before: u8,
    partition_lba_start: u32,
    partition_sector_count: u32,
) -> Option<Fat32Layout> {
    if partition_lba_start == 0 || partition_sector_count < 1024 {
        return None;
    }

    let mut fat_size_sectors = 1u32;
    loop {
        let non_data_sectors = u32::from(FAT32_RESERVED_SECTORS)
            .checked_add(u32::from(FAT32_FAT_COUNT).checked_mul(fat_size_sectors)?)?;
        if non_data_sectors >= partition_sector_count {
            return None;
        }

        let data_sectors = partition_sector_count - non_data_sectors;
        let data_cluster_count = data_sectors / u32::from(FAT32_SECTORS_PER_CLUSTER);
        let needed_fat_sectors = div_ceil(
            data_cluster_count.checked_add(2)?.checked_mul(4)?,
            Block::LEN_U32,
        );
        if needed_fat_sectors == fat_size_sectors {
            if !(FAT32_MIN_CLUSTERS..=FAT32_MAX_CLUSTERS).contains(&data_cluster_count) {
                return None;
            }
            return Some(Fat32Layout {
                partition_status,
                partition_type_before,
                partition_lba_start,
                partition_sector_count,
                fat_size_sectors,
                data_cluster_count,
            });
        }
        fat_size_sectors = needed_fat_sectors;
    }
}

fn layout_from_mbr(
    partition_status: u8,
    partition_type_before: u8,
    partition_lba_start: u32,
    partition_sector_count: u32,
    fat_size_sectors: u32,
) -> Fat32Layout {
    Fat32Layout {
        partition_status,
        partition_type_before,
        partition_lba_start,
        partition_sector_count,
        fat_size_sectors,
        data_cluster_count: 0,
    }
}

fn empty_layout() -> Fat32Layout {
    layout_from_mbr(0, 0, 0, 0, 0)
}

fn mbr_block(layout: &Fat32Layout, partition_type: u8) -> Block {
    let mut block = Block::new();
    let entry = PARTITION0_OFFSET;
    block.contents[entry + PARTITION_STATUS_OFFSET] = 0;
    block.contents[entry + 1] = 0;
    block.contents[entry + 2] = 2;
    block.contents[entry + 3] = 0;
    block.contents[entry + PARTITION_TYPE_OFFSET] = partition_type;
    block.contents[entry + 5] = 0xff;
    block.contents[entry + 6] = 0xff;
    block.contents[entry + 7] = 0xff;
    write_u32(
        &mut block.contents,
        entry + PARTITION_LBA_OFFSET,
        layout.partition_lba_start,
    );
    write_u32(
        &mut block.contents,
        entry + PARTITION_SECTORS_OFFSET,
        layout.partition_sector_count,
    );
    write_u16(&mut block.contents, MBR_SIGNATURE_OFFSET, MBR_SIGNATURE);
    block
}

fn boot_sector_block(layout: &Fat32Layout) -> Block {
    let mut block = Block::new();
    block.contents[0] = 0xeb;
    block.contents[1] = 0x58;
    block.contents[2] = 0x90;
    block.contents[3..11].copy_from_slice(b"MSDOS5.0");
    write_u16(&mut block.contents, 11, Block::LEN as u16);
    block.contents[13] = FAT32_SECTORS_PER_CLUSTER;
    write_u16(&mut block.contents, 14, FAT32_RESERVED_SECTORS);
    block.contents[16] = FAT32_FAT_COUNT;
    write_u16(&mut block.contents, 17, 0);
    write_u16(&mut block.contents, 19, 0);
    block.contents[21] = 0xf8;
    write_u16(&mut block.contents, 22, 0);
    write_u16(&mut block.contents, 24, 63);
    write_u16(&mut block.contents, 26, 255);
    write_u32(&mut block.contents, 28, layout.partition_lba_start);
    write_u32(&mut block.contents, 32, layout.partition_sector_count);
    write_u32(&mut block.contents, 36, layout.fat_size_sectors);
    write_u16(&mut block.contents, 40, 0);
    write_u16(&mut block.contents, 42, 0);
    write_u32(&mut block.contents, 44, FAT32_ROOT_CLUSTER);
    write_u16(&mut block.contents, 48, FAT32_FSINFO_SECTOR);
    write_u16(&mut block.contents, 50, FAT32_BACKUP_BOOT_SECTOR);
    block.contents[64] = 0x80;
    block.contents[66] = 0x29;
    write_u32(&mut block.contents, 67, VOLUME_ID);
    block.contents[71..82].copy_from_slice(VOLUME_LABEL);
    block.contents[82..90].copy_from_slice(b"FAT32   ");
    write_u16(&mut block.contents, MBR_SIGNATURE_OFFSET, MBR_SIGNATURE);
    block
}

fn fs_info_block(layout: &Fat32Layout) -> Block {
    let mut block = Block::new();
    write_u32(&mut block.contents, 0, 0x4161_5252);
    write_u32(&mut block.contents, 484, 0x6141_7272);
    write_u32(
        &mut block.contents,
        488,
        layout.data_cluster_count.saturating_sub(1),
    );
    write_u32(&mut block.contents, 492, 3);
    write_u32(&mut block.contents, 508, 0xaa55_0000);
    block
}

fn fat_header_block() -> Block {
    let mut block = Block::new();
    write_u32(&mut block.contents, 0, 0x0fff_fff8);
    write_u32(&mut block.contents, 4, 0xffff_ffff);
    write_u32(&mut block.contents, 8, 0x0fff_ffff);
    block
}

fn write_block(p: &Peripherals, sector: u32, block: &Block) -> Result<(), ()> {
    let words = bytes_to_words(&block.contents);
    let report = micro_sd::write_sector_words(p, sector, &words);
    if matches!(report.status, micro_sd::WriteStatus::Ready) {
        Ok(())
    } else {
        Err(())
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

fn file_open_error_status(error: FatError<FatBlockError>) -> FatStatus {
    match error {
        FatError::DeviceError(_) => FatStatus::BlockDeviceFailed,
        FatError::FilenameError(_) => FatStatus::InvalidPath,
        FatError::NotFound => FatStatus::NotFound,
        _ => FatStatus::FileOpenFailed,
    }
}

fn open_root(
    p: &Peripherals,
) -> Result<
    (
        VolumeManager<PsocBlockDevice<'_>, FixedTime, 2, 1, 1>,
        RawVolume,
        RawDirectory,
    ),
    FatStatus,
> {
    let device = PsocBlockDevice { p };
    let manager: VolumeManager<_, _, 2, 1, 1> =
        VolumeManager::new_with_limits(device, FixedTime, 7000);
    let volume = manager
        .open_raw_volume(VolumeIdx(0))
        .map_err(open_error_status)?;
    let root = match manager.open_root_dir(volume) {
        Ok(root) => root,
        Err(error) => {
            close_volume_ignore_error(&manager, volume);
            return Err(root_error_status(error));
        }
    };
    Ok((manager, volume, root))
}

fn close_root(
    manager: VolumeManager<PsocBlockDevice<'_>, FixedTime, 2, 1, 1>,
    volume: RawVolume,
    root: RawDirectory,
) -> FatStatus {
    if manager.close_dir(root).is_err() {
        close_volume_ignore_error(&manager, volume);
        return FatStatus::RootCloseFailed;
    }
    if manager.close_volume(volume).is_err() {
        return FatStatus::VolumeCloseFailed;
    }
    FatStatus::Ready
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

fn path_short_name(path: lisp::StringBytes) -> Result<ShortFileName, FatStatus> {
    let path_len = path.len as usize;
    if path_len == 0 {
        return Err(FatStatus::EmptyPath);
    }
    if path_len > lisp::MAX_STRING_BYTES {
        return Err(FatStatus::PathTooLong);
    }
    let path_bytes = &path.bytes[..path_len];
    if let Some(path_text) = mapped_lisp_short_path(path_bytes) {
        let path_text = str::from_utf8(&path_text.bytes[..path_text.len])
            .map_err(|_| FatStatus::InvalidPath)?;
        return ShortFileName::create_from_str(path_text).map_err(|_| FatStatus::InvalidPath);
    }
    let path_text = str::from_utf8(path_bytes).map_err(|_| FatStatus::InvalidPath)?;
    ShortFileName::create_from_str(path_text).map_err(|_| FatStatus::InvalidPath)
}

fn mapped_lisp_short_path(path: &[u8]) -> Option<ShortPathText> {
    let dot_index = lisp_extension_dot_index(path)?;
    if dot_index == 0 || dot_index > 8 {
        return None;
    }

    let mut mapped = ShortPathText {
        len: dot_index + 4,
        bytes: [0; MAX_SHORT_PATH_TEXT_BYTES],
    };
    mapped.bytes[..dot_index].copy_from_slice(&path[..dot_index]);
    mapped.bytes[dot_index] = b'.';
    mapped.bytes[dot_index + 1..dot_index + 4].copy_from_slice(FAT_LISP_EXTENSION);
    Some(mapped)
}

fn lisp_extension_dot_index(path: &[u8]) -> Option<usize> {
    if path.len() < LISP_EXTENSION.len() + 2 {
        return None;
    }
    let dot_index = path.len() - LISP_EXTENSION.len() - 1;
    if path[dot_index] != b'.' {
        return None;
    }
    if ascii_eq_ignore_case(&path[dot_index + 1..], LISP_EXTENSION) {
        Some(dot_index)
    } else {
        None
    }
}

fn file_write_report(
    status: FatStatus,
    path: lisp::StringBytes,
    content: lisp::StringBytes,
) -> FileWriteReport {
    FileWriteReport {
        status,
        path_len: path.len,
        content_len: if matches!(status, FatStatus::Ready) {
            content.len
        } else {
            0
        },
    }
}

fn file_read_report(
    status: FatStatus,
    path: lisp::StringBytes,
    content: lisp::StringBytes,
) -> FileReadReport {
    FileReadReport {
        status,
        path_len: path.len,
        content_len: if matches!(status, FatStatus::Ready) {
            content.len
        } else {
            0
        },
        content,
    }
}

fn file_list_report(
    status: FatStatus,
    file_count: u8,
    files: [lisp::StringBytes; lisp::MAX_STORE_FILES],
) -> FileListReport {
    FileListReport {
        status,
        file_count: if matches!(status, FatStatus::Ready) {
            file_count
        } else {
            0
        },
        files,
    }
}

fn short_name_string(name: &ShortFileName) -> lisp::StringBytes {
    let mut out = empty_string();
    let mut len = 0usize;
    let display_lisp_extension = ascii_eq_ignore_case(name.extension(), FAT_LISP_EXTENSION);
    for &byte in name.base_name() {
        if display_lisp_extension {
            push_byte(&mut out, &mut len, ascii_lower_byte(byte));
        } else {
            push_byte(&mut out, &mut len, byte);
        }
    }
    let extension = name.extension();
    if !extension.is_empty() {
        push_byte(&mut out, &mut len, b'.');
        if display_lisp_extension {
            for &byte in LISP_EXTENSION {
                push_byte(&mut out, &mut len, byte);
            }
        } else {
            for &byte in extension {
                push_byte(&mut out, &mut len, byte);
            }
        }
    }
    out.len = len as u8;
    out
}

fn ascii_eq_ignore_case(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0usize;
    while index < left.len() {
        if ascii_lower_byte(left[index]) != ascii_lower_byte(right[index]) {
            return false;
        }
        index += 1;
    }
    true
}

fn ascii_lower_byte(byte: u8) -> u8 {
    if byte.is_ascii_uppercase() {
        byte + (b'a' - b'A')
    } else {
        byte
    }
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

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset] = value as u8;
    bytes[offset + 1] = (value >> 8) as u8;
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

fn div_ceil(value: u32, divisor: u32) -> u32 {
    (value + divisor - 1) / divisor
}
