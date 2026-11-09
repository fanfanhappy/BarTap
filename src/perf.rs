//! 系统性能采样：CPU、内存、网络吞吐与系统盘空间。
//!
//! 采样在独立线程中每秒执行一次，界面只读最新快照与 60 秒历史。
//! 性能页不可见时暂停，避免后台无意义占用。

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::time::{Duration, Instant};

const HISTORY: usize = 60;
const GB: f64 = 1024.0 * 1024.0 * 1024.0;

#[derive(Clone, Debug, Default)]
pub struct Sample {
    pub cpu: Option<f32>,
    pub mem_used: u64,
    pub mem_total: u64,
    pub net_down: Option<f64>,
    pub net_up: Option<f64>,
    pub storage_free: u64,
    pub storage_total: u64,
    pub cores: usize,
}

pub struct Monitor {
    rx: Option<Receiver<Sample>>,
    active: Arc<AtomicBool>,
    latest: Sample,
    cpu: VecDeque<f32>,
    mem: VecDeque<f32>,
}

impl Default for Monitor {
    fn default() -> Self {
        Self {
            rx: None,
            active: Arc::new(AtomicBool::new(false)),
            latest: Sample::default(),
            cpu: VecDeque::with_capacity(HISTORY),
            mem: VecDeque::with_capacity(HISTORY),
        }
    }
}

impl Monitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ensure_started(&mut self) {
        if self.rx.is_some() {
            return;
        }
        self.active.store(true, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel();
        let active = self.active.clone();
        std::thread::spawn(move || {
            let mut sampler = Sampler::default();
            let mut was_active = false;
            let mut first = true;
            loop {
                if !active.load(Ordering::Relaxed) {
                    was_active = false;
                    std::thread::sleep(Duration::from_millis(300));
                    continue;
                }
                if !was_active {
                    sampler = Sampler::default();
                    was_active = true;
                    first = true;
                }
                if tx.send(sampler.sample()).is_err() {
                    break;
                }
                std::thread::sleep(if first { Duration::from_millis(250) } else { Duration::from_secs(1) });
                first = false;
            }
        });
        self.rx = Some(rx);
    }

    pub fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::Relaxed);
    }

    pub fn poll(&mut self) {
        let Some(rx) = &self.rx else { return; };
        while let Ok(sample) = rx.try_recv() {
            if let Some(cpu) = sample.cpu {
                push(&mut self.cpu, cpu);
            }
            let mem = memory_percent(sample.mem_used, sample.mem_total);
            if mem > 0.0 {
                push(&mut self.mem, mem);
            }
            self.latest = sample;
        }
    }

    pub fn latest(&self) -> &Sample {
        &self.latest
    }

    pub fn cpu_history(&self) -> &VecDeque<f32> {
        &self.cpu
    }

    pub fn mem_history(&self) -> &VecDeque<f32> {
        &self.mem
    }
}

fn push(history: &mut VecDeque<f32>, value: f32) {
    if history.len() == HISTORY {
        history.pop_front();
    }
    history.push_back(value.clamp(0.0, 100.0));
}

pub fn memory_percent(used: u64, total: u64) -> f32 {
    if total == 0 {
        0.0
    } else {
        (used as f64 * 100.0 / total as f64).clamp(0.0, 100.0) as f32
    }
}

pub fn storage_percent(free: u64, total: u64) -> f32 {
    if total == 0 {
        0.0
    } else {
        (100.0 - free as f64 * 100.0 / total as f64).clamp(0.0, 100.0) as f32
    }
}

pub fn gb(bytes: u64) -> f64 {
    bytes as f64 / GB
}

