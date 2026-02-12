use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::mem::size_of;
use std::path::Path;
use std::process::Command;
use std::{env, fs};

const MAGIC_MARKER: [u8; 10] = *b"REX_BUNDLE";

#[repr(C, packed)]
struct BundleMetadata {
    payload_size: u64,
    target_bin_name_len: u32,
}

const _: () = assert!(size_of::<BundleMetadata>() == 12);

struct PayloadInfo {
    metadata: BundleMetadata,
    payload_start_offset: u64,
    target_binary_name: String,
}

pub struct Runtime {
    payload_info: Option<PayloadInfo>,
    executed: bool,
}

impl Runtime {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let payload_info = Self::find_payload_info()?;
        Ok(Self {
            payload_info,
            executed: false,
        })
    }

    pub fn is_bundled(&self) -> bool {
        self.payload_info.is_some()
    }

    pub fn has_run(&self) -> bool {
        self.executed
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        #[cfg(debug_assertions)]
        {
            let args: Vec<String> = env::args().collect();
            if args.len() > 1 && args[1] == "--rex-extract" {
                if let Some(info) = &self.payload_info {
                    let current_dir = env::current_dir()?;
                    println!("[rex] Extracting bundle to {}", current_dir.display());
                    Self::extract_payload(info, &current_dir)?;
                    println!("[rex] Extraction completed successfully!");
                    return Ok(());
                }
            }
        }

        self.payload_info
            .take()
            .map_or(Ok(()), |info| self.run_bundled_binary(&info))
    }

    fn find_payload_info() -> Result<Option<PayloadInfo>, Box<dyn Error>> {
        let exec = env::current_exe()?;
        let mut file = File::open(&exec)?;
        let file_size = file.metadata()?.len();

        const FIXED_METADATA_SIZE: u64 =
            size_of::<BundleMetadata>() as u64 + MAGIC_MARKER.len() as u64;

        let start_pos = file_size.saturating_sub(FIXED_METADATA_SIZE + 256);
        file.seek(SeekFrom::Start(start_pos))?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let marker_idx = buffer
            .windows(MAGIC_MARKER.len())
            .rposition(|w| w == MAGIC_MARKER);
        let marker_pos = match marker_idx {
            Some(idx) => start_pos + idx as u64,
            None => return Ok(None),
        };

        let meta_pos = marker_pos
            .checked_sub(size_of::<BundleMetadata>() as u64)
            .ok_or("Invalid metadata")?;
        file.seek(SeekFrom::Start(meta_pos))?;
        let mut meta_bytes = [0u8; size_of::<BundleMetadata>()];
        file.read_exact(&mut meta_bytes)?;

        let payload_size = u64::from_le_bytes(meta_bytes[0..8].try_into().unwrap());
        let name_len = u32::from_le_bytes(meta_bytes[8..12].try_into().unwrap()) as u64;

        let name_pos = meta_pos
            .checked_sub(name_len)
            .ok_or("Invalid name offset")?;
        file.seek(SeekFrom::Start(name_pos))?;
        let mut name_bytes = vec![0u8; name_len as usize];
        file.read_exact(&mut name_bytes)?;
        let target_binary_name = String::from_utf8(name_bytes)?;

        let payload_start_offset = name_pos
            .checked_sub(payload_size)
            .ok_or("Invalid payload offset")?;

        Ok(Some(PayloadInfo {
            metadata: BundleMetadata {
                payload_size,
                target_bin_name_len: name_len as u32,
            },
            payload_start_offset,
            target_binary_name,
        }))
    }

    fn extract_payload(info: &PayloadInfo, dest_path: &Path) -> Result<(), Box<dyn Error>> {
        let exec = env::current_exe()?;
        let mut file = File::open(&exec)?;
        file.seek(SeekFrom::Start(info.payload_start_offset))?;

        let payload_reader = file.take(info.metadata.payload_size);
        let decoder = zstd::Decoder::new(payload_reader)?;
        let mut archive = tar_minimal::Decoder::new(decoder);
        archive.unpack(&dest_path.display().to_string())?;
        Ok(())
    }

    fn run_bundled_binary(&mut self, info: &PayloadInfo) -> Result<(), Box<dyn Error>> {
        let extraction_root = env::temp_dir();
        Self::extract_payload(info, extraction_root.as_path())?;

        let bundle_dir = extraction_root.join(format!("{}_bundle", info.target_binary_name));
        let bin_dir = bundle_dir.join("bins");
        let libs_dir = bundle_dir.join("libs");
        let target_bin_path = bundle_dir.join(&info.target_binary_name);

        let loader = fs::read_dir(&libs_dir)?
            .filter_map(|entry| entry.ok())
            .map(|e| e.path())
            .find(|p| {
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                name.starts_with("ld-linux") || name.starts_with("ld-musl")
            })
            .ok_or("No compatible loader found")?;

        if bin_dir.exists() {
            let existing = env::var("PATH").unwrap_or_default();
            let new_path = format!("{}:{}", bin_dir.display(), existing);
            unsafe {
                env::set_var("PATH", new_path);
            }
        }

        let args: Vec<String> = env::args().skip(1).collect();
        let mut cmd_args = vec![
            "--library-path".to_string(),
            libs_dir.to_string_lossy().into(),
            target_bin_path.to_string_lossy().into(),
        ];
        cmd_args.extend(args);

        let result = Command::new(loader)
            .args(&cmd_args)
            .current_dir(&bundle_dir)
            .status();

        self.executed = true;
        let _ = fs::remove_dir_all(&bundle_dir);

        match result {
            Ok(s) if s.success() => Ok(()),
            Ok(_) => Err("fail".into()),
            Err(e) => Err(format!("Failed to execute: {e}").into()),
        }
    }
}
