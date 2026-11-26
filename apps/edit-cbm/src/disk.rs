//! Commodore disk images (.D64, .D71, .D81) for File > Open from Image
//! and Save to Image, using the cbm-disk crate: CBM DOS, with the 1581's
//! partitions as directories.

use cbm_disk::{DiskImage, Error, FileSystem, FileType, Layout, Volume};
use retro_core::disk::{
    DateTime, DiskEntry, DiskError, DiskErrorKind, DiskImages, DiskResult, DiskSize, DiskVolume,
    NewDisk,
};

/// CBM DOS's separator between a 1581 partition and what's in it.
const SEPARATOR: u8 = b'/';

/// The types a file can be saved as, in the order Save to Image shows
/// them.
const FILE_TYPES: [&str; 3] = ["PRG", "SEQ", "USR"];

/// The least size of a 1581 partition used as a directory, in tracks
/// (120 blocks), also what Create Directory offers first.
const MIN_DIR_TRACKS: u32 = 3;

pub struct CbmDisks;

/// New Image's drives; the image type follows from the drive.
const SIZES: &[DiskSize] = &[
    DiskSize {
        label: "1541 (D64)",
        sector_size: Some(256),
    },
    DiskSize {
        label: "1571 (D71)",
        sector_size: Some(256),
    },
    DiskSize {
        label: "1581 (D81)",
        sector_size: Some(256),
    },
];

impl DiskImages for CbmDisks {
    fn name(&self) -> &'static str {
        "Commodore disk images"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["D64", "D71", "D81"]
    }

    fn open(&self, bytes: &[u8]) -> DiskResult<Box<dyn DiskVolume>> {
        let vol = Volume::open(DiskImage::from_bytes(bytes).map_err(error)?).map_err(error)?;
        match vol.file_system() {
            FileSystem::CbmDos => Ok(Box::new(CbmVolume(vol))),
            FileSystem::CpM => Err(DiskError::other(
                "it's a CP/M disk: sectors only, no files the editor can open",
            )),
            _ => Err(DiskError::other(
                "the disk has no CBM DOS file system: it may be unformatted, or a \
                 game's own disk",
            )),
        }
    }

    fn new_systems(&self) -> &'static [&'static str] {
        &["CBM DOS"]
    }

    fn new_sizes(&self, _system: usize) -> &'static [DiskSize] {
        SIZES
    }

    fn has_volume_name(&self, _system: usize) -> bool {
        true
    }

    fn volume_name_max(&self, _system: usize) -> usize {
        16
    }

    fn volume_name_label(&self) -> &'static str {
        "Disk name:"
    }

    fn has_volume_id(&self, _system: usize) -> bool {
        true
    }

    /// A 1541 disk is a D64, a 1571 disk a D71, a 1581 disk a D81.
    fn fixed_container(&self, _system: usize, size: usize) -> Option<usize> {
        Some(size)
    }

    /// As the drive's `N0:NAME,ID` makes it. An ID left empty is "00", as
    /// one character it's padded with a space.
    fn format(&self, new: &NewDisk) -> DiskResult<Box<dyn DiskVolume>> {
        let layout = match new.size {
            0 => Layout::D64,
            1 => Layout::D71,
            2 => Layout::D81,
            _ => return Err(DiskError::other("no such disk")),
        };
        let id = match new.volume_id.as_slice() {
            [] => *b"00",
            [a] => [*a, b' '],
            [a, b, ..] => [*a, *b],
        };
        let vol = Volume::format(layout, &new.volume_name, id).map_err(error)?;
        Ok(Box::new(CbmVolume(vol)))
    }
}

struct CbmVolume(Volume);

/// The drive a disk is for, as the info line shows it.
fn drive(layout: Layout) -> &'static str {
    match layout {
        Layout::D64 => "1541 (D64)",
        Layout::D64_40 => "1541, 40 tracks (D64)",
        Layout::D71 => "1571 (D71)",
        Layout::D81 => "1581 (D81)",
        _ => "Commodore disk",
    }
}

/// `GAMES/PACMAN` from `["GAMES", "PACMAN"]`.
fn join(path: &[Vec<u8>]) -> Vec<u8> {
    path.join(&SEPARATOR)
}

fn error(e: Error) -> DiskError {
    let kind = match e {
        Error::AlreadyExists(_) => DiskErrorKind::AlreadyExists,
        Error::Locked(_) => DiskErrorKind::Locked,
        Error::NotFound(_) => DiskErrorKind::NotFound,
        _ => DiskErrorKind::Other,
    };
    DiskError::new(kind, e.to_string())
}

impl DiskVolume for CbmVolume {
    /// "412 blocks free, 1541 (D64)".
    fn info(&self) -> String {
        let geos = if self.0.is_geos() { ", GEOS" } else { "" };
        format!(
            "{} blocks free, {}{geos}",
            self.0.blocks_free(),
            drive(self.0.layout())
        )
    }

    fn volume_name(&self) -> Option<Vec<u8>> {
        self.0.disk_name()
    }

    /// `"GAMES" 01`, as a directory listing's first line has them.
    fn title(&self) -> Option<Vec<u8>> {
        let mut t = vec![b'"'];
        t.extend(self.0.disk_name()?);
        t.push(b'"');
        if let Some(id) = self.0.disk_id() {
            t.push(b' ');
            t.extend(id);
        }
        Some(t)
    }

    fn has_directories(&self) -> bool {
        self.0.features().directories
    }

    fn has_dates(&self) -> bool {
        false
    }

