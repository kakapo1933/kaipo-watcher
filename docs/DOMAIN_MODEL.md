# Domain Model Documentation

## Problem Space

The kaipo-watcher tool addresses the critical need for transparent network usage monitoring in an era of:

- **Data Caps**: ISPs imposing monthly bandwidth limits
- **Usage-Based Billing**: Cloud and mobile data charges
- **Security Concerns**: Detecting unauthorized network access
- **Performance Issues**: Identifying bandwidth-hungry applications
- **Privacy Requirements**: Understanding what data leaves the system

## Core Domain Concepts

### 1. Network Traffic

The fundamental unit of observation in our domain model.

```rust
pub struct NetworkPacket {
    // Identity
    pub timestamp: Instant,
    pub interface: NetworkInterface,
    
    // Protocol layers
    pub link_layer: LinkLayer,
    pub network_layer: NetworkLayer,
    pub transport_layer: TransportLayer,
    pub application_layer: Option<ApplicationLayer>,
    
    // Metrics
    pub size_bytes: u64,
    pub direction: TrafficDirection,
}

pub enum TrafficDirection {
    Inbound,  // Data received
    Outbound, // Data sent
    Local,    // Inter-process communication
}
```

**Business Logic**: Network traffic represents the raw material from which all insights are derived. The model captures both technical details (protocols, addresses) and business-relevant metrics (size, direction).

### 2. Bandwidth

The rate of data transfer, a key performance indicator.

```rust
pub struct Bandwidth {
    pub download_bps: u64,  // Bits per second
    pub upload_bps: u64,
    pub measurement_interval: Duration,
}

impl Bandwidth {
    pub fn to_human_readable(&self) -> String {
        // Business rule: Display in most appropriate unit
        // Kbps for < 1 Mbps, Mbps for < 1 Gbps, etc.
    }
    
    pub fn is_saturated(&self, capacity: &LinkCapacity) -> bool {
        // Business rule: >90% of capacity = saturated
        self.total_bps() > capacity.max_bps * 0.9
    }
}
```

**Business Logic**: Bandwidth measurements drive real-time monitoring and performance alerts. The domain model includes intelligent unit conversion and saturation detection.

### 3. Data Usage

Accumulated network traffic over time periods.

```rust
pub struct DataUsage {
    pub period: UsagePeriod,
    pub downloaded_bytes: u64,
    pub uploaded_bytes: u64,
    pub application_breakdown: HashMap<ApplicationId, AppUsage>,
}

pub enum UsagePeriod {
    Daily(NaiveDate),
    Weekly { start: NaiveDate, end: NaiveDate },
    Monthly { year: i32, month: u32 },
    Custom { start: DateTime<Local>, end: DateTime<Local> },
}

impl DataUsage {
    pub fn percentage_of_limit(&self, limit: &DataLimit) -> f64 {
        // Business rule: Calculate usage against configured limits
    }
    
    pub fn projected_monthly_usage(&self) -> u64 {
        // Business rule: Linear projection based on current rate
    }
    
    pub fn cost_estimate(&self, plan: &DataPlan) -> Money {
        // Business rule: Calculate overage charges
    }
}
```

**Business Logic**: Data usage tracking enables budget management and prevents overage charges. The model supports various billing cycles and cost calculations.

### 4. Application Identity

Mapping network traffic to specific applications.

```rust
pub struct Application {
    pub id: ApplicationId,
    pub name: String,
    pub executable_path: PathBuf,
    pub process_info: ProcessInfo,
    pub category: ApplicationCategory,
}

pub enum ApplicationCategory {
    Browser,
    Streaming,
    Gaming,
    CloudStorage,
    Development,
    System,
    Unknown,
}

pub struct ApplicationMatcher {
    // Business rules for identifying applications
    pub by_port: HashMap<u16, ApplicationHint>,
    pub by_domain: HashMap<String, ApplicationHint>,
    pub by_signature: Vec<PacketSignature>,
}
```

**Business Logic**: Application identification is crucial for understanding usage patterns. The model uses multiple strategies to accurately map traffic to applications.

