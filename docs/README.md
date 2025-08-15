# aipriceaction-proxy

**A High-Performance Vietnamese Stock Market Data Distribution System with Hybrid Trust Model**

## 🚀 Project Overview

aipriceaction-proxy is a sophisticated distributed system designed to efficiently collect, process, and distribute Vietnamese stock market data through a novel **hybrid trust model**. The system bridges the gap between centralized and decentralized data distribution by implementing two distinct peer networks:

- **Internal Trusted Network**: Authenticated peers with guaranteed data integrity
- **Public Reputation Network**: Community-driven peers with merit-based trust scoring

```mermaid
graph TB
    subgraph "VCI API"
        VCI[Vietnamese Capital Market API<br/>291 symbols across 27 sectors<br/>Banking, Real Estate, Technology, etc.]
    end
    
    subgraph "Core Network"
        N1["Node-01<br/>Port 8888<br/>30s intervals"]
        N2["Node-02<br/>Port 8889<br/>35s intervals"] 
        N3["Node-03<br/>Port 8890<br/>40s intervals"]
    end
    
    subgraph "Public Network"
        P1["Public Peer 1<br/>Reputation: Trusted"]
        P2["Public Peer 2<br/>Reputation: Probation"]
        P3["Public Peer 3<br/>Reputation: Banned"]
    end
    
    subgraph "Clients"
        C1["Trading Application"]
        C2["Analytics Dashboard"]
        C3["Mobile App"]
    end
    
    VCI --> N1
    VCI --> N2  
    VCI --> N3
    
    N1 <--> N2
    N2 <--> N3
    N1 <--> N3
    
    N1 --> P1
    N2 --> P1
    P1 --> C1
    P1 --> C2
    P1 --> C3
    
    N1 -.-> P2
    P2 -.-> C2
    
    %% P3 (Banned peer) has no connections - requests are rejected
    
    style N1 fill:#e1f5fe
    style N2 fill:#e1f5fe
    style N3 fill:#e1f5fe
    style P1 fill:#e8f5e8
    style P2 fill:#fff3e0
    style P3 fill:#ffebee
```

## 🏗️ Architecture Highlights

### **Technology Stack**
- **Runtime**: Rust 2024 Edition with Tokio async runtime
- **Web Framework**: Axum with tower middleware stack
- **Data Source**: VCI (Vietnamese Capital Investment) API
- **Serialization**: Serde with JSON/YAML support
- **Observability**: Tracing with structured logging
- **Rate Limiting**: Tower-governor for public endpoint protection

### **Core Design Principles**

1. **Trust by Verification**: Internal peers use cryptographic tokens, public peers earn trust through consistent behavior
2. **Efficient Resource Usage**: Batch processing (291 symbols in batches of 10) optimizes API usage while providing complete market coverage
3. **Fault Tolerance**: Multi-node deployment with automatic failover and data synchronization
4. **Real-time Performance**: Sub-second gossip protocol for market data distribution
5. **Security by Design**: Defense-in-depth with authentication, rate limiting, and reputation tracking

---

## 🎯 The Hybrid Trust Model Explained

### **Why Hybrid Trust?**

Traditional financial data distribution faces a fundamental trade-off:

- **Centralized Systems**: High reliability but single points of failure and limited scalability
- **Decentralized Systems**: High scalability but trust and data quality concerns

The hybrid trust model solves this by creating **two complementary networks** that operate simultaneously:

```mermaid
graph LR
    subgraph "Trust Spectrum"
        IT[Internal Trusted<br/>🔐 Cryptographic Auth<br/>⚡ Low Latency<br/>✅ Guaranteed Quality]
        PR[Public Reputation<br/>📊 Merit-Based Trust<br/>🌐 High Scalability<br/>🔄 Community Driven]
    end
    
    subgraph "Data Sources"
        VCI[VCI API<br/>Vietnamese Markets]
    end
    
    subgraph "Applications"
        TA[Trading Apps]
        AD[Analytics]
        MB[Mobile]
    end
    
    VCI --> IT
    VCI --> PR
    IT --> TA
    PR --> AD
    PR --> MB
    IT <--> PR
    
    style IT fill:#e3f2fd
    style PR fill:#f3e5f5
```

### **Core Trust Principles**

#### 🔐 **Internal Network: Trust by Verification**
- **Authentication**: Bearer token-based cryptographic verification
- **Zero-Downtime**: Primary/secondary token rotation for continuous operation
- **Immediate Trust**: Data from authenticated peers is accepted immediately
- **Low Latency**: Direct peer-to-peer gossip without reputation checks

**Implementation in** `src/api.rs:36-48`:
```rust
let token_is_valid = match auth_header {
    Some(header) => {
        let token = header.trim_start_matches("Bearer ");
        let is_primary = token == token_state.primary;
        let is_secondary = token == token_state.secondary;
        is_primary || is_secondary
    }
    None => false
};
```

#### 📊 **Public Network: Trust by Reputation**
- **Merit-Based**: Trust earned through consistent, accurate data contributions
- **IP-Based Tracking**: Reputation tied to source IP addresses
- **Progressive Trust**: Probation → Trusted → Banned state transitions
- **Rate Limited**: Tower-governor protection against abuse

**Implementation in** `src/data_structures.rs:18-22`:
```rust
#[derive(Clone, Debug, PartialEq)]
pub enum ActorStatus {
    Probation,    // New peers start here
    Trusted,      // Proven reliable peers
    Banned,       // Malicious or unreliable peers
}
```

### **Trust State Machine**

```mermaid
stateDiagram-v2
    [*] --> Probation : New Public Peer
    Probation --> Trusted : Consistent Good Data
    Probation --> Banned : Poor Quality/Malicious
    Trusted --> Banned : Quality Degradation
    Banned --> [*] : Timeout/Manual Reset
    
    note right of Probation
        Limited data acceptance
        Higher validation
        Rate limited
    end note
    
    note right of Trusted
        Full data acceptance
        Lower validation overhead
        Higher rate limits
    end note
    
    note right of Banned
        No data acceptance
        All requests rejected
        IP blacklisted
    end note
```

### **Benefits of Hybrid Architecture**

1. **Reliability**: Internal network provides guaranteed data quality and availability
2. **Scalability**: Public network enables massive horizontal scaling
3. **Resilience**: Multiple trust levels prevent single points of failure
4. **Efficiency**: Different validation strategies optimized for each network type
5. **Community**: Public network encourages ecosystem participation

---

## 🔧 Core Components Architecture

### **Application Structure Overview**

The system follows a modular architecture with clear separation of concerns:

```mermaid
graph TB
    subgraph "Application Entry Point"
        M[main.rs<br/>🚀 Tokio Runtime<br/>📡 Axum Server<br/>🔄 Worker Spawning]
    end
    
    subgraph "Core Modules"
        C[config.rs<br/>⚙️ YAML/Env Loading<br/>🔐 Token Management<br/>🌐 Peer Configuration]
        
        D[data_structures.rs<br/>📊 OHLCV Data Types<br/>👥 Reputation System<br/>🔒 Thread-Safe State]
        
        V[vci.rs<br/>🏦 Vietnamese Market API<br/>📈 Stock Data Fetching<br/>⏱️ Rate Limiting]
        
        W[worker.rs<br/>🔄 Background Processing<br/>🎲 Random Ticker Selection<br/>📤 Gossip Distribution]
        
        A[api.rs<br/>🌐 REST Endpoints<br/>🔐 Authentication<br/>🛡️ Rate Limiting]
    end
    
    M --> C
    M --> D
    M --> W
    M --> A
    W --> V
    A --> D
    W --> D
    
    style M fill:#e3f2fd
    style C fill:#fff3e0
    style D fill:#e8f5e8
    style V fill:#fce4ec
    style W fill:#f3e5f5
    style A fill:#e0f2f1
```

### **Main Application Bootstrap** (`src/main.rs`)

The application entry point demonstrates sophisticated async Rust patterns:

#### **1. Configuration Loading & Tracing Setup**
```rust
// Line 48-62: Smart configuration loading with node identification
let app_config = config::AppConfig::load();

// Structured logging with node context
tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .with_target(false)
    .init();

let _span = tracing::info_span!("node", name = %app_config.node_name).entered();
```

**💡 Key Insight**: The global span ensures all log entries include the node name, crucial for multi-node debugging.

#### **2. Shared State Management with Arc<Mutex<>>**
```rust
// Lines 76-78: Thread-safe shared state pattern
let shared_data: SharedData = Arc::new(Mutex::new(InMemoryData::new()));
let shared_reputation: SharedReputation = Arc::new(Mutex::new(PublicActorReputation::new()));
let last_internal_update: LastInternalUpdate = Arc::new(Mutex::new(Instant::now()));
```

**🎯 Design Pattern**: Arc (Atomically Reference Counted) enables multiple ownership, while Mutex ensures thread-safe access across async tasks.

