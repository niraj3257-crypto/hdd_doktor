use anyhow::{anyhow, Context, Result};
use std::process::Command;

#[cfg(windows)]
fn cmd(program: &str) -> Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let mut c = Command::new(program);
    c.creation_flags(CREATE_NO_WINDOW);
    c
}

#[cfg(not(windows))]
fn cmd(program: &str) -> Command {
    Command::new(program)
}

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WipePattern {
    Zero,
    Random,
    DoD3,
    Gutmann,
}

impl WipePattern {
    pub fn etiket(&self, dil: crate::lang::Dil) -> &'static str {
        match self {
            Self::Zero => dil.wp_zero(),
            Self::Random => dil.wp_random(),
            Self::DoD3 => dil.wp_dod(),
            Self::Gutmann => dil.wp_gutmann(),
        }
    }

    pub fn gecis_sayisi(&self) -> usize {
        match self {
            Self::Zero => 1,
            Self::Random => 1,
            Self::DoD3 => 3,
            Self::Gutmann => 35,
        }
    }
}

fn xorshift(seed: &mut u64) -> u64 {
    let mut x = *seed;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *seed = x;
    x
}

fn buffer_doldur(buf: &mut [u8], pattern: WipePattern, gecis: usize, seed: &mut u64) {
    match (pattern, gecis) {
        (WipePattern::Zero, _) => buf.fill(0),
        (WipePattern::Random, _) => {
            for chunk in buf.chunks_mut(8) {
                let r = xorshift(seed).to_le_bytes();
                for (i, b) in chunk.iter_mut().enumerate() {
                    *b = r[i];
                }
            }
        }
        (WipePattern::DoD3, 0) => buf.fill(0),
        (WipePattern::DoD3, 1) => buf.fill(0xFF),
        (WipePattern::DoD3, _) => {
            for chunk in buf.chunks_mut(8) {
                let r = xorshift(seed).to_le_bytes();
                for (i, b) in chunk.iter_mut().enumerate() {
                    *b = r[i];
                }
            }
        }
        (WipePattern::Gutmann, g) => {
            let sabit = [
                0x55, 0xAA, 0x92, 0x49, 0x24, 0x00, 0x11, 0x22, 0x33, 0x44,
                0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
                0x92, 0x49, 0x24, 0x6D, 0xB6, 0xDB,
            ];
            if g < 4 || g >= 31 {
                for chunk in buf.chunks_mut(8) {
                    let r = xorshift(seed).to_le_bytes();
                    for (i, b) in chunk.iter_mut().enumerate() {
                        *b = r[i];
                    }
                }
            } else {
                let idx = (g - 4) % sabit.len();
                buf.fill(sabit[idx]);
            }
        }
    }
}

#[cfg(windows)]
pub fn wipe(
    disk: &str,
    pattern: WipePattern,
    iptal: Arc<AtomicBool>,
    progress: impl Fn(f32, String) + Send + Sync,
) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::os::windows::fs::OpenOptionsExt;

    let windows_path = disk_yolu_windows(disk)?;
    let boyut = disk_boyutu_windows(&windows_path)?;
    let gecis_toplam = pattern.gecis_sayisi();

    const FILE_FLAG_NO_BUFFERING: u32 = 0x20000000;
    const FILE_FLAG_WRITE_THROUGH: u32 = 0x80000000;
    const BUF_SIZE: usize = 4 * 1024 * 1024;

    let mut seed: u64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0xDEADBEEF);

    for gecis in 0..gecis_toplam {
        let mut file = OpenOptions::new()
            .write(true)
            .custom_flags(FILE_FLAG_NO_BUFFERING | FILE_FLAG_WRITE_THROUGH)
            .open(&windows_path)
            .context("Cannot open disk. Run as administrator.")?;

        let mut buf = vec![0u8; BUF_SIZE];
        buffer_doldur(&mut buf, pattern, gecis, &mut seed);

        let mut yazilan: u64 = 0;
        let mut bad_count = 0u64;
        let baslangic = std::time::Instant::now();

        while yazilan < boyut {
            if iptal.load(Ordering::Relaxed) {
                let oran = ((gecis as f32) + yazilan as f32 / boyut as f32) / gecis_toplam as f32;
                progress(oran, format!("CANCEL. Pass {}/{}, {} MB", gecis + 1, gecis_toplam, yazilan / 1_048_576));
                return Err(anyhow!("Cancelled"));
            }

            let kalan = (boyut - yazilan) as usize;
            let yaz = kalan.min(BUF_SIZE);
            match file.write_all(&buf[..yaz]) {
                Ok(_) => yazilan += yaz as u64,
                Err(_) => {
                    bad_count += 1;
                    yazilan += yaz as u64;
                }
            }

            if yazilan % (BUF_SIZE as u64 * 16) == 0 || yazilan == boyut {
                let gecis_oran = yazilan as f32 / boyut as f32;
                let toplam_oran = ((gecis as f32) + gecis_oran) / gecis_toplam as f32;
                let gecen = baslangic.elapsed().as_secs_f32().max(0.1);
                let hiz = yazilan as f32 / gecen / 1_048_576.0;
                let mb = yazilan / 1_048_576;
                let toplam_mb = boyut / 1_048_576;
                let kalan_sn = if hiz > 0.0 {
                    ((boyut - yazilan) as f32 / 1_048_576.0 / hiz) as u32
                } else { 0 };
                progress(
                    toplam_oran,
                    format!(
                        "Pass {}/{}: {}/{} MB - {:.1} MB/s - ETA: {} min - Err: {}",
                        gecis + 1, gecis_toplam, mb, toplam_mb, hiz, kalan_sn / 60, bad_count
                    ),
                );
            }
        }

        drop(file);
    }

    progress(1.0, format!("Done: {} pass(es)", gecis_toplam));
    Ok(())
}