### 5. Network Interface

Physical or virtual network adapters.

```rust
pub struct NetworkInterface {
    pub id: InterfaceId,
    pub name: String,  // e.g., "eth0", "wlan0"
    pub type: InterfaceType,
    pub addresses: Vec<IpAddr>,
    pub link_capacity: Option<LinkCapacity>,
}

pub enum InterfaceType {
    Ethernet,
    WiFi,
    Cellular,
    VPN,
    Loopback,
    Virtual,
}

impl NetworkInterface {
    pub fn is_metered(&self) -> bool {
        // Business rule: Cellular and some WiFi are metered
        matches!(self.type, InterfaceType::Cellular) || 
        self.has_metered_flag()
    }
}
```

**Business Logic**: Different interface types have different monitoring requirements. Cellular connections need stricter monitoring due to typical data caps.

### 6. Alert System

Proactive notifications for important events.

```rust
pub struct Alert {
    pub id: AlertId,
    pub type: AlertType,
    pub severity: AlertSeverity,
    pub triggered_at: DateTime<Local>,
    pub context: AlertContext,
}

pub enum AlertType {
    DataLimitApproaching { percentage: f64 },
    UnusualActivity { application: ApplicationId },
    BandwidthSaturation { duration: Duration },
    NewApplication { application: ApplicationId },
    SecurityAnomaly { details: AnomalyDetails },
}

pub struct AlertRule {
    pub condition: AlertCondition,
    pub action: AlertAction,
    pub cooldown: Duration, // Prevent spam
}
```

**Business Logic**: Alerts transform raw monitoring data into actionable insights. The model includes anti-spam measures and contextual information.

### 7. Usage Patterns

Statistical models of normal behavior.

```rust
pub struct UsagePattern {
    pub time_of_day: HashMap<u8, HourlyStats>,
    pub day_of_week: HashMap<Weekday, DailyStats>,
    pub application_habits: HashMap<ApplicationId, AppHabits>,
}

pub struct AnomalyDetector {
    pub baseline: UsagePattern,
    pub sensitivity: SensitivityLevel,
    
    pub fn is_anomalous(&self, current: &DataUsage) -> Option<Anomaly> {
        // Business rule: Statistical deviation detection
        // e.g., 3x normal usage = anomaly
    }
}
```

**Business Logic**: Understanding normal patterns enables intelligent alerting and prevents false positives. The model adapts to user behavior over time.

## Domain Boundaries

### What's In Scope

1. **Local Network Monitoring**: All traffic through system interfaces
2. **Application-Level Tracking**: Which apps use bandwidth
3. **Usage Analytics**: Trends, predictions, anomalies
4. **Cost Management**: Data plan tracking and projections
5. **Performance Metrics**: Bandwidth, latency, packet loss

### What's Out of Scope

1. **Deep Packet Inspection**: Content analysis (privacy)
2. **Network Configuration**: Changing network settings
3. **Traffic Shaping**: Controlling bandwidth allocation
4. **Remote Monitoring**: Other devices on network
5. **Cloud Integration**: Uploading data to external services

## Business Rules

### Data Retention

```rust
pub struct RetentionPolicy {
    pub raw_packets: Duration,      // Hours (privacy)
    pub minute_aggregates: Duration, // Days
    pub hourly_aggregates: Duration, // Months
    pub daily_aggregates: Duration,  // Years
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            raw_packets: Duration::hours(0),      // Don't store
            minute_aggregates: Duration::days(7),
            hourly_aggregates: Duration::days(90),
            daily_aggregates: Duration::days(365),
        }
    }
}
```

**Rationale**: Balance between useful history and storage/privacy concerns.

### Alert Thresholds

```rust
pub struct AlertThresholds {
    // Data limit warnings
    pub data_limit_warning: f64,     // 80% default
    pub data_limit_critical: f64,    // 95% default
    
    // Bandwidth saturation
    pub bandwidth_saturated: f64,    // 90% of capacity
    pub saturation_duration: Duration, // 5 minutes
    
    // Anomaly detection
    pub usage_spike_multiplier: f64, // 3x normal
    pub new_app_data_threshold: u64, // 100MB first day
}
```

