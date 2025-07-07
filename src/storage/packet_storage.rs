use crate::analyzers::{AnalysisResult, SecurityFlag, TrafficType};
use crate::models::{NetworkPacket, PacketStatistics};
use crate::storage::schema::{create_tables, setup_data_retention};
use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use log::{debug, info, warn};
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};

pub struct PacketStorage {
    conn: Arc<Mutex<Connection>>,
    batch_size: usize,
    pending_stats: Arc<Mutex<Vec<PacketStatistics>>>,
    pending_protocols: Arc<Mutex<Vec<ProtocolRecord>>>,
    pending_connections: Arc<Mutex<Vec<ConnectionRecord>>>,
    pending_security_events: Arc<Mutex<Vec<SecurityEvent>>>,
}

#[derive(Debug, Clone)]
pub struct ProtocolRecord {
    pub timestamp: DateTime<Local>,
    pub interface_name: String,
    pub protocol_name: String,
    pub packet_count: u64,
    pub byte_count: u64,
    pub is_encrypted: bool,
}

#[derive(Debug, Clone)]
pub struct ConnectionRecord {
    pub connection_key: String,
    pub source_ip: String,
    pub dest_ip: String,
    pub source_port: Option<u16>,
    pub dest_port: Option<u16>,
    pub protocol: String,
    pub application_protocol: Option<String>,
    pub first_seen: DateTime<Local>,
    pub last_seen: DateTime<Local>,
    pub packet_count: u64,
    pub byte_count: u64,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct SecurityEvent {
    pub timestamp: DateTime<Local>,
    pub interface_name: String,
    pub event_type: String,
    pub source_ip: Option<String>,
    pub dest_ip: Option<String>,
    pub port: Option<u16>,
    pub protocol: Option<String>,
    pub description: String,
    pub severity: String,
}

#[derive(Debug, Clone)]
pub struct TrafficSummary {
    pub timestamp: DateTime<Local>,
    pub interface_name: String,
    pub total_packets: u64,
    pub total_bytes: u64,
    pub protocols: HashMap<String, ProtocolStats>,
    pub traffic_types: HashMap<TrafficType, TrafficTypeStats>,
    pub top_connections: Vec<ConnectionSummary>,
}

#[derive(Debug, Clone)]
pub struct ProtocolStats {
    pub packets: u64,
    pub bytes: u64,
    pub connections: u64,
}

#[derive(Debug, Clone)]
pub struct TrafficTypeStats {
    pub packets: u64,
    pub bytes: u64,
    pub connections: u64,
}

#[derive(Debug, Clone)]
pub struct ConnectionSummary {
    pub source: String,
    pub destination: String,
    pub protocol: String,
    pub packets: u64,
    pub bytes: u64,
}

impl PacketStorage {
    pub fn new<P: AsRef<Path>>(db_path: P, batch_size: usize) -> Result<Self> {
        // Ensure the parent directory exists
        if let Some(parent) = db_path.as_ref().parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create database directory")?;
        }
        
        let conn = Connection::open(db_path)
            .context("Failed to open database connection")?;
        
        // Enable WAL mode for better concurrent access (ignore errors for in-memory DBs)
        let _ = conn.pragma_update(None, "journal_mode", "WAL");
        
        // Set reasonable timeout
        conn.busy_timeout(Duration::from_secs(5))
            .context("Failed to set busy timeout")?;

        create_tables(&conn)
            .context("Failed to create database tables")?;

        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
            batch_size,
            pending_stats: Arc::new(Mutex::new(Vec::new())),
            pending_protocols: Arc::new(Mutex::new(Vec::new())),
            pending_connections: Arc::new(Mutex::new(Vec::new())),
            pending_security_events: Arc::new(Mutex::new(Vec::new())),
        };

        // Start background flush task
        storage.start_background_flush();