    /// D64, D71 and D81 files have no write-protect flag (a read-only
    /// image file is treated as write-protected by the editor).
    fn write_protected(&self) -> bool {
        false
    }

    fn file_types(&self) -> &'static [&'static str] {
        &FILE_TYPES
    }

    fn names_have_extensions(&self) -> bool {
        false
    }

    fn name_max(&self) -> usize {
        16
    }

    fn dir_size(&self) -> Option<(u32, u32)> {
        self.0
            .features()
            .directories
            .then_some((MIN_DIR_TRACKS, MIN_DIR_TRACKS))
    }

    fn list(&self, dir: &[Vec<u8>]) -> DiskResult<Vec<DiskEntry>> {
        let entries = self.0.list(join(dir)).map_err(error)?;
        Ok(entries
            .into_iter()
            .map(|e| DiskEntry {
                is_dir: e.is_dir(),
                size: e.size,
                sectors: u32::from(e.blocks),
                locked: e.locked,
                modified: None,
                damaged: e.damaged,
                kind: e.type_text(),
                name: e.name,
            })
            .collect())
    }

    fn read(&self, path: &[Vec<u8>]) -> DiskResult<Vec<u8>> {
        self.0.read(join(path)).map_err(error)
    }

    fn write(
        &mut self,
        path: &[Vec<u8>],
        data: &[u8],
        replace: bool,
        _when: Option<DateTime>,
        file_type: usize,
    ) -> DiskResult<()> {
        let file_type = match FILE_TYPES.get(file_type) {
            Some(&"SEQ") => FileType::Seq,
            Some(&"USR") => FileType::Usr,
            _ => FileType::Prg,
        };
        self.0.set_overwrite(replace);
        let done = self.0.write(join(path), file_type, data).map_err(error);
        self.0.set_overwrite(false);
        done
    }

    fn make_dir(&mut self, path: &[Vec<u8>], _when: Option<DateTime>, size: u32) -> DiskResult<()> {
        let tracks = u8::try_from(size.max(MIN_DIR_TRACKS))
            .map_err(|_| DiskError::other(format!("{size} tracks is more than the disk has")))?;
        self.0.make_dir(join(path), tracks).map_err(error)
    }

    fn check_name(&self, name: &[u8]) -> DiskResult<()> {
        self.0.check_name(name).map_err(error)
    }

    /// Any PETSCII character but a quote (which would end the name in a
    /// command) and the shifted space that pads names.
    fn name_char(&self, code: u8) -> Option<u8> {
        match code {
            b'"' | 0xA0 => None,
            0x20..=0x7F | 0xA1..=0xFF => Some(code),
            _ => None,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.image().to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(names: &[&str]) -> Vec<Vec<u8>> {
        names.iter().map(|n| n.as_bytes().to_vec()).collect()
    }

    fn new_disk(size: usize, name: &[u8], id: &[u8]) -> Box<dyn DiskVolume> {
        CbmDisks
            .format(&NewDisk {
                system: 0,
                size,
                sectors: 0,
                sector_size: 256,
                container: size,
                volume_name: name.to_vec(),
                volume_id: id.to_vec(),
            })
            .unwrap()
    }

    #[test]
    fn write_then_read_back_through_the_image_bytes() {
        let mut vol = new_disk(0, b"TEST DISK", b"42");
        assert_eq!(vol.title(), Some(b"\"TEST DISK\" 42".to_vec()));
        assert_eq!(vol.info(), "664 blocks free, 1541 (D64)");
        vol.write(&p(&["HELLO"]), b"\x01\x08\x00\x00", false, None, 0)
            .unwrap();
        vol.write(&p(&["NOTES"]), b"HI\r", false, None, 1).unwrap();
        let again = CbmDisks.open(&vol.to_bytes()).unwrap();
        let list = again.list(&[]).unwrap();
        let kinds: Vec<&str> = list.iter().map(|e| e.kind.as_str()).collect();
        assert_eq!(kinds, ["PRG", "SEQ"]);
        assert_eq!(again.read(&p(&["NOTES"])).unwrap(), b"HI\r");
        // Already there: refused unless replacing.
        let mut vol = again;
        let e = vol.write(&p(&["NOTES"]), b"X", false, None, 1).unwrap_err();
        assert_eq!(e.kind, DiskErrorKind::AlreadyExists);
        vol.write(&p(&["NOTES"]), b"X", true, None, 1).unwrap();
        assert_eq!(vol.read(&p(&["NOTES"])).unwrap(), b"X");
    }

    #[test]
    fn partitions_on_a_1581() {
        let mut vol = new_disk(2, b"BIG", b"81");
        assert!(vol.has_directories());
        assert_eq!(vol.dir_size(), Some((3, 3)));
        vol.make_dir(&p(&["GAMES"]), None, 3).unwrap();
        vol.write(&p(&["GAMES", "PAC"]), b"\x01\x08\x00\x00", false, None, 0)
            .unwrap();
        assert_eq!(
            vol.read(&p(&["GAMES", "PAC"])).unwrap(),
            b"\x01\x08\x00\x00"
        );
        assert!(vol.list(&[]).unwrap()[0].is_dir);
        // A 1541 disk has none.
        assert!(!new_disk(0, b"", b"").has_directories());
    }

    #[test]
    fn empty_id_is_00() {
        let vol = new_disk(1, b"X", b"");
        assert_eq!(vol.title(), Some(b"\"X\" 00".to_vec()));
        assert_eq!(vol.info(), "1328 blocks free, 1571 (D71)");
    }
}