**Rationale**: Provide early warnings while avoiding false alarms.

### Cost Calculations

```rust
pub trait DataPlan {
    fn base_cost(&self) -> Money;
    fn included_data(&self) -> u64;
    fn overage_rate(&self) -> Money; // Per GB
    
    fn calculate_cost(&self, usage: &DataUsage) -> Cost {
        let base = self.base_cost();
        let overage = usage.total_bytes().saturating_sub(self.included_data());
        let overage_cost = self.overage_rate() * (overage / GB);
        
        Cost {
            base,
            overage: overage_cost,
            total: base + overage_cost,
        }
    }
}
```

**Rationale**: Accurate cost tracking helps users make informed decisions.

## Domain Events

Key events that drive the system:

```rust
pub enum DomainEvent {
    // Traffic events
    PacketCaptured(NetworkPacket),
    ApplicationIdentified { packet_id: PacketId, app: ApplicationId },
    
    // Usage events  
    DataLimitApproaching { limit: DataLimit, current: DataUsage },
    DataLimitExceeded { limit: DataLimit, current: DataUsage },
    
    // Performance events
    BandwidthSaturated { interface: InterfaceId, bandwidth: Bandwidth },
    LatencySpike { target: IpAddr, latency: Duration },
    
    // Security events
    UnknownApplicationDetected { executable: PathBuf },
    AnomalousTrafficPattern { details: AnomalyDetails },
    
    // System events
    MonitoringStarted { interfaces: Vec<InterfaceId> },
    MonitoringStopped { reason: StopReason },
}
```

## Domain Invariants

Rules that must always be true:

1. **Conservation of Bandwidth**: Upload + Download ≤ Link Capacity
2. **Temporal Ordering**: Packet timestamps must increase monotonically
3. **Application Uniqueness**: One application per process ID at any time
4. **Usage Accumulation**: Period usage = Sum of all packet sizes in period
5. **Alert Uniqueness**: No duplicate alerts within cooldown period

## Value Objects

Immutable domain concepts:

```rust
// Network addressing
pub struct MacAddress([u8; 6]);
pub struct IpAddress(std::net::IpAddr);
pub struct Port(u16);

// Data units with automatic conversion
pub struct DataSize(u64);
impl DataSize {
    pub fn bytes(&self) -> u64 { self.0 }
    pub fn kilobytes(&self) -> f64 { self.0 as f64 / 1024.0 }
    pub fn megabytes(&self) -> f64 { self.0 as f64 / 1_048_576.0 }
    pub fn gigabytes(&self) -> f64 { self.0 as f64 / 1_073_741_824.0 }
}

// Time periods
pub struct BillingCycle {
    pub start_day: u8,  // 1-31
    pub timezone: Tz,
}
```

## Domain Services

Stateless operations on domain objects:

```rust
pub trait PacketClassifier {
    fn classify(&self, packet: &NetworkPacket) -> Classification;
}

pub trait UsageCalculator {
    fn calculate_period_usage(&self, 
        packets: &[NetworkPacket], 
        period: &UsagePeriod
    ) -> DataUsage;
}

pub trait CostEstimator {
    fn estimate_monthly_cost(&self,
        current_usage: &DataUsage,
        data_plan: &DataPlan
    ) -> CostProjection;
}
```

## Anti-Patterns to Avoid

1. **Over-Monitoring**: Storing every packet detail (privacy violation)
2. **Assumption-Based Classification**: Guessing apps from ports alone
3. **Fixed Thresholds**: Not adapting to user patterns
4. **Cloud Dependency**: Requiring internet for local monitoring
5. **Resource Hogging**: Using more resources than monitored apps

## Future Domain Extensions

Potential areas for domain model growth:

1. **Quality of Service**: Latency, jitter, packet loss tracking
2. **Protocol Analysis**: Deeper understanding of traffic types
3. **Multi-Device**: Household-wide usage tracking
4. **Predictive Analytics**: ML-based usage forecasting
5. **Integration Hub**: APIs for other monitoring tools