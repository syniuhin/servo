/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use constellation_msg::{HangAlert, HangAnnotation};
use constellation_msg::{MonitoredComponentId, MonitoredComponentMsg};
use ipc_channel::ipc::IpcSender;
use servo_channel::{Receiver, base_channel};
use std::collections::HashMap;
use std::time::{Duration, Instant};


struct MonitoredComponent {
    last_activity: Instant,
    last_annotation: Option<HangAnnotation>,
    transient_hang_timeout: Duration,
    permanent_hang_timeout: Duration,
    sent_transient_alert: bool,
    sent_permanent_alert: bool,
    is_waiting: bool,
}

pub struct BackgroundHangMonitor {
    monitored_components: HashMap<MonitoredComponentId, MonitoredComponent>,
    constellation_chan: IpcSender<HangAlert>,
    port: Receiver<(MonitoredComponentId, MonitoredComponentMsg)>,
}

impl BackgroundHangMonitor {
    pub fn new(
        port: Receiver<(MonitoredComponentId, MonitoredComponentMsg)>,
        constellation_chan: IpcSender<HangAlert>,
        component_id: MonitoredComponentId,
        transient_hang_timeout: Duration,
        permanent_hang_timeout: Duration,
    ) -> Self {
        let mut monitor = BackgroundHangMonitor {
            monitored_components: Default::default(),
            constellation_chan,
            port,
        };
        let component = MonitoredComponent {
            last_activity: Instant::now(),
            last_annotation: None,
            transient_hang_timeout,
            permanent_hang_timeout,
            sent_transient_alert: false,
            sent_permanent_alert: false,
            is_waiting: true,
        };
        assert!(
            monitor
                .monitored_components
                .insert(component_id, component)
                .is_none(),
            "This component was already registered for monitoring."
        );
        monitor
    }

    pub fn run(&mut self) -> bool {
        let received = select! {
            recv(self.port.select(), event) => {
                match event {
                    Some(msg) => Some(msg),
                    None => return false,
                }
            },
            recv(base_channel::after(Duration::from_millis(100))) => None,
        };
        if let Some(msg) = received {
            self.handle_msg(msg);
        }
        self.perform_a_hang_monitor_checkpoint();
        true
    }

    fn handle_msg(&mut self, msg: (MonitoredComponentId, MonitoredComponentMsg)) {
        match msg {
            (component_id, MonitoredComponentMsg::NotifyActivity(annotation)) => {
                let mut component = self
                    .monitored_components
                    .get_mut(&component_id)
                    .expect("Receiced NotifyActivity for an unknown component");
                component.last_activity = Instant::now();
                component.last_annotation = Some(annotation);
                component.is_waiting = false;
            },
            (component_id, MonitoredComponentMsg::NotifyWait) => {
                let mut component = self
                    .monitored_components
                    .get_mut(&component_id)
                    .expect("Receiced NotifyWait for an unknown component");
                component.last_activity = Instant::now();
                component.sent_transient_alert = false;
                component.sent_permanent_alert = false;
                component.is_waiting = true;
            },
        }
    }

    fn perform_a_hang_monitor_checkpoint(&mut self) {
        for (component_id, mut monitored) in self.monitored_components.iter_mut() {
            if monitored.is_waiting {
                continue;
            }
            let last_annotation = monitored.last_annotation.unwrap();
            if monitored.last_activity.elapsed() > monitored.permanent_hang_timeout {
                match monitored.sent_permanent_alert {
                    true => continue,
                    false => {
                        let _ = self
                            .constellation_chan
                            .send(HangAlert::Permanent(component_id.clone(), last_annotation));
                    },
                }
                monitored.sent_permanent_alert = true;
                continue;
            }
            if monitored.last_activity.elapsed() > monitored.transient_hang_timeout {
                match monitored.sent_transient_alert {
                    true => continue,
                    false => {
                        let _ = self
                            .constellation_chan
                            .send(HangAlert::Transient(component_id.clone(), last_annotation));
                    },
                }
                monitored.sent_transient_alert = true;
                continue;
            }
        }
    }
}