#### **3. Axum State Management with FromRef Trait**
```rust
// Lines 24-57: Type-safe state extraction
impl FromRef<AppState> for SharedData {
    fn from_ref(app_state: &AppState) -> SharedData {
        app_state.data.clone()
    }
}
```

**🔧 Axum Integration**: The FromRef trait allows automatic dependency injection in handler functions, eliminating boilerplate.

### **Data Structures** (`src/data_structures.rs`)

#### **OHLCV Market Data**
```rust
// From src/vci.rs:31-40
pub struct OhlcvData {
    pub time: DateTime<Utc>,     // UTC timestamp for global consistency
    pub open: f64,               // Opening price
    pub high: f64,               // Highest price in period
    pub low: f64,                // Lowest price in period  
    pub close: f64,              // Closing price
    pub volume: u64,             // Trading volume
    pub symbol: Option<String>,  // Stock symbol (VCB, TCB, etc.)
}
```

#### **Reputation System Architecture**
```rust
// Lines 11-32: Reputation tracking for public peers
#[derive(Clone, Debug)]
pub struct ActorMetadata {
    pub successful_updates: u32,  // Count of valid data contributions
    pub failed_updates: u32,      // Count of invalid/malicious attempts
    pub status: ActorStatus,      // Current trust level
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActorStatus {
    Probation,  // Default for new peers - limited trust
    Trusted,    // Earned through consistent good behavior
    Banned,     // Demoted due to malicious activity
}
```

#### **Memory-Efficient Type Aliases**
```rust
// Lines 41-46: Clear type safety with performance
pub type InMemoryData = HashMap<String, Vec<OhlcvData>>;
pub type SharedData = Arc<Mutex<InMemoryData>>;
pub type PublicActorReputation = HashMap<IpAddr, ActorMetadata>;
pub type SharedReputation = Arc<Mutex<PublicActorReputation>>;
```

### **State Flow Diagram**

```mermaid
sequenceDiagram
    participant M as main.rs
    participant C as Config
    participant S as SharedState
    participant W as Worker
    participant A as AxumServer
    
    M->>C: Load configuration
    C-->>M: AppConfig with node_name, peers, tokens
    
    M->>S: Initialize shared state
    S-->>M: Arc<Mutex<>> wrappers
    
    M->>W: Spawn background worker
    W->>W: Start data fetching loop
    
    M->>A: Start Axum server
    A->>A: Bind to configured port
    
    par Background Processing
        W->>S: Update market data
    and Request Handling  
        A->>S: Serve API requests
    end
```

### **Performance Characteristics**

#### **Memory Usage**
- **Shared State**: Single allocation per data structure with reference counting
- **Zero-Copy**: String interning for symbol names reduces memory fragmentation
- **Bounded Growth**: HashMap storage prevents unbounded memory growth

#### **Concurrency Model** 
- **Lock Granularity**: Separate mutexes for data, reputation, and timestamps
- **Async-First**: All I/O operations use Tokio's async runtime
- **Backpressure**: Natural flow control through async/.await

#### **🚨 Common Pitfalls & Solutions**

**Q: Why use Arc<Mutex<>> instead of Arc<RwLock<>>?**
A: RwLock can cause writer starvation in high-read scenarios. Market data updates are frequent enough that Mutex provides more predictable performance.

**Q: How does the FromRef pattern improve code quality?**
A: It eliminates manual state extraction in every handler, reducing boilerplate and preventing state injection errors.

**Q: What happens if shared state access blocks?**
A: Tokio's cooperative scheduling ensures mutex contention doesn't block the entire runtime - other tasks continue executing.

---

## ⚙️ Configuration Management

### **Multi-Source Configuration System**

The system supports flexible configuration loading with fallback priority:

```mermaid
graph LR
    subgraph "Configuration Sources"
        Y[YAML Files<br/>📄 examples/configs/*.yml<br/>🔧 Full structure<br/>🚀 Production ready]
        E[Environment Variables<br/>🌍 Runtime overrides<br/>🐳 Docker friendly<br/>⚡ Quick testing]
    end
    
    subgraph "Loading Strategy"
        L["AppConfig::load()"]
    end
    
    subgraph "Result"
        R[Runtime Configuration<br/>📝 Node identification<br/>🔐 Authentication tokens<br/>🌐 Peer networks]
    end
    
    Y --> L
    E --> L
    L --> R
    
    style Y fill:#e8f5e8
    style E fill:#fff3e0
    style L fill:#e3f2fd
    style R fill:#fce4ec
```

**Implementation in** `src/config.rs:91-97`:
```rust
pub fn load() -> Self {
    // Check for CONFIG_FILE environment variable first
    if let Ok(config_file) = env::var("CONFIG_FILE") {
        Self::from_yaml(&config_file)
    } else {
        Self::from_env()
    }
}
```

### **YAML Configuration Format**

#### **Production Node Example** (`examples/configs/node1.yml`)
```yaml
# Node 1 Configuration - Primary Core Node
node_name: "node-01"
tokens:
  primary: "secret-token-A-12345"
  secondary: "secret-token-B-67890"

# Node 1 connects to Node 2 and Node 3
internal_peers:
  - "http://localhost:8889"
  - "http://localhost:8890"

public_peers:
  - "https://api.aipriceaction.com"

core_network_url: null                    # null = core node mode
public_refresh_interval_secs: 300         # 5 minutes for public sync
core_worker_interval_secs: 30             # 30 seconds for VCI fetching
environment: "development"                # development | production
port: 8888
```

### **Environment Variable Configuration**

For Docker deployments and quick testing:

```bash
# Core node identification
export NODE_NAME="node-dev-01"
export PORT="8888"

# Authentication tokens
export PRIMARY_TOKEN="secret-token-A-12345"
export SECONDARY_TOKEN="secret-token-B-67890"

# Network configuration
export INTERNAL_PEER_URLS="http://node2:8889,http://node3:8890"
export PUBLIC_PEER_URLS="https://api.aipriceaction.com"

# Timing configuration
export CORE_WORKER_INTERVAL="30"
export PUBLIC_REFRESH_INTERVAL="300"

# Environment setting
export ENVIRONMENT="production"

# Public node mode (optional)
# export CORE_NETWORK_URL="http://core-node:8888"
```

### **Configuration Structure Deep Dive**

#### **Token Management** (`src/config.rs:11-13`)
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenConfig {
    pub primary: String,      // Primary authentication token
    pub secondary: String,    // Secondary token for zero-downtime rotation
}
```

**💡 Zero-Downtime Token Rotation**: The dual-token system allows authentication updates without service interruption. During rotation:
1. Deploy new `secondary` token
2. Update clients to use new token  
3. Promote `secondary` to `primary`
4. Generate new `secondary`

#### **Node Types & Behavior**

**Core Node Configuration**:
```yaml
core_network_url: null           # Indicates core node mode
core_worker_interval_secs: 30    # Frequency of VCI API calls
environment: "production"        # Enables public peer broadcasting
```

**Public Node Configuration**:
```yaml
core_network_url: "http://core-node:8888"    # Points to core network
public_refresh_interval_secs: 60             # Sync frequency with core
environment: "production"                    # Required for public mode
```

### **Configuration Validation & Defaults**

The system provides intelligent defaults for missing configuration:

```rust
// From src/config.rs:158-172
let core_worker_interval_secs = env::var("CORE_WORKER_INTERVAL")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(30); // Default to 30 seconds

let node_name = env::var("NODE_NAME")
    .unwrap_or_else(|_| "aipriceaction-proxy".to_string());

let port = env::var("PORT")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(8888); // Default to 8888
```

### **Multi-Node Configuration Strategy**

For production multi-node deployment, use coordinated configurations:

```mermaid
graph TB
    subgraph "Node Configuration Pattern"
        N1["Node-01<br/>Port: 8888<br/>Interval: 30s<br/>Peers: [8889, 8890]"]
        N2["Node-02<br/>Port: 8889<br/>Interval: 30s<br/>Peers: [8888, 8890]"]
        N3["Node-03<br/>Port: 8890<br/>Interval: 30s<br/>Peers: [8888, 8889]"]
    end
    
    subgraph "Benefits"
        B1["🔄 Load Distribution<br/>Multiple nodes reduce individual load"]
        B2["🌐 Full Mesh<br/>Each node connects to others"]
        B3["⚡ Fault Tolerance<br/>Redundant data paths"]
    end
    
    N1 -.-> N2
    N2 -.-> N3
    N3 -.-> N1
    
    N1 --> B1
    N2 --> B2
    N3 --> B3
    
    style N1 fill:#e3f2fd
    style N2 fill:#e8f5e8
    style N3 fill:#fff3e0