        info!("Packet storage initialized with batch size: {batch_size}");
        Ok(storage)
    }

    pub fn store_packet_stats(&self, stats: PacketStatistics) -> Result<()> {
        let mut pending = self.pending_stats.lock().unwrap();
        pending.push(stats);
        
        if pending.len() >= self.batch_size {
            self.flush_packet_stats()?;
        }
        
        Ok(())
    }

    pub fn store_protocol_info(&self, record: ProtocolRecord) -> Result<()> {
        let mut pending = self.pending_protocols.lock().unwrap();
        pending.push(record);
        
        if pending.len() >= self.batch_size {
            self.flush_protocol_records()?;
        }
        
        Ok(())
    }

    pub fn store_connection(&self, record: ConnectionRecord) -> Result<()> {
        let mut pending = self.pending_connections.lock().unwrap();
        pending.push(record);
        
        if pending.len() >= self.batch_size {
            self.flush_connection_records()?;
        }
        
        Ok(())
    }

    pub fn store_security_event(&self, event: SecurityEvent) -> Result<()> {
        let mut pending = self.pending_security_events.lock().unwrap();
        pending.push(event);
        
        if pending.len() >= self.batch_size {
            self.flush_security_events()?;
        }
        
        Ok(())
    }

    pub fn analyze_packet_for_storage(
        &self,
        packet: &NetworkPacket,
        analysis: &AnalysisResult,
    ) -> Result<()> {
        // Store protocol information
        if let Some(protocol_name) = &analysis.application_protocol {
            let protocol_record = ProtocolRecord {
                timestamp: packet.timestamp,
                interface_name: packet.interface.clone(),
                protocol_name: protocol_name.clone(),
                packet_count: 1,
                byte_count: packet.size_bytes,
                is_encrypted: analysis.is_encrypted,
            };
            self.store_protocol_info(protocol_record)?;
        }

        // Store connection information
        if let (Some(src), Some(dst)) = (packet.source_addr, packet.dest_addr) {
            let connection_key = format!(
                "{}:{}-{}:{}",
                src,
                packet.source_port.unwrap_or(0),
                dst,
                packet.dest_port.unwrap_or(0)
            );
            
            let connection_record = ConnectionRecord {
                connection_key,
                source_ip: src.to_string(),
                dest_ip: dst.to_string(),
                source_port: packet.source_port,
                dest_port: packet.dest_port,
                protocol: format!("{:?}", packet.transport_protocol),
                application_protocol: analysis.application_protocol.clone(),
                first_seen: packet.timestamp,
                last_seen: packet.timestamp,
                packet_count: 1,
                byte_count: packet.size_bytes,
                is_active: true,
            };
            self.store_connection(connection_record)?;
        }

        // Store security events
        for flag in &analysis.security_flags {
            let event = SecurityEvent {
                timestamp: packet.timestamp,
                interface_name: packet.interface.clone(),
                event_type: format!("{flag:?}"),
                source_ip: packet.source_addr.map(|ip| ip.to_string()),
                dest_ip: packet.dest_addr.map(|ip| ip.to_string()),
                port: packet.dest_port.or(packet.source_port),
                protocol: Some(format!("{:?}", packet.transport_protocol)),
                description: self.security_flag_description(flag),
                severity: self.security_flag_severity(flag),
            };
            self.store_security_event(event)?;
        }

        Ok(())
    }

    pub fn get_traffic_summary(
        &self,
        interface: &str,
        since: DateTime<Local>,
    ) -> Result<TrafficSummary> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT 
                COALESCE(SUM(total_packets), 0) as total_packets,
                COALESCE(SUM(total_bytes), 0) as total_bytes
             FROM packet_stats 
             WHERE interface_name = ?1 AND timestamp >= ?2"
        )?;
        
        let (total_packets, total_bytes): (u64, u64) = stmt.query_row(
            params![interface, since.format("%Y-%m-%d %H:%M:%S").to_string()],
            |row| Ok((row.get(0)?, row.get(1)?))
        ).unwrap_or((0, 0));

        // Get protocol breakdown
        let mut protocols = HashMap::new();
        let mut protocol_stmt = conn.prepare(
            "SELECT protocol_name, SUM(packet_count), SUM(byte_count), COUNT(DISTINCT id)
             FROM protocol_distribution 
             WHERE interface_name = ?1 AND timestamp >= ?2
             GROUP BY protocol_name"
        )?;
        
        let protocol_rows = protocol_stmt.query_map(
            params![interface, since.format("%Y-%m-%d %H:%M:%S").to_string()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    ProtocolStats {
                        packets: row.get(1)?,
                        bytes: row.get(2)?,
                        connections: row.get(3)?,
                    }
                ))
            }
        )?;

        for protocol_row in protocol_rows {
            let (protocol_name, stats) = protocol_row?;
            protocols.insert(protocol_name, stats);
        }

        // Get top connections
        let mut connection_stmt = conn.prepare(
            "SELECT source_ip, dest_ip, protocol, packet_count, byte_count
             FROM connections 
             WHERE last_seen >= ?1
             ORDER BY byte_count DESC 
             LIMIT 10"
        )?;
        
        let connection_rows = connection_stmt.query_map(
            params![since.format("%Y-%m-%d %H:%M:%S").to_string()],
            |row| {
                Ok(ConnectionSummary {
                    source: row.get(0)?,
                    destination: row.get(1)?,
                    protocol: row.get(2)?,
                    packets: row.get(3)?,
                    bytes: row.get(4)?,
                })
            }
        )?;

        let top_connections: Result<Vec<_>, _> = connection_rows.collect();

        Ok(TrafficSummary {
            timestamp: Local::now(),
            interface_name: interface.to_string(),
            total_packets,
            total_bytes,
            protocols,
            traffic_types: HashMap::new(), // TODO: Implement traffic type aggregation
            top_connections: top_connections?,
        })
    }

    pub fn cleanup_old_data(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;
        setup_data_retention(&tx)?;
        tx.commit()?;
        info!("Database cleanup completed");
        Ok(())
    }

    fn flush_packet_stats(&self) -> Result<()> {
        let stats_to_flush = {
            let mut pending = self.pending_stats.lock().unwrap();
            if pending.is_empty() {
                return Ok(());
            }
            std::mem::take(&mut *pending)
        };

        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO packet_stats (
                    timestamp, interface_name, total_packets, total_bytes,
                    packets_per_second, bytes_per_second,
                    tcp_packets, udp_packets, icmp_packets, other_packets,
                    tcp_bytes, udp_bytes, icmp_bytes, other_bytes
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)"
            )?;

            for stats in stats_to_flush {
                stmt.execute(params![
                    stats.start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                    "", // interface name would need to be added to PacketStatistics
                    stats.total_packets,
                    stats.total_bytes,
                    stats.packets_per_second,
                    stats.bytes_per_second,
                    stats.protocol_distribution.tcp_packets,
                    stats.protocol_distribution.udp_packets,
                    stats.protocol_distribution.icmp_packets,
                    stats.protocol_distribution.other_packets,
                    stats.protocol_distribution.tcp_bytes,
                    stats.protocol_distribution.udp_bytes,
                    stats.protocol_distribution.icmp_bytes,
                    stats.protocol_distribution.other_bytes,
                ])?;
            }
        }

        tx.commit()?;
        debug!("Flushed packet statistics to database");
        Ok(())
    }

    fn flush_protocol_records(&self) -> Result<()> {
        let records_to_flush = {
            let mut pending = self.pending_protocols.lock().unwrap();
            if pending.is_empty() {
                return Ok(());
            }
            std::mem::take(&mut *pending)
        };

        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO protocol_distribution (
                    timestamp, interface_name, protocol_name, 
                    packet_count, byte_count, is_encrypted
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
            )?;

            for record in records_to_flush {
                stmt.execute(params![
                    record.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                    record.interface_name,
                    record.protocol_name,
                    record.packet_count,
                    record.byte_count,
                    record.is_encrypted,
                ])?;
            }
        }

        tx.commit()?;
        debug!("Flushed protocol records to database");
        Ok(())
    }

    fn flush_connection_records(&self) -> Result<()> {
        let records_to_flush = {
            let mut pending = self.pending_connections.lock().unwrap();
            if pending.is_empty() {
                return Ok(());
            }
            std::mem::take(&mut *pending)
        };

        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;

        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO connections (
                    connection_key, source_ip, dest_ip, source_port, dest_port,
                    protocol, application_protocol, first_seen, last_seen,
                    packet_count, byte_count, is_active
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"
            )?;

            for record in records_to_flush {
                stmt.execute(params![
                    record.connection_key,
                    record.source_ip,
                    record.dest_ip,
                    record.source_port,
                    record.dest_port,
                    record.protocol,
                    record.application_protocol,
                    record.first_seen.format("%Y-%m-%d %H:%M:%S").to_string(),
                    record.last_seen.format("%Y-%m-%d %H:%M:%S").to_string(),
                    record.packet_count,
                    record.byte_count,
                    record.is_active,
                ])?;
            }
        }

        tx.commit()?;
        debug!("Flushed connection records to database");
        Ok(())
    }

    fn flush_security_events(&self) -> Result<()> {
        let events_to_flush = {
            let mut pending = self.pending_security_events.lock().unwrap();
            if pending.is_empty() {
                return Ok(());
            }
            std::mem::take(&mut *pending)
        };

        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO security_events (
                    timestamp, interface_name, event_type, source_ip, dest_ip,
                    port, protocol, description, severity
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
            )?;

            for event in events_to_flush {
                stmt.execute(params![
                    event.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                    event.interface_name,
                    event.event_type,
                    event.source_ip,
                    event.dest_ip,
                    event.port,
                    event.protocol,
                    event.description,
                    event.severity,
                ])?;
            }
        }

        tx.commit()?;
        debug!("Flushed security events to database");
        Ok(())
    }

    fn start_background_flush(&self) {
        let stats_clone = Arc::clone(&self.pending_stats);
        let protocols_clone = Arc::clone(&self.pending_protocols);
        let connections_clone = Arc::clone(&self.pending_connections);
        let security_events_clone = Arc::clone(&self.pending_security_events);
        let conn_clone = Arc::clone(&self.conn);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                
                // Force flush all pending data every 30 seconds
                if let Err(e) = Self::force_flush_all(
                    &stats_clone,
                    &protocols_clone,
                    &connections_clone,
                    &security_events_clone,
                    &conn_clone,
                ).await {
                    warn!("Background flush failed: {e}");
                }
            }
        });
    }

    async fn force_flush_all(
        _stats: &Arc<Mutex<Vec<PacketStatistics>>>,
        _protocols: &Arc<Mutex<Vec<ProtocolRecord>>>,
        _connections: &Arc<Mutex<Vec<ConnectionRecord>>>,
        _security_events: &Arc<Mutex<Vec<SecurityEvent>>>,
        _conn: &Arc<Mutex<Connection>>,
    ) -> Result<()> {
        // Implementation would flush all pending data
        // This is a simplified version for now
        debug!("Background flush executed");
        Ok(())
    }

    fn security_flag_description(&self, flag: &SecurityFlag) -> String {
        match flag {
            SecurityFlag::SuspiciousPort => "Traffic detected on suspicious port".to_string(),
            SecurityFlag::UnencryptedSensitive => "Sensitive data transmitted without encryption".to_string(),
            SecurityFlag::HighFrequency => "High frequency traffic pattern detected".to_string(),
            SecurityFlag::UnknownProtocol => "Unknown or unusual protocol detected".to_string(),
            SecurityFlag::LargePayload => "Unusually large payload detected".to_string(),
        }
    }

    fn security_flag_severity(&self, flag: &SecurityFlag) -> String {
        match flag {
            SecurityFlag::SuspiciousPort => "warning".to_string(),
            SecurityFlag::UnencryptedSensitive => "high".to_string(),
            SecurityFlag::HighFrequency => "info".to_string(),
            SecurityFlag::UnknownProtocol => "info".to_string(),
            SecurityFlag::LargePayload => "info".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{NetworkPacket, PacketDirection, PacketProtocol, TransportProtocol};
    use std::net::{IpAddr, Ipv4Addr};
    use tempfile::tempdir;

    #[test]
    fn test_packet_storage_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let storage = PacketStorage::new(db_path, 10);
        assert!(storage.is_ok());
    }

    #[test]
    fn test_store_protocol_info() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = PacketStorage::new(db_path, 1).unwrap();

        let record = ProtocolRecord {
            timestamp: Local::now(),
            interface_name: "eth0".to_string(),
            protocol_name: "HTTP".to_string(),
            packet_count: 1,
            byte_count: 1500,
            is_encrypted: false,
        };

        let result = storage.store_protocol_info(record);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_packet_for_storage() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = PacketStorage::new(db_path, 1).unwrap();

        let mut packet = NetworkPacket::new(
            "eth0".to_string(),
            1500,
            PacketProtocol::IPv4,
            PacketDirection::Outbound,
        );
        packet.source_addr = Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
        packet.dest_addr = Some(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)));
        packet.transport_protocol = TransportProtocol::TCP;
        packet.dest_port = Some(80);

        let analysis = AnalysisResult {
            application_protocol: Some("HTTP".to_string()),
            is_encrypted: false,
            traffic_type: crate::analyzers::TrafficType::Web,
            security_flags: vec![],
            flow_direction: crate::analyzers::FlowDirection::Outbound,
            geolocation: None,
        };

        let result = storage.analyze_packet_for_storage(&packet, &analysis);
        assert!(result.is_ok());
    }
}