#[cfg(not(windows))]
pub fn wipe(
    _disk: &str,
    _pattern: WipePattern,
    _iptal: Arc<AtomicBool>,
    _progress: impl Fn(f32, String) + Send + Sync,
) -> Result<()> {
    Err(anyhow!("wipe sadece Windows'ta destekleniyor"))
}

pub fn chkdsk_calistir(surucu_harfi: &str, tam_tara: bool) -> Result<String> {
    let mut args = vec!["-Command".to_string()];
    let bayrak = if tam_tara { "/r /f" } else { "/f" };
    let ps = format!(
        "$out = & chkdsk {}: {} 2>&1 | Out-String; Write-Output $out",
        surucu_harfi.trim_end_matches(':').trim_end_matches('\\'),
        bayrak
    );
    args.push(ps);

    let out = std::process::Command::new("powershell")
        .args(&args)
        .output()?;

    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();

    let mut r = String::new();
    r.push_str(&stdout);
    if !stderr.trim().is_empty() {
        r.push_str("\n---STDERR---\n");
        r.push_str(&stderr);
    }
    Ok(r)
}

#[cfg(windows)]
pub fn zero_fill(
    disk: &str,
    iptal: Arc<AtomicBool>,
    progress: impl Fn(f32, String) + Send,
) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::os::windows::fs::OpenOptionsExt;

    let windows_path = disk_yolu_windows(disk)?;
    let boyut = disk_boyutu_windows(&windows_path)?;

    const FILE_FLAG_NO_BUFFERING: u32 = 0x20000000;
    const FILE_FLAG_WRITE_THROUGH: u32 = 0x80000000;

    let mut file = OpenOptions::new()
        .write(true)
        .custom_flags(FILE_FLAG_NO_BUFFERING | FILE_FLAG_WRITE_THROUGH)
        .open(&windows_path)
        .context("Cannot open disk. Run as administrator.")?;

    const BUF_SIZE: usize = 4 * 1024 * 1024;
    let buf = vec![0u8; BUF_SIZE];

    let mut yazilan: u64 = 0;
    let mut bad_count = 0u64;
    let baslangic = std::time::Instant::now();

    while yazilan < boyut {
        if iptal.load(Ordering::Relaxed) {
            let oran = yazilan as f32 / boyut as f32;
            progress(oran, format!("CANCELLED. {} MB written.", yazilan / 1_048_576));
            return Err(anyhow!("Cancelled"));
        }

        let kalan = (boyut - yazilan) as usize;
        let yaz = kalan.min(BUF_SIZE);
        match file.write_all(&buf[..yaz]) {
            Ok(_) => yazilan += yaz as u64,
            Err(_) => {
                bad_count += 1;
                yazilan += yaz as u64;
            }
        }

        if yazilan % (BUF_SIZE as u64 * 16) == 0 || yazilan == boyut {
            let oran = yazilan as f32 / boyut as f32;
            let gecen = baslangic.elapsed().as_secs_f32().max(0.1);
            let hiz = yazilan as f32 / gecen / 1_048_576.0;
            let mb = yazilan / 1_048_576;
            let toplam_mb = boyut / 1_048_576;
            let kalan_saniye = if hiz > 0.0 {
                ((boyut - yazilan) as f32 / 1_048_576.0 / hiz) as u32
            } else {
                0
            };
            progress(
                oran,
                format!(
                    "{}/{} MB - {:.1} MB/s - Kalan: {} dk - Hata: {}",
                    mb, toplam_mb, hiz, kalan_saniye / 60, bad_count
                ),
            );
        }
    }

    progress(1.0, format!("Done. {} bad sector(s) remapped.", bad_count));
    Ok(())
}

