use anyhow::Result;
use rusqlite::{Connection, Transaction};

pub fn create_tables(conn: &Connection) -> Result<()> {
    // Create packet statistics table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS packet_stats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME NOT NULL,
            interface_name TEXT NOT NULL,
            total_packets INTEGER NOT NULL DEFAULT 0,
            total_bytes INTEGER NOT NULL DEFAULT 0,
            packets_per_second REAL NOT NULL DEFAULT 0.0,
            bytes_per_second REAL NOT NULL DEFAULT 0.0,
            tcp_packets INTEGER NOT NULL DEFAULT 0,
            udp_packets INTEGER NOT NULL DEFAULT 0,
            icmp_packets INTEGER NOT NULL DEFAULT 0,
            other_packets INTEGER NOT NULL DEFAULT 0,
            tcp_bytes INTEGER NOT NULL DEFAULT 0,
            udp_bytes INTEGER NOT NULL DEFAULT 0,
            icmp_bytes INTEGER NOT NULL DEFAULT 0,
            other_bytes INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )?;

    // Create protocol distribution table for historical tracking
    conn.execute(
        "CREATE TABLE IF NOT EXISTS protocol_distribution (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME NOT NULL,
            interface_name TEXT NOT NULL,
            protocol_name TEXT NOT NULL,
            packet_count INTEGER NOT NULL DEFAULT 0,
            byte_count INTEGER NOT NULL DEFAULT 0,
            is_encrypted BOOLEAN NOT NULL DEFAULT FALSE
        )",
        [],
    )?;

    // Create connection tracking table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS connections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            connection_key TEXT UNIQUE NOT NULL,
            source_ip TEXT NOT NULL,
            dest_ip TEXT NOT NULL,
            source_port INTEGER,
            dest_port INTEGER,
            protocol TEXT NOT NULL,
            application_protocol TEXT,
            first_seen DATETIME NOT NULL,
            last_seen DATETIME NOT NULL,
            packet_count INTEGER NOT NULL DEFAULT 0,
            byte_count INTEGER NOT NULL DEFAULT 0,
            is_active BOOLEAN NOT NULL DEFAULT TRUE
        )",
        [],
    )?;

    // Create security events table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS security_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME NOT NULL,
            interface_name TEXT NOT NULL,
            event_type TEXT NOT NULL,
            source_ip TEXT,
            dest_ip TEXT,
            port INTEGER,
            protocol TEXT,
            description TEXT NOT NULL,
            severity TEXT NOT NULL DEFAULT 'info'
        )",
        [],
    )?;

    // Create traffic analysis table for aggregated data
    conn.execute(
        "CREATE TABLE IF NOT EXISTS traffic_analysis (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME NOT NULL,
            interface_name TEXT NOT NULL,
            period_minutes INTEGER NOT NULL DEFAULT 1,
            traffic_type TEXT NOT NULL,
            packet_count INTEGER NOT NULL DEFAULT 0,
            byte_count INTEGER NOT NULL DEFAULT 0,
            unique_connections INTEGER NOT NULL DEFAULT 0,
            top_protocol TEXT,
            avg_packet_size REAL NOT NULL DEFAULT 0.0
        )",
        [],
    )?;

    // Create indexes for better query performance
    create_indexes(conn)?;

    Ok(())
}

fn create_indexes(conn: &Connection) -> Result<()> {
    // Index on timestamp for time-based queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_packet_stats_timestamp 
         ON packet_stats(timestamp)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_protocol_distribution_timestamp 
         ON protocol_distribution(timestamp)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_connections_last_seen 
         ON connections(last_seen)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_security_events_timestamp 
         ON security_events(timestamp)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_traffic_analysis_timestamp 
         ON traffic_analysis(timestamp)",
        [],
    )?;

    // Index on interface for interface-specific queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_packet_stats_interface 
         ON packet_stats(interface_name)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_protocol_distribution_interface 
         ON protocol_distribution(interface_name)",
        [],
    )?;

    // Composite indexes for common query patterns
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_connections_active_last_seen 
         ON connections(is_active, last_seen)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_traffic_analysis_type_timestamp 
         ON traffic_analysis(traffic_type, timestamp)",
        [],
    )?;

    Ok(())
}

pub fn setup_data_retention(tx: &Transaction) -> Result<()> {
    // Set up automatic cleanup of old data
    // Keep detailed packet stats for 7 days
    tx.execute(
        "DELETE FROM packet_stats 
         WHERE timestamp < datetime('now', '-7 days')",
        [],
    )?;

    // Keep protocol distribution for 30 days
    tx.execute(
        "DELETE FROM protocol_distribution 
         WHERE timestamp < datetime('now', '-30 days')",
        [],
    )?;

    // Keep inactive connections for 1 day
    tx.execute(
        "DELETE FROM connections 
         WHERE is_active = FALSE AND last_seen < datetime('now', '-1 day')",
        [],
    )?;

    // Keep security events for 90 days
    tx.execute(
        "DELETE FROM security_events 
         WHERE timestamp < datetime('now', '-90 days')",
        [],
    )?;

    // Keep traffic analysis for 1 year
    tx.execute(
        "DELETE FROM traffic_analysis 
         WHERE timestamp < datetime('now', '-1 year')",
        [],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_create_tables() {
        let conn = Connection::open_in_memory().unwrap();
        let result = create_tables(&conn);
        assert!(result.is_ok());

        // Verify tables were created
        let table_count: i32 = conn
            .prepare("SELECT COUNT(*) FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_row([], |row| row.get(0))
            .unwrap();

        assert!(table_count >= 5); // We created 5 tables
    }

    #[test]
    fn test_indexes_created() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        let index_count: i32 = conn
            .prepare("SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'")
            .unwrap()
            .query_row([], |row| row.get(0))
            .unwrap();

        assert!(index_count > 0); // We created multiple indexes
    }
}