use serde::{Deserialize, Serialize};
use sysinfo::Networks;

/// Statistics collected about the network interfaces from the host machine.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// count of network interfaces
    pub interfaces: usize,
    /// Sum of received data in bytes
    pub total_received: u64,
    /// Sum of transmitted data in bytes
    pub total_transmitted: u64,
    /// Sum of received packets
    pub total_packets_received: u64,
    /// Sum of transmitted packets
    pub total_packets_transmitted: u64,
    /// Sum of errors on receiving data
    pub total_errors_on_received: u64,
    /// Sum of errors on transmitting data
    pub total_errors_on_transmitted: u64,
}

impl From<Networks> for NetworkInfo {
    fn from(networks: Networks) -> Self {
        let total_received = networks.iter().map(|n| n.1.total_received()).sum::<u64>();
        let total_transmitted = networks
            .iter()
            .map(|n| n.1.total_transmitted())
            .sum::<u64>();
        let total_packets_received = networks
            .iter()
            .map(|n| n.1.total_packets_received())
            .sum::<u64>();
        let total_packets_transmitted = networks
            .iter()
            .map(|n| n.1.total_packets_transmitted())
            .sum::<u64>();
        let total_errors_on_received = networks
            .iter()
            .map(|n| n.1.total_errors_on_received())
            .sum::<u64>();
        let total_errors_on_transmitted = networks
            .iter()
            .map(|n| n.1.total_errors_on_transmitted())
            .sum::<u64>();

        NetworkInfo {
            interfaces: networks.len(),
            total_received,
            total_transmitted,
            total_packets_received,
            total_packets_transmitted,
            total_errors_on_received,
            total_errors_on_transmitted,
        }
    }
}