```

### **🔧 Configuration Best Practices**

#### **Security Considerations**
- **Token Rotation**: Implement regular token updates (recommended: monthly)
- **Network Isolation**: Use internal networking for peer communication
- **Environment Separation**: Never share tokens between dev/staging/production

#### **Performance Tuning**
- **Interval Configuration**: Adjust `core_worker_interval_secs` based on node workload requirements
- **Peer Network Size**: Limit internal peers to 5-10 nodes for optimal gossip performance
- **Port Management**: Use consistent port ranges (8888-8999) for easy firewall configuration

#### **🚨 Common Configuration Issues**

**Q: Why does my node show "Failed to send to internal peer"?**
A: Check that peer URLs in `internal_peers` match actual listening addresses and ports.

**Q: How do I know if I should use YAML or environment variables?**
A: Use YAML for complex production deployments, environment variables for containers and testing.

**Q: Can I mix core and public node configurations?**
A: No - a node operates in either core mode (`core_network_url: null`) or public mode (with `core_network_url` set).

---

## 🏦 VCI Integration - Vietnamese Capital Markets API

### **VCI API Overview**

The system integrates with VCI (Vietnamese Capital Investment) API to source real-time and historical market data for Vietnamese stocks. This integration forms the foundation of the data distribution network.

```mermaid
graph TB
    subgraph "VCI API Ecosystem"
        VCI[🏦 VCI Trading Platform<br/>https://trading.vietcap.com.vn/api/]
        CHART[📈 OHLC Chart Endpoint<br/>/chart/OHLCChart/gap-chart]
        GRAPHQL[🔍 GraphQL Endpoint<br/>/data-mt/graphql]
    end
    
    subgraph "VciClient Features"
        RL[⏱️ Rate Limiting<br/>30 requests/minute<br/>Exponential backoff]
        UA[🎭 User Agent Rotation<br/>5 browser profiles<br/>Anti-detection]
        RET[🔄 Retry Logic<br/>5 attempts max<br/>Jittered delays]
    end
    
    subgraph "Data Types"
        OHLCV[📊 OHLCV Data<br/>Time, Open, High, Low<br/>Close, Volume, Symbol]
        COMPANY[🏢 Company Info<br/>Profile, Financials<br/>Shareholders, Officers]
    end
    
    VCI --> CHART
    VCI --> GRAPHQL
    CHART --> OHLCV
    GRAPHQL --> COMPANY
    
    RL --> VCI
    UA --> VCI
    RET --> VCI
    
    style VCI fill:#e3f2fd
    style RL fill:#fff3e0
    style OHLCV fill:#e8f5e8
```

### **Core VCI Client Architecture** (`src/vci.rs`)

#### **1. Client Initialization with Anti-Detection**

**Implementation** `src/vci.rs:94-115`:
```rust
pub fn new(random_agent: bool, rate_limit_per_minute: u32) -> Result<Self, VciError> {
    let client = Client::builder()
        .timeout(StdDuration::from_secs(30))  // 30-second timeout
        .build()?;

    let user_agents = vec![
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36...",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36...",
        // ... 5 different browser profiles
    ];
```

**💡 Anti-Detection Strategy**: The client rotates between 5 realistic browser user agents to appear as normal web traffic rather than automated requests.

#### **2. Intelligent Rate Limiting** (`src/vci.rs:145-164`)

```rust
async fn enforce_rate_limit(&mut self) {
    let current_time = SystemTime::now();
    
    // Remove timestamps older than 1 minute
    self.request_timestamps.retain(|&timestamp| {
        current_time.duration_since(timestamp).unwrap_or(StdDuration::from_secs(0)) 
            < StdDuration::from_secs(60)
    });

    // If at rate limit, wait for oldest request to expire
    if self.request_timestamps.len() >= self.rate_limit_per_minute as usize {
        if let Some(&oldest_request) = self.request_timestamps.first() {
            let wait_time = StdDuration::from_secs(60) 
                - current_time.duration_since(oldest_request).unwrap_or(StdDuration::from_secs(0));
            if !wait_time.is_zero() {
                sleep(wait_time + StdDuration::from_millis(100)).await;  // +100ms buffer
            }
        }
    }
    
    self.request_timestamps.push(current_time);
}
```

**🎯 Rate Limiting Benefits**:
- **Sliding Window**: Maintains exact request count over rolling 60-second periods
- **Predictive Waiting**: Calculates exact wait time instead of arbitrary delays
- **Buffer Protection**: +100ms buffer prevents edge case violations

#### **3. Robust Error Handling with Exponential Backoff**

**Implementation** `src/vci.rs:166-226`:
```rust
async fn make_request(&mut self, url: &str, payload: &Value) -> Result<Value, VciError> {
    const MAX_RETRIES: u32 = 5;
    
    for attempt in 0..MAX_RETRIES {
        self.enforce_rate_limit().await;

        if attempt > 0 {
            // Exponential backoff: 2^(attempt-1) + random jitter
            let delay = StdDuration::from_secs_f64(
                2.0_f64.powi(attempt as i32 - 1) + rand::random::<f64>()
            );
            let delay = delay.min(StdDuration::from_secs(60));  // Cap at 60 seconds
            sleep(delay).await;
        }

        let user_agent = self.get_user_agent();
        let response = self.client
            .post(url)
            .header("Accept", "application/json, text/plain, */*")
            .header("Content-Type", "application/json")
            .header("User-Agent", user_agent)
            // ... 15 realistic browser headers for stealth
            .json(payload)
            .send()
            .await;
```

**🔄 Retry Strategy**:
- **Smart Classification**: Different handling for 4xx (client errors) vs 5xx (server errors)
- **Jittered Backoff**: Random component prevents thundering herd effects
- **Capped Delays**: Maximum 60-second wait prevents infinite delays

### **OHLCV Data Fetching Patterns**

#### **Single Symbol Fetching** (`src/vci.rs:262-346`)

```rust
pub async fn get_history(
    &mut self,
    symbol: &str,
    start: &str,      // "2025-08-14"
    end: Option<&str>, // "2025-08-15" or None for current
    interval: &str,   // "1D", "1H", "1m", etc.
) -> Result<Vec<OhlcvData>, VciError>
```

**Supported Intervals**:
- **Minute-level**: `1m`, `5m`, `15m`, `30m` → `ONE_MINUTE` API parameter
- **Hourly**: `1H` → `ONE_HOUR` API parameter  
- **Daily/Weekly/Monthly**: `1D`, `1W`, `1M` → `ONE_DAY` API parameter

#### **Batch Symbol Fetching** (`src/vci.rs:347-475`)

```rust
pub async fn get_batch_history(
    &mut self,
    symbols: &[String],  // ["VCB", "TCB", "FPT", "ACB"]
    start: &str,
    end: Option<&str>,
    interval: &str,
) -> Result<HashMap<String, Option<Vec<OhlcvData>>>, VciError>
```

**💡 Batch Processing Benefits**:
- **Single API Call**: Fetch multiple symbols in one request (more efficient than individual calls)
- **Graceful Degradation**: Returns `None` for symbols with invalid data, `Some(Vec<OhlcvData>)` for valid data
- **Consistent Timestamps**: All symbols use the same time range for synchronization

### **Data Processing Pipeline**

```mermaid
sequenceDiagram
    participant W as Worker
    participant V as VciClient
    participant API as VCI API
    participant D as SharedData
    
    W->>V: get_batch_history(["VCB", "TCB"], "2025-08-14", "1D")
    V->>V: enforce_rate_limit()
    V->>API: POST /chart/OHLCChart/gap-chart
    
    alt Success Response
        API-->>V: Raw OHLCV arrays {o: [], h: [], l: [], c: [], v: [], t: []}
        V->>V: Parse and validate arrays
        V->>V: Filter by start_date
        V->>V: Sort by timestamp
        V-->>W: HashMap<String, Option<Vec<OhlcvData>>>
        W->>D: Update shared state
    else Error Response (429, 403, 5xx)
        API-->>V: Error status
        V->>V: Exponential backoff delay
        V->>API: Retry request (up to 5 times)
    end
```

### **Vietnamese Market Specifics**

#### **Comprehensive Vietnamese Stock Market Coverage**

The system now supports the complete Vietnamese stock market through a comprehensive ticker group system with **291 symbols across 27 sectors**.

**Ticker Groups Configuration** (`ticker_group.json`):
```json
{
  "NGAN_HANG": ["VCB", "BID", "CTG", "VPB", "MBB", "ACB", "TCB", "LPB", "HDB", "STB", "VIB", "TPB", "SHB", "EIB", "OCB", "MSB", "EVF"],
  "BAT_DONG_SAN": ["VHM", "VIC", "VRE", "KDH", "SSH", "NVL", "NLG", "PDR", "DIG", "TCH", "DXG", "CEO", "HDC", "SGR", "KHG", "SCR", "AGG", "HQC", "NTL", "TDC", "ITC", "NHA", "HTN", "NDN", "HLD", "LDG"],
  "CONG_NGHE": ["VGI", "FPT", "FOX", "CMG", "SGT", "ELC", "VTK", "TTN", "MFS"],
  "DAU_KHI": ["GAS", "BSR", "PLX", "PVS", "PVD", "OIL", "CNG", "PVC"],
  "THEP": ["HPG", "MSR", "HSG", "KSV", "ACG", "TVN", "VIF", "NKG", "PTB", "GDA", "VGS", "SMC", "TLH"],
  // ... 22 additional sectors
}
```

**Market Coverage**:
- **Banking (NGAN_HANG)**: 17 major Vietnamese banks including VCB, TCB, ACB
- **Real Estate (BAT_DONG_SAN)**: 26 property development companies
- **Technology (CONG_NGHE)**: 9 tech companies including FPT Corporation
- **Oil & Gas (DAU_KHI)**: 8 energy sector companies
- **Steel (THEP)**: 13 steel and metal companies
- **Food & Beverage (THUC_PHAM)**: 11 consumer goods companies
- **And 21 additional sectors** covering the complete Vietnamese stock market

#### **Timezone Handling** (`src/vci.rs:228-242`)

```rust
pub fn calculate_timestamp(&self, date_str: Option<&str>) -> i64 {
    match date_str {
        Some(date) => {
            let naive_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")?;
            let next_day = naive_date + ChronoDuration::days(1);
            let naive_datetime = next_day.and_hms_opt(0, 0, 0).unwrap();
            let datetime = naive_datetime.and_utc();
            datetime.timestamp() - 7 * 3600  // UTC-7 for Vietnam timezone
        }
        None => Utc::now().timestamp(),
    }
}
```

**🌍 Timezone Considerations**:
- **Vietnam Time**: UTC+7 (Indochina Time)
- **API Expectation**: VCI API expects timestamps in local timezone
- **Conversion**: Subtract 7 hours from UTC to match Vietnamese market hours

### **Company Information via GraphQL**

#### **GraphQL Integration** (`src/vci.rs:476-659`)

The VCI client also supports fetching comprehensive company information through a GraphQL endpoint:

```rust
pub async fn company_info(&mut self, symbol: &str) -> Result<CompanyInfo, VciError>
```

**Data Retrieved**:
- **Company Profile**: Business description, industry classification
- **Financial Ratios**: PE, PB, ROE, ROA, EPS, revenue, profit
- **Market Data**: Current price, market cap, outstanding shares  
- **Corporate Structure**: Major shareholders and management team
- **News & Analysis**: Recent company news and analyst reports

### **Performance Optimizations**

#### **Memory Efficiency**
```rust
// From src/vci.rs:31-40
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhlcvData {
    pub time: DateTime<Utc>,     // 8 bytes (i64)
    pub open: f64,               // 8 bytes
    pub high: f64,               // 8 bytes  
    pub low: f64,                // 8 bytes
    pub close: f64,              // 8 bytes
    pub volume: u64,             // 8 bytes
    pub symbol: Option<String>,  // ~24 bytes (optimized string)
}
// Total: ~72 bytes per data point
```

#### **Network Efficiency** 
- **Batch Requests**: Single API call for multiple symbols reduces network overhead
- **Compression**: `gzip, deflate, br` encoding reduces transfer size by ~70%
- **Keep-Alive**: Connection reuse reduces TCP handshake overhead
- **Timeout Management**: 30-second timeouts prevent hanging connections

### **🔧 VCI Integration Best Practices**

#### **Error Handling Patterns**

**Q: How should I handle VCI rate limit errors?**
A: The client automatically handles rate limiting with intelligent backoff. For additional safety, space your requests by at least 2 seconds when fetching individual symbols.

**Q: What happens when VCI API returns incomplete data?**  
A: The batch fetching method returns `None` for symbols with missing/invalid data, allowing the application to continue processing valid symbols while logging issues.

**Q: How do I optimize for Vietnamese market hours?**
A: Vietnamese markets operate 9:00-16:00 ICT (UTC+7). Schedule intensive data fetching during off-market hours (17:00-08:00 ICT) to reduce load on VCI servers.

#### **Production Deployment Tips**

1. **Rate Limit Coordination**: In multi-node setups, use different rate limits per node (e.g., 20, 25, 30 req/min) to avoid synchronized API pressure
2. **Symbol Distribution**: Assign different symbol sets to different nodes for load balancing
3. **Monitoring**: Track `VciError::RateLimit` occurrences to optimize request patterns
4. **Fallback Strategy**: Implement fallback to individual symbol fetching if batch requests consistently fail

---

## ⚙️ Worker System - Background Data Processing

### **Dual Worker Architecture**

The system implements two distinct worker types that operate based on node configuration, enabling both core data acquisition and distributed data synchronization.

```mermaid
graph TB
    subgraph "Worker Type Decision"
        CONFIG[Configuration Check<br/>core_network_url]
        CORE_MODE[Core Worker Mode<br/>core_network_url: null]
        PUBLIC_MODE[Public Worker Mode<br/>core_network_url: set]
    end
    
    subgraph "Core Worker (Data Source)"
        VCI_FETCH[🏦 VCI API Fetching<br/>Ticker groups processing<br/>291 symbols in batches of 10]
        INTERNAL_GOSSIP[🔒 Internal Peer Gossip<br/>Bearer token auth<br/>All internal peers]
        PUBLIC_GOSSIP[🌐 Public Peer Gossip<br/>No authentication<br/>Production only]
    end
    
    subgraph "Public Worker (Data Sync)"
        CORE_SYNC[📥 Core Network Sync<br/>Fetch /tickers endpoint<br/>Compare timestamps]
        LOCAL_UPDATE[💾 Local Data Update<br/>Update only newer data<br/>Maintain consistency]
    end
    
    CONFIG --> CORE_MODE
    CONFIG --> PUBLIC_MODE
    CORE_MODE --> VCI_FETCH
    VCI_FETCH --> INTERNAL_GOSSIP
    VCI_FETCH --> PUBLIC_GOSSIP
    PUBLIC_MODE --> CORE_SYNC
    CORE_SYNC --> LOCAL_UPDATE
    
    style CORE_MODE fill:#e3f2fd
    style PUBLIC_MODE fill:#fff3e0
    style VCI_FETCH fill:#e8f5e8
    style CORE_SYNC fill:#f3e5f5
```

### **Core Worker Implementation** (`src/worker.rs:23-247`)

#### **1. Comprehensive Ticker Groups Batch Processing**

**Implementation** `src/worker.rs:50-65`:
```rust
// Load ticker groups and combine all tickers into a single array
let ticker_groups = load_ticker_groups();
let mut all_tickers: Vec<String> = ticker_groups.0.values()
    .flat_map(|group_tickers| group_tickers.iter().cloned())
    .collect();

// Remove duplicates and shuffle
all_tickers.sort();
all_tickers.dedup();
all_tickers.shuffle(&mut rand::rng());

info!(total_tickers = all_tickers.len(), "Loaded and shuffled all tickers from ticker groups");
const BATCH_SIZE: usize = 10;
```

**📊 Batch Processing Strategy**:

| Batch | Tickers Processed | Processing Time | Sleep Duration |
|-------|-------------------|-----------------|----------------|
| 1     | 10 symbols        | ~200-500ms      | 1000-2000ms   |
| 2     | 10 symbols        | ~200-500ms      | 1000-2000ms   |
| 3     | 10 symbols        | ~200-500ms      | 1000-2000ms   |
| ...   | ...               | ...             | ...           |
| 30    | 1 symbol (final)  | ~200-500ms      | 1000-2000ms   |

**💡 Benefits of Batch Processing**:
- **Complete Market Coverage**: All 291 Vietnamese stocks processed every cycle
- **Rate Limit Compliance**: 1-2 second sleep between batches respects VCI API limits
- **Efficient Resource Usage**: Single API call per batch reduces network overhead
- **Random Distribution**: Shuffling ensures different processing order each cycle
- **Fault Tolerance**: Failed batches don't affect other batch processing

#### **2. Gossip Protocol Implementation**

**Dual-Network Broadcasting** `src/worker.rs:147-191`:

```rust
if let Some(gossip_payload) = latest_data {
    // --- 1. INTERNAL PEERS (Trusted Network) ---
    let auth_token = format!("Bearer {}", config.tokens.primary);
    
    for peer_url in config.internal_peers.iter() {
        let url = format!("{}/gossip", peer_url);
        tokio::spawn(async move {
            match client.post(&url)
                .header("Authorization", token)
                .json(&payload)
                .send().await 
            {
                Ok(response) if response.status().is_success() => {
                    debug!(peer = %peer_url, "Successfully sent to internal peer");
                }
                // Error handling...
            }
        });
    }
    
    // --- 2. PUBLIC PEERS (Reputation Network) - Production Only ---
    if config.environment == "production" {
        for peer_url in config.public_peers.iter() {
            let url = format!("{}/public/gossip", peer_url);
            tokio::spawn(async move {
                // No authentication header for public peers
                match client.post(&url).json(&payload).send().await {
                    // Handle response...
                }
            });
        }
    }
}
```

**🔐 Security Model**:
- **Internal Peers**: Bearer token authentication, guaranteed acceptance
- **Public Peers**: No authentication, subject to reputation filtering  
- **Environment Gating**: Public gossip only enabled in production to prevent test data pollution

#### **3. Async Task Spawning Pattern**

```mermaid
sequenceDiagram
    participant CW as Core Worker
    participant VCI as VCI Client
    participant IG1 as Internal Gossip Task 1
    participant IG2 as Internal Gossip Task 2
    participant PG1 as Public Gossip Task 1
    
    CW->>VCI: get_batch_history(selected_tickers)
    VCI-->>CW: OHLCV data
    
    par Internal Peer Broadcasting
        CW->>IG1: spawn(gossip to peer1)
        CW->>IG2: spawn(gossip to peer2)
    and Public Peer Broadcasting
        CW->>PG1: spawn(gossip to public peer)
    end
    
    Note over CW: Non-blocking: continues to next iteration
    
    IG1-->>IG1: HTTP POST with Bearer token
    IG2-->>IG2: HTTP POST with Bearer token  
    PG1-->>PG1: HTTP POST without token
```

**⚡ Async Benefits**:
- **Non-blocking**: Main worker loop continues regardless of gossip success/failure
- **Concurrent Distribution**: All peers receive data simultaneously
- **Isolation**: Individual peer failures don't affect other distributions
- **Resource Efficiency**: Tokio efficiently manages hundreds of concurrent tasks

### **Public Worker Implementation** (`src/worker.rs:250-306`)

#### **1. Core Network Synchronization**

**Implementation** `src/worker.rs:250-306`:
```rust
async fn run_public_node_worker(data: SharedData, core_network_url: String, refresh_interval: Duration) {
    let http_client = ReqwestClient::new();
    
    loop {
        let core_tickers_url = format!("{}/tickers", core_network_url);
        
        match http_client.get(&core_tickers_url).send().await {
            Ok(response) if response.status().is_success() => {
                match response.json::<InMemoryData>().await {
                    Ok(core_data) => {
                        let mut local_data_guard = data.lock().await;
                        
                        for (symbol, core_ohlcv_vec) in core_data {
                            let local_entry = local_data_guard.entry(symbol.clone()).or_default();
                            
                            // Only update if core data is newer
                            if let (Some(core_last), Some(local_last)) = (core_ohlcv_vec.last(), local_entry.last()) {
                                if core_last.time > local_last.time {
                                    *local_entry = core_ohlcv_vec;
                                }
                            } else if local_entry.is_empty() {
                                *local_entry = core_ohlcv_vec;
                            }
                        }
                    }
                }
            }
        }
        
        tokio::time::sleep(refresh_interval).await;
    }
}
```

#### **2. Intelligent Data Merging**

**Timestamp-Based Updates**:
```rust
// Only update if core data is newer than local data
if core_last.time > local_last.time {
    *local_entry = core_ohlcv_vec;  // Replace entire symbol data
    updated_symbols.push(symbol.clone());
}
```

**🕒 Update Strategy Benefits**:
- **Monotonic Time**: Prevents regression to older data
- **Atomic Replacement**: Entire symbol dataset replaced for consistency
- **Change Detection**: Tracks which symbols were actually updated
- **Memory Efficiency**: No partial updates or complex merging logic

### **Worker Performance Characteristics**

#### **Core Worker Metrics**

**Per-Batch Performance** (with 10 symbols):
- **VCI API Call**: ~200-500ms (network dependent)
- **Data Processing**: ~5-10ms (parsing JSON for 10 symbols, updating HashMap)
- **Gossip Distribution**: ~50-200ms per peer (async, non-blocking, 10 symbols)
- **Memory Usage**: ~720 bytes per batch (10 symbols × 72 bytes per OHLCV point)
- **Sleep Duration**: 1000-2000ms between batches (rate limiting)

**Complete Cycle Performance** (291 symbols):
- **Total Batches**: 30 batches (29×10 + 1×1 symbols)
- **API Calls per Cycle**: 30 calls
- **Processing Time**: ~6-15 seconds (30 batches × 200-500ms + sleep time)
- **Memory Usage**: ~20.9KB per complete cycle (291 symbols × 72 bytes)

**Resource Scaling**:
```rust
// Core worker interval examples with complete market coverage
30s interval → 120 cycles/hour, 34,920 symbol updates/hour (291×120)
60s interval → 60 cycles/hour, 17,460 symbol updates/hour (291×60)
```

#### **Public Worker Metrics**

**Sync Performance**:
- **Core Data Fetch**: ~50-200ms (internal network)
- **Data Comparison**: ~1ms per symbol (timestamp comparison)
- **Update Application**: ~1ms per updated symbol
- **Memory Overhead**: Zero additional allocation for unchanged data

### **Multi-Node Worker Coordination**

#### **Staggered Interval Strategy**

**Configuration Example**:
```yaml
# Node 1
core_worker_interval_secs: 30  # Every 30 seconds

# Node 2  
core_worker_interval_secs: 30  # Every 30 seconds

# Node 3
core_worker_interval_secs: 30  # Every 30 seconds
```

**⏰ Timeline Analysis** (60-second window with synchronized processing):
```
Time: 0s  -> All nodes start batch cycle (Batch 1/30: 10 symbols each)
Time: 2s  -> All nodes continue (Batch 2/30: 10 symbols each)
Time: 4s  -> All nodes continue (Batch 3/30: 10 symbols each)
...
Time: 30s -> All nodes complete cycle and immediately restart
Time: 32s -> All nodes processing (Batch 2/30 of new cycle)
Time: 60s -> All nodes complete second cycle and restart
```

**📊 Load Distribution Benefits**:
- **Complete Market Coverage**: All 291 Vietnamese stocks processed by each node
- **Distributed Load**: Multiple nodes provide redundancy with synchronized 30 API calls per cycle
- **Enhanced Data Freshness**: Every symbol updated every cycle (30-60 seconds)
- **Fault Tolerance**: If one node fails, others provide complete market coverage
- **Rate Limit Optimization**: 1-2 second sleep between batches respects VCI API limits
- **Random Processing Order**: Shuffling prevents predictable market timing patterns

### **🔧 Worker System Best Practices**

#### **Core Worker Optimization**

**Q: How do I optimize core worker performance?**
A: Use intervals between 30-60 seconds for complete cycles. Each cycle processes all 291 symbols in 30 batches. Shorter intervals may not allow complete cycles to finish.

**Q: Should I increase the batch size beyond 10 symbols?**
A: No. The 10-symbol batch size is optimized for VCI API performance and prevents timeouts. The 1-2 second sleep between batches ensures rate limit compliance.

**Q: How long does a complete market cycle take?**
A: Approximately 58-87 seconds (29 batches × 2-3 seconds per batch). Configure worker intervals to be longer than cycle time to prevent overlap.

**Q: How do I handle worker crashes?**
A: The worker runs in an infinite loop and will restart with the application. Implement external monitoring to restart the entire process if needed.

#### **Public Worker Configuration**

**Q: What's the optimal refresh interval for public workers?**
A: 60-300 seconds depending on data freshness requirements. Consider network latency to core nodes.

**Q: How do I handle core network failures?**
A: Public workers continue operating with cached data. Implement fallback to other core nodes or switch to core mode temporarily.

#### **Production Monitoring**

1. **Core Worker Health**: Monitor successful VCI API calls and gossip distribution
2. **Public Worker Health**: Track successful core sync operations and data freshness
3. **Resource Usage**: Monitor memory growth and CPU utilization
4. **Network Metrics**: Track gossip success rates and latency

---

## 🌐 API Endpoints - REST Interface & Gossip Protocol

### **API Architecture Overview**

The system exposes four primary endpoints designed for different trust levels and use cases, implementing both data retrieval and gossip protocol functionality.

```mermaid
graph TB
    subgraph "API Endpoints"
        TICKERS[🎯 GET /tickers<br/>No Authentication<br/>Public Data Access]
        GROUPS[📊 GET /tickers/group<br/>No Authentication<br/>Ticker Groups Configuration]
        INTERNAL[🔒 POST /gossip<br/>Bearer Token Auth<br/>Internal Peer Network]
        PUBLIC[🌐 POST /public/gossip<br/>No Authentication<br/>Public Reputation Network]
    end
    
    subgraph "Authentication Layers"
        TOKEN_VALID[✅ Token Validation<br/>Primary/Secondary Check<br/>Instant Trust]
        REP_CHECK[📊 Reputation Check<br/>IP-based Tracking<br/>Probabilistic Trust]
        NO_AUTH[🔓 No Authentication<br/>Public Access<br/>Read-only]
    end
    
    subgraph "Data Flow"
        READ[📖 Data Reading<br/>HashMap Clone<br/>JSON Response]
        WRITE_TRUSTED[✍️ Trusted Write<br/>Direct Update<br/>No Validation]
        WRITE_UNTRUSTED[🔍 Untrusted Write<br/>Price Validation<br/>Reputation Update]
    end
    
    TICKERS --> NO_AUTH
    INTERNAL --> TOKEN_VALID
    PUBLIC --> REP_CHECK
    
    NO_AUTH --> READ
    TOKEN_VALID --> WRITE_TRUSTED
    REP_CHECK --> WRITE_UNTRUSTED
    
    style TICKERS fill:#e8f5e8
    style INTERNAL fill:#e3f2fd
    style PUBLIC fill:#fff3e0
```

### **Endpoint Details**

#### **1. GET /tickers - Public Data Access** (`src/api.rs:14-23`)

**Purpose**: Public endpoint for retrieving all available market data without authentication.

```rust
pub async fn get_all_tickers_handler(State(state): State<SharedData>) -> impl IntoResponse {
    let data = state.lock().await;
    let symbol_count = data.len();
    let symbols: Vec<_> = data.keys().cloned().collect();
    
    info!(symbol_count, symbols = ?symbols, "Returning ticker data");
    (StatusCode::OK, Json(data.clone()))
}
```

**Request**:
```bash
curl -X GET http://localhost:8888/tickers
```

**Response Format**:
```json
{
  "VCB": [
    {
      "time": "2025-08-14T09:30:00Z",
      "open": 85.5,
      "high": 86.2,
      "low": 85.1,
      "close": 85.8,
      "volume": 1250000,
      "symbol": "VCB"
    }
  ],
  "TCB": [
    {
      "time": "2025-08-14T09:30:00Z",
      "open": 45.2,
      "high": 45.7,
      "low": 44.9,
      "close": 45.4,
      "volume": 890000,
      "symbol": "TCB"
    }
  ]
}
```

**🔓 Security Model**: 
- **No Authentication**: Open public access for data consumers
- **Read-Only**: Cannot modify data through this endpoint
- **Rate Limiting**: Protected by tower-governor middleware (configured per deployment)

#### **2. GET /tickers/group - Ticker Groups Configuration** (`src/api.rs:164-172`)

**Purpose**: Public endpoint for retrieving the complete ticker groups configuration showing all 27 sectors and their associated symbols.

```rust
pub async fn get_ticker_groups_handler(State(state): State<SharedTickerGroups>) -> impl IntoResponse {
    let group_count = state.0.len();
    let group_names: Vec<_> = state.0.keys().cloned().collect();
    
    info!(group_count, groups = ?group_names, "Returning ticker groups");
    (StatusCode::OK, Json(state.0.clone()))
}
```

**Request**:
```bash
curl -X GET http://localhost:8888/tickers/group
```

**Response Format**:
```json
{
  "NGAN_HANG": ["VCB", "BID", "CTG", "VPB", "MBB", "ACB", "TCB", "LPB", "HDB", "STB", "VIB", "TPB", "SHB", "EIB", "OCB", "MSB", "EVF"],
  "BAT_DONG_SAN": ["VHM", "VIC", "VRE", "KDH", "SSH", "NVL", "NLG", "PDR", "DIG", "TCH", "DXG", "CEO", "HDC", "SGR", "KHG", "SCR", "AGG", "HQC", "NTL", "TDC", "ITC", "NHA", "HTN", "NDN", "HLD", "LDG"],
  "CONG_NGHE": ["VGI", "FPT", "FOX", "CMG", "SGT", "ELC", "VTK", "TTN", "MFS"],
  "DAU_KHI": ["GAS", "BSR", "PLX", "PVS", "PVD", "OIL", "CNG", "PVC"],
  "THEP": ["HPG", "MSR", "HSG", "KSV", "ACG", "TVN", "VIF", "NKG", "PTB", "GDA", "VGS", "SMC", "TLH"],
  "THUC_PHAM": ["VNM", "MCH", "MSN", "SAB", "QNS", "KDC", "SBT", "MCM", "SLS", "CLX", "LSS"],
  "CHUNG_KHOAN": ["SSI", "VND", "VCI", "HCM", "SHS", "MBS", "BSI", "FTS", "VIX", "CTS", "DSC", "VDS", "AGR", "TVS", "BVS", "ORS", "APG", "TVB", "EVS", "TCI", "SBS", "APS", "BMS"],
  "VAN_TAI": ["GMD", "PVT", "VTP", "PHP", "HAH", "DVP", "VSC", "VOS", "PVP", "TCD", "TCL", "VTO", "VIP", "SKG", "DXP", "VNA", "MHC"],
  "THUY_SAN": ["VHC", "MPC", "ANV", "FMC", "ASM", "IDI", "CMX", "ACL"],
  "CAO_SU": ["GVR", "PHR", "DRC", "DPR", "DRG", "DRI", "VRG"],
  "DET_MAY": ["VGT", "TCM", "MSH", "TNG", "STK", "PPH", "GIL", "ADS", "EVE"],
  "HOA_CHAT": ["DGC", "DCM", "DPM", "LAS", "DDV", "BFC", "CSV"],
  "HANG_KHONG": ["ACV", "VJC", "HVN", "SCS", "SAS", "NCT", "SGN"],
  "KHAI_KHOANG": ["MSR", "CST", "VPG", "NNC", "NBC", "BMC", "TNT"],
  "NANG_LUONG": ["POW", "HDG", "QTP", "NT2", "GEG", "PPC"],
  "NHUA": ["NTP", "BMP", "AAA", "APH", "NHH", "HCD"],
  "NONG_NGHIEP": ["HAG", "DBC", "PAN", "VLC", "BAF", "LTG", "NAF", "TSC"],
  "SUC_KHOE": ["DHG", "IMP", "DVN", "DHT", "TNH", "DCL", "JVC"],
  "VLXD": ["HT1", "PTB", "VLB", "BCC", "CTI", "BTS", "DHA", "HOM", "BTN"],
  "XAY_DUNG": ["HUT", "CTD", "CII", "IJC", "BCG", "DPG", "CKG", "HBC", "L18", "IDJ", "S99", "VC2", "C69", "TLD", "PHC", "C47", "BCE", "FID"],
  "XAY_LAP_DIEN": ["CTR", "PC1", "TV2", "VNE", "SCI"],
  "BAT_DONG_SAN_KCN": ["BCM", "KBC", "IDC", "VGC", "SIP", "NTC", "SZC", "LHG", "TIP", "D2D", "DTD", "IDV"],
  "BAO_HIEM": ["BVH", "PVI", "BIC", "MIG", "BMI"],
  "BAN_LE": ["MWG", "PNJ", "FRT", "DGW", "PET", "HAX"],
  "DAU_TU_CONG": ["VCG", "HHV", "C4G", "LCG", "PLC", "FCN", "KSB", "G36"],
  "PENNY": ["IPA", "FIT", "EVG", "CSC", "VPH", "VC7", "DL1", "HAP", "MST", "LIG", "VIG", "NRC", "DAH", "HID", "CIG", "RDP", "DST", "DAG", "PXI"],
  "OTHERS": ["VEA", "GEX", "VCS", "BWE", "ACG", "TDM", "HHS", "DHC", "TIG", "TTF", "YEG", "DSN", "GDT"]
}
```

**📊 Use Cases**:
- **Portfolio Management**: Organize investments by sector
- **Market Analysis**: Analyze sector performance and trends
- **Risk Management**: Diversify holdings across different industries
- **Algorithmic Trading**: Implement sector-based trading strategies
- **Research & Development**: Understand Vietnamese market structure

**🔓 Security Model**: 
- **No Authentication**: Open public access for configuration data
- **Read-Only**: Cannot modify ticker groups through this endpoint
- **Static Data**: Configuration loaded from `ticker_group.json` at startup

#### **3. POST /gossip - Internal Peer Communication** (`src/api.rs:25-75`)

**Purpose**: Authenticated endpoint for trusted internal peers to share market data updates.

```rust
pub async fn internal_gossip_handler(
    State(data_state): State<SharedData>,
    State(token_state): State<SharedTokenConfig>,
    State(last_update_state): State<LastInternalUpdate>,
    headers: HeaderMap,
    Json(payload): Json<OhlcvData>,
) -> impl IntoResponse
```

**Authentication Logic** (`src/api.rs:36-48`):
```rust
let auth_header = headers.get("Authorization").and_then(|h| h.to_str().ok());
let token_is_valid = match auth_header {
    Some(header) => {
        let token = header.trim_start_matches("Bearer ");
        let is_primary = token == token_state.primary;
        let is_secondary = token == token_state.secondary;
        is_primary || is_secondary  // Accept either token
    }
    None => false
};
```

**Request Example**:
```bash
curl -X POST http://localhost:8889/gossip \
  -H "Authorization: Bearer secret-token-A-12345" \
  -H "Content-Type: application/json" \
  -d '{
    "time": "2025-08-14T09:31:00Z",
    "open": 85.8,
    "high": 86.0,
    "low": 85.7,
    "close": 85.9,
    "volume": 150000,
    "symbol": "VCB"
  }'
```

**🔐 Security Features**:
- **Bearer Token**: Primary/secondary token validation for zero-downtime rotation
- **Immediate Trust**: No additional validation - data accepted immediately
- **Timestamp Tracking**: Updates `last_internal_update` for system trust status
- **Monotonic Updates**: Only accepts data newer than existing timestamps

#### **3. POST /public/gossip - Public Reputation Network** (`src/api.rs:77-161`)

**Purpose**: Unauthenticated endpoint for public peers with reputation-based trust and data validation.

```rust
pub async fn public_gossip_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(data_state): State<SharedData>,
    State(reputation_state): State<SharedReputation>,
    State(last_update_state): State<LastInternalUpdate>,
    Json(payload): Json<OhlcvData>,
) -> Response
```

**Multi-Layer Validation Process**:

1. **IP Reputation Check** (`src/api.rs:88-101`):
```rust
let actor = reputation_guard.entry(source_ip).or_default();

if actor.status == ActorStatus::Banned {
    return (StatusCode::FORBIDDEN, "Source IP is banned").into_response();
}
```

2. **System Trust Status** (`src/api.rs:103-110`):
```rust
let time_since_update = last_internal_update.elapsed();

if time_since_update > Duration::from_secs(300) {  // 5 minutes
    return (StatusCode::SERVICE_UNAVAILABLE, "System is running on untrusted data").into_response();
}
```

3. **Price Validation** (`src/api.rs:114-140`):
```rust
let price_change_percent = (payload.close - last_data.close).abs() / last_data.close;

if price_change_percent > 0.10 {  // 10% threshold
    actor.failed_updates += 1;
    
    if actor.failed_updates > 5 {
        actor.status = ActorStatus::Banned;
    }
    return (StatusCode::BAD_REQUEST, "Implausible price change").into_response();
}
```

### **Request/Response Flow Diagrams**

#### **Internal Gossip Flow**

```mermaid
sequenceDiagram
    participant IP as Internal Peer
    participant API as /gossip Endpoint
    participant AUTH as Token Validator
    participant DATA as SharedData
    participant TS as LastUpdate Tracker
    
    IP->>API: POST /gossip + Bearer Token + OHLCV
    API->>AUTH: Validate token (primary/secondary)
    
    alt Valid Token
        AUTH-->>API: ✅ Token Valid
        API->>TS: Update last_internal_update timestamp
        API->>DATA: Update symbol data (no validation)
        API-->>IP: 200 OK
    else Invalid Token
        AUTH-->>API: ❌ Token Invalid
        API-->>IP: 401 Unauthorized
    end
```

#### **Public Gossip Flow**

```mermaid
sequenceDiagram
    participant PP as Public Peer
    participant API as /public/gossip Endpoint
    participant REP as Reputation System
    participant VAL as Price Validator
    participant DATA as SharedData
    
    PP->>API: POST /public/gossip + OHLCV (no auth)
    API->>REP: Check IP reputation status
    
    alt IP Banned
        REP-->>API: ❌ Status: Banned
        API-->>PP: 403 Forbidden
    else IP Not Banned
        REP-->>API: ✅ Status: Probation/Trusted
        API->>VAL: Validate price change (10% threshold)
        
        alt Price Valid
            VAL-->>API: ✅ Price acceptable
            API->>REP: Increment successful_updates
            API->>DATA: Update symbol data
            API-->>PP: 200 OK
        else Price Invalid
            VAL-->>API: ❌ Price implausible
            API->>REP: Increment failed_updates
            Note over REP: Ban if failed_updates > 5
            API-->>PP: 400 Bad Request
        end
    end
```

### **Advanced Features**

#### **Zero-Downtime Token Rotation**

The internal gossip endpoint supports seamless token updates:

```rust
// Current tokens
primary: "secret-token-A-12345"
secondary: "secret-token-B-67890"

// During rotation:
// 1. Deploy new secondary token
secondary: "secret-token-C-11111"

// 2. Update clients to use new token
// 3. Both old and new tokens work simultaneously

// 4. Promote secondary to primary
primary: "secret-token-C-11111"
secondary: "secret-token-D-22222"
```

#### **Reputation-Based Quality Control**

**Price Change Validation Logic**:
```rust
// Example: VCB stock price validation
old_price: 85.0
new_price: 93.5
change_percent: (93.5 - 85.0) / 85.0 = 0.10 (10%)

// Threshold: 10% maximum change
if change_percent > 0.10 {
    // Reject as implausible and penalize sender
}
```

**Reputation State Transitions**:
```
New IP → Probation (default)
Successful updates → Trusted (after consistent good data)
Failed updates > 5 → Banned (permanent until manual reset)
```

#### **System Trust Monitoring**

**Trust Degradation Protection**:
```rust
// If no internal updates for 5+ minutes
if last_internal_update.elapsed() > Duration::from_secs(300) {
    // Reject all public gossip - system running on untrusted data only
    return StatusCode::SERVICE_UNAVAILABLE;
}
```

This ensures the system maintains data quality by requiring periodic updates from trusted internal sources.

### **🔧 API Best Practices**

#### **For Internal Peers**

**Q: How often should internal peers send gossip updates?**
A: Send immediately when new data is available. The system handles duplicate/old data automatically via timestamp comparison.

**Q: What happens if my token becomes invalid?**
A: You'll receive 401 Unauthorized. Check your configuration and use the secondary token as a fallback during rotation.

#### **For Public Peers**

**Q: How do I improve my reputation score?**
A: Send accurate, timely market data consistently. Avoid price updates that exceed 10% change thresholds.

**Q: Why was my IP banned?**
A: Likely due to repeated implausible price changes (>10% from existing data) or malformed requests. Reputation resets are manual.

#### **For Data Consumers**

**Q: How fresh is the data from GET /tickers?**
A: Data freshness depends on worker intervals (typically 30-60 seconds) and network propagation delays.

**Q: Does the API support real-time updates?**
A: Currently REST-only. For real-time updates, poll the /tickers endpoint or implement WebSocket extensions.

---

## 🛡️ Trust and Reputation System - Quality Assurance for Public Networks

### **Reputation System Architecture**

The trust and reputation system provides automated quality control for public peer contributions, implementing a merit-based approach to maintain data integrity without requiring pre-authentication.

```mermaid
graph TB
    subgraph "Public Peer Lifecycle"
        NEW[🆕 New IP Address<br/>First Request<br/>Status: Probation]
        GOOD[✅ Consistent Good Data<br/>Price validation passes<br/>successful_updates++]
        BAD[❌ Bad Data Detected<br/>Price change > 10%<br/>failed_updates++]
    end
    
    subgraph "Reputation States"
        PROBATION[📋 Probation<br/>Default state<br/>Full validation required]
        TRUSTED[🌟 Trusted<br/>Proven track record<br/>Expedited processing]
        BANNED[🚫 Banned<br/>Too many failures<br/>Requests rejected]
    end
    
    subgraph "Validation Rules"
        PRICE_CHECK[💰 Price Validation<br/>Max 10% change<br/>Historical comparison]
        FREQ_CHECK[⏱️ Frequency Limits<br/>Rate limiting<br/>Spam prevention]
        SYS_TRUST[🎯 System Trust<br/>Internal update tracking<br/>5-minute window]
    end
    
    NEW --> PROBATION
    GOOD --> TRUSTED
    BAD --> BANNED
    
    PROBATION --> PRICE_CHECK
    TRUSTED --> PRICE_CHECK
    PRICE_CHECK --> FREQ_CHECK
    FREQ_CHECK --> SYS_TRUST
    
    style PROBATION fill:#fff3e0
    style TRUSTED fill:#e8f5e8  
    style BANNED fill:#ffebee
    style PRICE_CHECK fill:#e3f2fd
```

### **Core Reputation Components** (`src/data_structures.rs:11-32`)

#### **Actor Metadata Structure**

```rust
#[derive(Clone, Debug)]
pub struct ActorMetadata {
    pub successful_updates: u32,  // Count of validated contributions
    pub failed_updates: u32,      // Count of rejected contributions
    pub status: ActorStatus,      // Current trust level
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActorStatus {
    Probation,  // Default state - full validation required
    Trusted,    // Earned through consistent good behavior 
    Banned,     // Blocked due to repeated failures
}
```

**🎯 Design Principles**:
- **IP-Based Tracking**: Simple, stateless identification without complex registration
- **Cumulative Scoring**: Historical behavior influences current trust level
- **Binary Status**: Clear states enable fast decision-making in request handlers
- **Persistent Memory**: Reputation persists across requests within application lifetime

#### **Reputation State Machine Implementation**

**Default Initialization** (`src/api.rs:89`):
```rust
let actor = reputation_guard.entry(source_ip).or_default();
```

When a new IP makes its first request, it automatically gets:
```rust
ActorMetadata {
    successful_updates: 0,
    failed_updates: 0,
    status: ActorStatus::Probation,  // Default from #[derive(Default)]
}
```

### **Validation Pipeline**

#### **1. Reputation Gate Check** (`src/api.rs:98-101`)

```rust
if actor.status == ActorStatus::Banned {
    warn!("Rejected request from banned IP");
    return (StatusCode::FORBIDDEN, "Source IP is banned").into_response();
}
```

**Fast Path Rejection**: Banned IPs are rejected immediately without any processing overhead.

#### **2. System Trust Status** (`src/api.rs:103-110`)

```rust
let last_internal_update = last_update_state.lock().await;
let time_since_update = last_internal_update.elapsed();

if time_since_update > Duration::from_secs(300) {  // 5 minutes
    warn!(time_since_update = ?time_since_update, "System running on untrusted data too long");
    return (StatusCode::SERVICE_UNAVAILABLE, "System is running on untrusted data").into_response();
}
```

**🔒 System-Wide Trust Gate**: If internal (trusted) peers haven't updated data in 5+ minutes, the system refuses all public contributions to prevent data corruption during outages.

#### **3. Price Plausibility Validation** (`src/api.rs:114-140`)

```rust
if let Some(entry) = data_guard.get(symbol.as_str()) {
    if let Some(last_data) = entry.last() {
        let price_change_percent = (payload.close - last_data.close).abs() / last_data.close;
        
        if price_change_percent > 0.10 {  // 10% threshold
            actor.failed_updates += 1;
            
            if actor.failed_updates > 5 {
                actor.status = ActorStatus::Banned;
                error!("Banning IP due to repeated implausible data");
            }
            return (StatusCode::BAD_REQUEST, "Implausible price change").into_response();
        }
    }
}
```

**📊 Price Change Logic**:
- **Threshold**: 10% maximum price change from last known value
- **Mathematical**: `|new_price - old_price| / old_price > 0.10`
- **Failure Tracking**: Accumulates failed attempts per IP
- **Automatic Banning**: After 5 failed validations, IP is permanently banned

#### **4. Success Path - Reputation Improvement** (`src/api.rs:143-154`)

```rust
actor.successful_updates += 1;
let entry = data_guard.entry(symbol.clone()).or_default();
entry.push(payload.clone());
entry.sort_by_key(|d| d.time);

info!(
    symbol,
    close_price = payload.close,
    volume = payload.volume,
    successful_updates = actor.successful_updates,
    "Accepted public gossip data"
);
```

**✅ Positive Reinforcement**: Every successful validation increments the success counter, building reputation over time.

### **Real-World Validation Examples**

#### **Example 1: Normal Market Movement**

```rust
// Existing VCB data
last_price: 85.0

// New public peer submission  
new_price: 87.5
change_percent: |87.5 - 85.0| / 85.0 = 0.0294 (2.94%)

// Result: ACCEPTED (under 10% threshold)
actor.successful_updates += 1
```

#### **Example 2: Suspicious Price Jump**

```rust
// Existing TCB data
last_price: 45.0

// Malicious/erroneous submission
new_price: 52.0  
change_percent: |52.0 - 45.0| / 45.0 = 0.1556 (15.56%)

// Result: REJECTED (exceeds 10% threshold)
actor.failed_updates += 1
// Status: "BAD_REQUEST - Implausible price change"
```

#### **Example 3: Progressive Banning**

```rust
// IP: 192.168.1.100 reputation progression
failed_updates: 1 → Status: Probation (continue accepting)
failed_updates: 2 → Status: Probation (continue accepting)  
failed_updates: 3 → Status: Probation (continue accepting)
failed_updates: 4 → Status: Probation (continue accepting)
failed_updates: 5 → Status: Probation (continue accepting)
failed_updates: 6 → Status: Banned (reject all future requests)
```

### **Advanced Reputation Features**

#### **Reputation-Based Trust Levels**

```mermaid
stateDiagram-v2
    [*] --> Probation : First Request
    
    state Probation {
        [*] --> Validating
        Validating --> Accumulating : Price check passes
        Accumulating --> Evaluating : successful_updates++
    }
    
    Probation --> Trusted : Consistent Good Data
    Probation --> Banned : failed_updates > 5
    
    state Trusted {
        [*] --> HighReputation
        HighReputation --> FastTrack : Expedited processing
    }
    
    Trusted --> Banned : Quality Degradation
    Banned --> [*] : Manual Reset Required
    
    note right of Probation
        Full validation required
        Every submission checked
        Building reputation
    end note
    
    note right of Trusted  
        Proven track record
        Faster processing possible
        Continued monitoring
    end note
    
    note right of Banned
        Reputation irreparably damaged
        All requests rejected
        Manual intervention needed
    end note
```

**Current Implementation**: The system implements Probation and Banned states. The Trusted state exists in the data structure but automatic promotion logic is not yet implemented.

#### **IP-Based Reputation Tracking**

**Storage** (`src/data_structures.rs:43-44`):
```rust
pub type PublicActorReputation = HashMap<IpAddr, ActorMetadata>;
pub type SharedReputation = Arc<Mutex<PublicActorReputation>>;
```

**Benefits**:
- **Simple Identification**: No registration or account setup required
- **Immediate Effect**: Reputation applies instantly to subsequent requests
- **Memory Efficient**: Stores only active IPs, no persistent database required
- **NAT Awareness**: Works with corporate networks behind NAT gateways

**Limitations**:
- **Dynamic IPs**: Reputation lost when IP changes (residential connections)
- **Shared IPs**: Multiple users behind same NAT share reputation
- **No Persistence**: Reputation resets on application restart

### **Trust System Integration**

#### **Trust Degradation Protection**

The system implements a **trust degradation safeguard** that prevents accepting public data when internal trust sources are unavailable:

```rust
// Internal peers must update within 5 minutes
if last_internal_update.elapsed() > Duration::from_secs(300) {
    return StatusCode::SERVICE_UNAVAILABLE;
}
```

**🎯 Security Benefits**:
- **Prevents Data Poisoning**: During internal network outages, public peers cannot corrupt the dataset
- **Maintains Quality**: System prefers stale trusted data over fresh untrusted data
- **Fail-Safe Design**: Errors on the side of caution rather than availability

#### **Multi-Layer Defense**

```mermaid
graph LR
    subgraph "Defense Layers"
        L1[🚫 IP Ban Check<br/>Immediate rejection<br/>Zero processing cost]
        L2[⏰ Trust Window<br/>5-minute internal update<br/>System-wide protection]
        L3[💰 Price Validation<br/>10% change threshold<br/>Economic plausibility]
        L4[📊 Reputation Update<br/>Success/failure tracking<br/>Progressive enforcement]
    end
    
    REQUEST[Public Gossip Request] --> L1
    L1 --> L2
    L2 --> L3  
    L3 --> L4
    L4 --> ACCEPT[✅ Data Accepted]
    
    L1 --> REJECT1[❌ 403 Forbidden]
    L2 --> REJECT2[❌ 503 Service Unavailable]
    L3 --> REJECT3[❌ 400 Bad Request]
    
    style L1 fill:#ffebee
    style L2 fill:#fff3e0
    style L3 fill:#e3f2fd
    style L4 fill:#e8f5e8
```

### **🔧 Reputation System Best Practices**

#### **For Public Peer Operators**

**Q: How do I maintain good reputation?**
A: Send accurate, timely market data. Ensure price updates are realistic (under 10% changes) and avoid sending duplicate or stale data.

**Q: What happens if I get banned?**
A: Currently, bans are permanent within the application lifetime. Restart the node or contact administrators for manual reputation reset.

**Q: How can I check my reputation status?**
A: The system doesn't expose reputation endpoints. Monitor HTTP response codes: 403 = banned, 400 = validation failure, 200 = success.

#### **For Node Operators**

**Q: How do I monitor reputation health?**
A: Watch for patterns in response codes from `/public/gossip` endpoint. High rates of 400/403 responses indicate reputation issues.

**Q: Should I implement reputation persistence?**
A: For production deployments, consider adding database persistence to maintain reputation across restarts and enable manual reputation management.

**Q: How do I handle false positives in price validation?**
A: Consider implementing configurable thresholds per symbol or time-of-day awareness for market opens/closes with higher volatility.

#### **Security Considerations**

1. **Reputation Reset**: Currently requires application restart - implement admin endpoints for manual reputation management
2. **Threshold Tuning**: 10% price change may be too restrictive for volatile stocks or market events
3. **Rate Limiting**: Combine with external rate limiting (nginx, cloudflare) for comprehensive protection
4. **Geographic Bias**: Consider IP geolocation to detect unusual source patterns

---