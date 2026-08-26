use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use openlcd_runtime::RuntimeData;

use crate::SystemCollector;

const DEFAULT_REFRESH_INTERVAL: Duration = Duration::from_millis(500);

pub struct MonitorService {
    receiver: Receiver<RuntimeData>,
}

impl MonitorService {
    pub fn start() -> Self {
        Self::start_with_interval(DEFAULT_REFRESH_INTERVAL)
    }

    pub fn start_with_interval(interval: Duration) -> Self {
        let (sender, receiver) = mpsc::channel();

        thread::Builder::new()
            .name("openlcd-monitor".to_owned())
            .spawn(move || {
                run_monitor_loop(sender, interval);
            })
            .expect("failed to start monitor thread");

        Self { receiver }
    }

    pub fn latest(&self) -> Option<RuntimeData> {
        let mut latest = None;

        while let Ok(data) = self.receiver.try_recv() {
            latest = Some(data);
        }

        latest
    }
}

fn run_monitor_loop(sender: Sender<RuntimeData>, interval: Duration) {
    let mut collector = SystemCollector::new();

    //
    // CPU usage needs to be measured beforehand.
    // This first collection initializes the baseline.
    //
    let _ = collector.collect();

    thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);

    loop {
        let data = collector.collect();

        if sender.send(data).is_err() {
            // GUI foi encerrada.
            break;
        }

        thread::sleep(interval);
    }
}