pub fn rate_text(bytes_per_sec: f64) -> String {
    if !bytes_per_sec.is_finite() || bytes_per_sec <= 0.0 {
        return "0 B/s".into();
    }
    if bytes_per_sec >= 1024.0 * 1024.0 {
        format!("{:.1} MB/s", bytes_per_sec / (1024.0 * 1024.0))
    } else if bytes_per_sec >= 1024.0 {
        format!("{:.0} KB/s", bytes_per_sec / 1024.0)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}

fn cpu_usage(prev: (u64, u64, u64), now: (u64, u64, u64)) -> Option<f32> {
    let idle = now.0.saturating_sub(prev.0);
    let busy = now.1.saturating_sub(prev.1).saturating_add(now.2.saturating_sub(prev.2));
    if busy == 0 {
        return None;
    }
    Some((100.0 * (1.0 - idle as f64 / busy as f64)).clamp(0.0, 100.0) as f32)
}

fn net_rate(prev: u64, now: u64, seconds: f64) -> f64 {
    if seconds <= 0.0 {
        return 0.0;
    }
    now.saturating_sub(prev) as f64 / seconds
}

#[derive(Default)]
struct Sampler {
    prev_cpu: Option<(u64, u64, u64)>,
    prev_net: Option<(u64, u64, Instant)>,
}

impl Sampler {
    fn sample(&mut self) -> Sample {
        let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
        let (mem_used, mem_total) = memory();
        let (net_down, net_up) = self.network();
        let (storage_free, storage_total) = storage();
        Sample {
            cpu: self.cpu(),
            mem_used,
            mem_total,
            net_down,
            net_up,
            storage_free,
            storage_total,
            cores,
        }
    }

    #[cfg(windows)]
    fn cpu(&mut self) -> Option<f32> {
        use windows::Win32::Foundation::FILETIME;
        use windows::Win32::System::Threading::GetSystemTimes;
        let mut idle = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();
        unsafe { GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user)).ok()? };
        let now = (filetime(idle), filetime(kernel), filetime(user));
        let value = self.prev_cpu.and_then(|prev| cpu_usage(prev, now));
        self.prev_cpu = Some(now);
        value
    }

    #[cfg(not(windows))]
    fn cpu(&mut self) -> Option<f32> {
        None
    }

    #[cfg(windows)]
    fn network(&mut self) -> (Option<f64>, Option<f64>) {
        use windows::Win32::NetworkManagement::IpHelper::{
            FreeMibTable, GetIfTable2, MIB_IF_ROW2, IF_TYPE_ETHERNET_CSMACD, IF_TYPE_IEEE80211,
            IF_TYPE_WWANPP, IF_TYPE_WWANPP2,
        };
        use windows::Win32::NetworkManagement::Ndis::IfOperStatusUp;
        let mut table = std::ptr::null_mut();
        unsafe {
            if GetIfTable2(&mut table).is_err() || table.is_null() {
                return (None, None);
            }
            let count = (*table).NumEntries as usize;
            let base = (*table).Table.as_ptr() as *const MIB_IF_ROW2;
            let mut down = 0u64;
            let mut up = 0u64;
            for i in 0..count {
                let row = &*base.add(i);
                let hardware = matches!(
                    row.Type,
                    IF_TYPE_ETHERNET_CSMACD | IF_TYPE_IEEE80211 | IF_TYPE_WWANPP | IF_TYPE_WWANPP2
                );
                if hardware && row.OperStatus == IfOperStatusUp {
                    down = down.saturating_add(row.InOctets);
                    up = up.saturating_add(row.OutOctets);
                }
            }
            FreeMibTable(table as *const _);
            let now = Instant::now();
            let rates = self.prev_net.map(|(prev_down, prev_up, at)| {
                let seconds = now.duration_since(at).as_secs_f64();
                (net_rate(prev_down, down, seconds), net_rate(prev_up, up, seconds))
            });
            self.prev_net = Some((down, up, now));
            rates.map_or((None, None), |(down, up)| (Some(down), Some(up)))
        }
    }

    #[cfg(not(windows))]
    fn network(&mut self) -> (Option<f64>, Option<f64>) {
        (None, None)
    }
}

#[cfg(windows)]
fn filetime(value: windows::Win32::Foundation::FILETIME) -> u64 {
    ((value.dwHighDateTime as u64) << 32) | value.dwLowDateTime as u64
}

#[cfg(windows)]
fn memory() -> (u64, u64) {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    if unsafe { GlobalMemoryStatusEx(&mut status) }.is_err() {
        return (0, 0);
    }
    (
        status.ullTotalPhys.saturating_sub(status.ullAvailPhys),
        status.ullTotalPhys,
    )
}

#[cfg(not(windows))]
fn memory() -> (u64, u64) {
    (0, 0)
}

#[cfg(windows)]
fn storage() -> (u64, u64) {
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    let drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
    let mut wide: Vec<u16> = format!("{drive}\\").encode_utf16().collect();
    wide.push(0);
    let mut free = 0u64;
    let mut total = 0u64;
    unsafe {
        if GetDiskFreeSpaceExW(PCWSTR(wide.as_ptr()), Some(&mut free), Some(&mut total), None).is_err() {
            return (0, 0);
        }
    }
    (free, total)
}

#[cfg(not(windows))]
fn storage() -> (u64, u64) {
    (0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_usage_rejects_empty_delta_and_clamps() {
        assert_eq!(cpu_usage((10, 10, 10), (10, 10, 10)), None);
        assert_eq!(cpu_usage((10, 10, 10), (20, 30, 40)), Some(80.0));
        assert_eq!(cpu_usage((10, 10, 10), (10, 30, 40)), Some(100.0));
    }

    #[test]
    fn network_rate_handles_time_and_wrap() {
        assert_eq!(net_rate(100, 300, 2.0), 100.0);
        assert_eq!(net_rate(300, 100, 2.0), 0.0);
        assert_eq!(net_rate(100, 300, 0.0), 0.0);
    }

    #[test]
    fn formatting_memory_storage_and_rates() {
        assert_eq!(gb(1024 * 1024 * 1024), 1.0);
        assert_eq!(memory_percent(1, 4), 25.0);
        assert_eq!(storage_percent(25, 100), 75.0);
        assert_eq!(rate_text(0.0), "0 B/s");
        assert_eq!(rate_text(900.0), "900 B/s");
        assert_eq!(rate_text(2048.0), "2 KB/s");
        assert_eq!(rate_text(2.0 * 1024.0 * 1024.0), "2.0 MB/s");
    }
}