#[cfg(not(windows))]
pub fn zero_fill(
    _disk: &str,
    _iptal: Arc<AtomicBool>,
    _progress: impl Fn(f32, String) + Send,
) -> Result<()> {
    Err(anyhow!("Zero-fill sadece Windows'ta destekleniyor bu surumde."))
}

pub fn secure_erase(disk: &str) -> Result<String> {
    let mut cikti = String::new();
    cikti.push_str("=== SECURE ERASE ===\n\n");

    let out = cmd(crate::embedded::smartctl_yol()).args(["-g", "security", disk]).output()?;
    let sec = String::from_utf8_lossy(&out.stdout).to_string();
    cikti.push_str(&sec);
    cikti.push('\n');

    if sec.contains("frozen") && !sec.contains("not frozen") {
        cikti.push_str("\nDisk is FROZEN.\n");
        cikti.push_str("Fix: put PC to sleep and wake it, then try again.\n");
        return Ok(cikti);
    }

    cikti.push_str("\nSecure Erase cannot be done directly via smartctl on Windows.\n");
    cikti.push_str("Recommended tools:\n");
    cikti.push_str("  1. WD Data Lifeguard Diagnostic\n");
    cikti.push_str("     https://support.wdc.com/downloads.aspx\n");
    cikti.push_str("  2. Parted Magic (Linux Live USB)\n");
    cikti.push_str("  3. hdparm (Linux):\n");
    cikti.push_str("     hdparm --user-master u --security-set-pass p ");
    cikti.push_str(disk);
    cikti.push('\n');
    cikti.push_str("     hdparm --user-master u --security-erase p ");
    cikti.push_str(disk);
    cikti.push('\n');

    Ok(cikti)
}

#[cfg(windows)]
fn disk_yolu_windows(disk: &str) -> Result<String> {
    if let Some(num) = disk.strip_prefix("/dev/pd") {
        Ok(format!("\\\\.\\PhysicalDrive{}", num))
    } else if disk.starts_with("\\\\.\\") {
        Ok(disk.to_string())
    } else {
        Err(anyhow!("Bilinmeyen disk yolu: {}", disk))
    }
}

#[cfg(windows)]
fn disk_boyutu_windows(yol: &str) -> Result<u64> {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{CloseHandle, GENERIC_READ, HANDLE};
    use windows::Win32::Storage::FileSystem::{
        CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    };
    use windows::Win32::System::Ioctl::{GET_LENGTH_INFORMATION, IOCTL_DISK_GET_LENGTH_INFO};
    use windows::Win32::System::IO::DeviceIoControl;

    let wide: Vec<u16> = yol.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let handle: HANDLE = CreateFileW(
            PCWSTR(wide.as_ptr()),
            GENERIC_READ.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        )
        .context("Cannot open disk (admin required)")?;

        let mut info = GET_LENGTH_INFORMATION::default();
        let mut ret: u32 = 0;

        DeviceIoControl(
            handle,
            IOCTL_DISK_GET_LENGTH_INFO,
            None,
            0,
            Some(&mut info as *mut _ as *mut _),
            std::mem::size_of::<GET_LENGTH_INFORMATION>() as u32,
            Some(&mut ret),
            None,
        )
        .context("Cannot read disk size")?;

        let _ = CloseHandle(handle);
        Ok(info.Length as u64)
    }
}
