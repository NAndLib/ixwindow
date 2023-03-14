use bspc_rs::events::{
    DesktopEvent, DesktopFocusInfo, Event, NodeAddInfo, NodeEvent,
    NodeFocusInfo, NodeRemoveInfo, NodeTransferInfo, Subscription,
};

use bspc_rs::properties::State;
use bspc_rs::BspwmConnection;
use std::thread;
use std::time::Duration;

use crate::config::BspwmConfig;
use crate::core::{ConfigFeatures as _, Core};

pub fn exec(monitor_name: Option<String>) {
    let mut conn = BspwmConnection::new();
    let mut core = Core::init(monitor_name);
    core.process_start();

    let subscriptions = [
        Subscription::NodeAdd,
        Subscription::NodeFocus,
        Subscription::NodeRemove,
        Subscription::NodeFlag,
        Subscription::NodeState,
        // Subscription::DesktopFocus,
    ];

    let subscriber = conn
        .subscribe(&subscriptions, false, None)
        .expect("Couldn't subscribe to events");

    for raw_event in subscriber {
        println!("Fooo");
        // println!("{raw_event:#?}");

        match raw_event {
            Ok(event) => {
                core.handle_event(event);
            }

            Err(e) => {
                println!("While listening to events, encounter the following error: {e}");
            }
        }
    }
}

impl Core<BspwmConnection, BspwmConfig> {
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::NodeEvent(e) => self.handle_node_event(e),
            Event::DesktopEvent(e) => self.handle_desktop_event(e),
            _ => {
                unreachable!();
            }
        }
    }

    fn handle_node_event(&mut self, event: NodeEvent) {
        match event {
            NodeEvent::NodeAdd(node_info) => {
                thread::sleep(Duration::from_millis(100));
            }
            NodeEvent::NodeFocus(node_info) => {
                self.process_focused_window(
                    node_info.node_id.try_into().unwrap(),
                );
            }

            NodeEvent::NodeRemove(node_info) => {
                if self.is_curr_desk_empty() {
                    self.process_empty_desktop();
                }
            }

            NodeEvent::NodeFlag(node_info) => {}

            NodeEvent::NodeState(node_info) => {
                if let State::Fullscreen = node_info.state {
                    // We can use unwrap, because some desktop should be focused
                    let current_desktop =
                        self.get_focused_desktop_id().unwrap();

                    match self.get_fullscreen_window_id(current_desktop) {
                        Some(_) => {
                            self.process_fullscreen_window();
                        }

                        None => {
                            let window = self.get_focused_window_id();
                            if let Some(id) = window {
                                self.process_focused_window(id);
                            }
                        }
                    }
                }
            }
            _ => {
                unreachable!();
            }
        }
    }

    fn handle_desktop_event(&mut self, event: DesktopEvent) {
        match event {
            DesktopEvent::DesktopFocus(event_info) => {
                let current_desktop = match self.get_focused_desktop_id() {
                    Some(x) => x,

                    // No desktop is focused on the monitor
                    None => {
                        return;
                    }
                };

                if self.is_desk_empty(current_desktop) {
                    self.process_empty_desktop();
                }

                if self.get_fullscreen_window_id(current_desktop).is_some() {
                    self.process_fullscreen_window();
                }
            }

            _ => {
                unreachable!();
            }
        }
    }
}
