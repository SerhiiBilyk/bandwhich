use ::std::collections::{BTreeMap, HashMap};
use ::std::net::Ipv4Addr;

use crate::network::{Connection, Utilization};

static BANDWIDTH_DECAY_FACTOR: f32 = 0.5;

pub trait Bandwidth {
    fn get_total_bytes_downloaded(&self) -> u128;
    fn get_total_bytes_uploaded(&self) -> u128;

    fn get_avg_bytes_downloaded(&self) -> u128;
    fn get_avg_bytes_uploaded(&self) -> u128;
}

#[derive(Default)]
pub struct NetworkData {
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
    pub prev_total_bytes_downloaded: u128,
    pub prev_total_bytes_uploaded: u128,
    pub connection_count: u128,
}

#[derive(Default)]
pub struct ConnectionData {
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
    pub prev_total_bytes_downloaded: u128,
    pub prev_total_bytes_uploaded: u128,
    pub process_name: String,
    pub interface_name: String,
}

fn calc_avg_bandwidth(prev_bandwidth: u128, curr_bandwidth: u128) -> u128 {
    if prev_bandwidth == 0 {
        curr_bandwidth
    } else {
        (prev_bandwidth as f32 * BANDWIDTH_DECAY_FACTOR
            + (1.0 - BANDWIDTH_DECAY_FACTOR) * curr_bandwidth as f32) as u128
    }
}

impl Bandwidth for ConnectionData {
    fn get_total_bytes_uploaded(&self) -> u128 {
        self.total_bytes_uploaded
    }
    fn get_total_bytes_downloaded(&self) -> u128 {
        self.total_bytes_downloaded
    }
    fn get_avg_bytes_uploaded(&self) -> u128 {
        calc_avg_bandwidth(self.prev_total_bytes_uploaded, self.total_bytes_uploaded)
    }
    fn get_avg_bytes_downloaded(&self) -> u128 {
        calc_avg_bandwidth(
            self.prev_total_bytes_downloaded,
            self.total_bytes_downloaded,
        )
    }
}

impl Bandwidth for NetworkData {
    fn get_total_bytes_uploaded(&self) -> u128 {
        self.total_bytes_uploaded
    }
    fn get_total_bytes_downloaded(&self) -> u128 {
        self.total_bytes_downloaded
    }
    fn get_avg_bytes_uploaded(&self) -> u128 {
        calc_avg_bandwidth(self.prev_total_bytes_uploaded, self.total_bytes_uploaded)
    }
    fn get_avg_bytes_downloaded(&self) -> u128 {
        calc_avg_bandwidth(
            self.prev_total_bytes_downloaded,
            self.total_bytes_downloaded,
        )
    }
}

#[derive(Default)]
pub struct UIState {
    pub processes: BTreeMap<String, NetworkData>,
    pub remote_addresses: BTreeMap<Ipv4Addr, NetworkData>,
    pub connections: BTreeMap<Connection, ConnectionData>,
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
}

impl UIState {
    pub fn new(
        connections_to_procs: HashMap<Connection, String>,
        mut network_utilization: Utilization,
        old_state: &UIState,
    ) -> Self {
        let mut processes: BTreeMap<String, NetworkData> = BTreeMap::new();
        let mut remote_addresses: BTreeMap<Ipv4Addr, NetworkData> = BTreeMap::new();
        let mut connections: BTreeMap<Connection, ConnectionData> = BTreeMap::new();
        let mut total_bytes_downloaded: u128 = 0;
        let mut total_bytes_uploaded: u128 = 0;
        for (connection, process_name) in connections_to_procs {
            if let Some(connection_info) = network_utilization.connections.remove(&connection) {
                let data_for_remote_address = remote_addresses
                    .entry(connection.remote_socket.ip)
                    .or_default();
                let connection_data = connections.entry(connection).or_default();
                let data_for_process = processes.entry(process_name.clone()).or_default();

                data_for_process.total_bytes_downloaded += connection_info.total_bytes_downloaded;
                data_for_process.total_bytes_uploaded += connection_info.total_bytes_uploaded;
                data_for_process.connection_count += 1;
                connection_data.total_bytes_downloaded += connection_info.total_bytes_downloaded;
                connection_data.total_bytes_uploaded += connection_info.total_bytes_uploaded;
                connection_data.process_name = process_name;
                connection_data.interface_name = connection_info.interface_name;
                data_for_remote_address.total_bytes_downloaded +=
                    connection_info.total_bytes_downloaded;
                data_for_remote_address.total_bytes_uploaded +=
                    connection_info.total_bytes_uploaded;
                data_for_remote_address.connection_count += 1;
                total_bytes_downloaded += connection_info.total_bytes_downloaded;
                total_bytes_uploaded += connection_info.total_bytes_uploaded;

                // Record bandwidth data of last iteration
                if let Some(prev_connection_info) = old_state.connections.get(&connection) {
                    // Using previous round's weighted average. Exponential decay
                    let prev_bytes_downloaded = prev_connection_info.get_avg_bytes_downloaded();
                    let prev_bytes_uploaded = prev_connection_info.get_avg_bytes_uploaded();

                    connection_data.prev_total_bytes_downloaded += prev_bytes_downloaded;
                    connection_data.prev_total_bytes_uploaded += prev_bytes_uploaded;

                    data_for_process.prev_total_bytes_downloaded += prev_bytes_downloaded;
                    data_for_process.prev_total_bytes_uploaded += prev_bytes_uploaded;

                    data_for_remote_address.prev_total_bytes_downloaded += prev_bytes_downloaded;
                    data_for_remote_address.prev_total_bytes_uploaded += prev_bytes_uploaded;
                }
            }
        }
        UIState {
            processes,
            remote_addresses,
            connections,
            total_bytes_downloaded,
            total_bytes_uploaded,
        }
    }
}
