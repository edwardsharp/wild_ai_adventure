# HTTP Access Logging in Axum

This document explains how to implement HTTP access logging in your Axum application using standard formats like Common Log Format (CLF) and Combined Log Format.

## Overview

We provide multiple approaches for access logging:

1. **Custom Middleware** - Full control over log format and output
2. **tower-http TraceLayer** - Built-in tracing integration
3. **Configuration-based** - Easy setup via config files

## Quick Start

### 1. Enable Access Logging in Configuration

Add the following to your `config.jsonc`:

```jsonc
{
  "logging": {
    "access_log": {
      "enabled": true,
      "file_path": "logs/access.log",
      "format": "combined",
      "also_log_to_tracing": false
    }
  }
}
```

### 2. Standard Log Formats

#### Common Log Format (CLF)
```
192.168.1.1 - - [10/Oct/2000:13:55:36 +0000] "GET /index.html HTTP/1.1" 200 1234
```

#### Combined Log Format
```
192.168.1.1 - - [10/Oct/2000:13:55:36 +0000] "GET /index.html HTTP/1.1" 200 1234 "https://example.com" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
```

#### Custom Format
```jsonc
{
  "logging": {
    "access_log": {
      "enabled": true,
      "file_path": "logs/access.log",
      "format": "custom",
      "custom_template": "{remote_addr} - - [{timestamp}] \"{method} {path} {version}\" {status} {size} \"{referer}\" \"{user_agent}\" {duration}ms"
    }
  }
}
```

## Implementation Approaches

### Approach 1: Custom Middleware (Recommended)

This approach gives you full control over the log format and is already integrated into your application.

```rust
use server::logging::{AccessLogConfig, AccessLogFormat, AccessLogger};

// In your main.rs, the middleware is automatically set up based on config
let access_logger = if let Some(access_config) = &config.logging.access_log {
    if access_config.enabled {
        let format = match access_config.format.as_str() {
            "common" => AccessLogFormat::CommonLog,
            "combined" => AccessLogFormat::CombinedLog,
            "custom" => AccessLogFormat::Custom(template),
            _ => AccessLogFormat::CombinedLog,
        };

        Some(AccessLogger::new(AccessLogConfig {
            file_path: access_config.file_path.clone(),
            format,
            also_log_to_tracing: access_config.also_log_to_tracing,
        }))
    } else {
        None
    }
} else {
    None
};
```

### Approach 2: tower-http TraceLayer

For a simpler setup using tower-http's built-in tracing:

```rust
use server::logging::access_log_trace_layer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Set up file appender for access logs
let file_appender = tracing_appender::rolling::daily("logs", "access.log");

// Configure tracing subscriber with access log layer
tracing_subscriber::registry()
    .with(
        tracing_subscriber::fmt::layer()
            .with_writer(file_appender)
            .with_ansi(false)
            .with_target(false)
            .without_time()
            .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
                metadata.target() == "access_log"
            }))
    )
    .with(
        tracing_subscriber::fmt::layer()
            .with_filter(tracing_subscriber::EnvFilter::from_default_env())
    )
    .init();

// Add to your router
let app = Router::new()
    .route("/", get(handler))
    .layer(access_log_trace_layer());
```

## Configuration Options

### Complete Access Log Configuration

```jsonc
{
  "logging": {
    "level": "info",
    "format": "pretty",
    "file": {
      "path": "logs/app.log",
      "rotate": true,
      "max_size_mb": 100,
      "keep_files": 10
    },
    "access_log": {
      // Enable/disable access logging
      "enabled": true,

      // File path for access logs
      "file_path": "logs/access.log",

      // Format options: "common", "combined", "custom"
      "format": "combined",

      // Custom template (only used when format = "custom")
      "custom_template": "{remote_addr} - - [{timestamp}] \"{method} {path} {version}\" {status} {size} \"{referer}\" \"{user_agent}\" {duration}ms",

      // Also send to application logger
      "also_log_to_tracing": false
    }
  }
}
```

### Custom Template Variables

When using `"format": "custom"`, you can use these placeholders:

- `{remote_addr}` - Client IP address
- `{timestamp}` - Request timestamp in CLF format
- `{method}` - HTTP method (GET, POST, etc.)
- `{path}` - Request path
- `{version}` - HTTP version
- `{status}` - Response status code
- `{size}` - Response size in bytes (or "-" if unknown)
- `{referer}` - Referer header (or "-" if not present)
- `{user_agent}` - User-Agent header (or "-" if not present)
- `{duration}` - Request duration in milliseconds

## Log Rotation

### Automatic Rotation

The application supports automatic log rotation. Configure in your main logging section:

```jsonc
{
  "logging": {
    "file": {
      "path": "logs/app.log",
      "rotate": true,
      "max_size_mb": 100,
      "keep_files": 10
    }
  }
}
```

### External Log Rotation

For production environments, consider using external tools like `logrotate`:

```bash
# /etc/logrotate.d/webauthn-app
/path/to/your/app/logs/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 644 appuser appgroup
    postrotate
        # Send SIGUSR1 to reload logs (if your app supports it)
        # pkill -SIGUSR1 your-app-name
    endscript
}
```

## Remote Address Detection

The middleware automatically detects client IP addresses from common proxy headers:

1. `X-Forwarded-For` (takes first IP in chain)
2. `X-Real-IP`
3. `Forwarded` (RFC 7239 format)

### Reverse Proxy Configuration

#### Nginx
```nginx
location / {
    proxy_pass http://backend;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header Host $host;
}
```

#### Apache
```apache
ProxyPass / http://backend/
ProxyPassReverse / http://backend/
ProxyPreserveHost On
ProxyAddHeaders On
```

#### Cloudflare
Cloudflare automatically adds headers like `CF-Connecting-IP`. You may want to prioritize this:

```rust
// In your custom middleware, check for CF-Connecting-IP first
if let Some(cf_ip) = headers.get("cf-connecting-ip") {
    return Some(cf_ip.to_str().unwrap_or("unknown").to_string());
}
```

## Performance Considerations

### Async Logging

The middleware writes logs asynchronously to avoid blocking request processing:

```rust
// Spawn task to write log without blocking response
let logger_clone = logger.clone();
tokio::spawn(async move {
    logger_clone.write_log(log_entry).await;
});
```

### File I/O Optimization

- Logs are buffered and flushed periodically
- File handles are shared across requests using `Arc<Mutex<File>>`
- Failed writes are logged to the application logger without crashing

### Memory Usage

- Log entries are formatted just-in-time
- No in-memory buffering of log entries
- Minimal memory overhead per request

## Integration with Analytics

Access logs complement your existing analytics system:

```rust
// Both systems can run in parallel
if config.features.analytics_enabled {
    app = app
        .layer(axum_middleware::from_fn(analytics_middleware))
        .layer(Extension(analytics_service));
}

if let Some(logger) = access_logger {
    app = app.layer(axum_middleware::from_fn(
        access_log_middleware_with_logger(logger)
    ));
}
```

### Key Differences

| Feature | Access Logs | Analytics |
|---------|-------------|-----------|
| **Purpose** | Standard HTTP logging | Business intelligence |
| **Format** | CLF/Combined Log Format | Structured data |
| **Storage** | Text files | Database |
| **Processing** | Text processing tools | SQL queries |
| **Retention** | External rotation | Configurable in app |
| **Real-time** | Immediate | Async processing |

## Monitoring and Alerting

### Log Analysis Tools

#### Traditional Tools
```bash
# Most active IPs
awk '{print $1}' access.log | sort | uniq -c | sort -nr | head -10

# Response codes
awk '{print $9}' access.log | sort | uniq -c | sort -nr

# Most requested paths
awk '{print $7}' access.log | sort | uniq -c | sort -nr | head -10
```

#### Modern Tools
- **GoAccess** - Real-time web log analyzer
- **ELK Stack** - Elasticsearch, Logstash, Kibana
- **Grafana Loki** - Log aggregation system
- **Vector** - High-performance log processor

### Alerting Examples

#### GoAccess Real-time Dashboard
```bash
goaccess access.log -o report.html --log-format=COMBINED --real-time-html
```

#### Prometheus Metrics from Logs
Use tools like `prometheus-log-exporter` to convert access logs to metrics.

## Troubleshooting

### Common Issues

#### Empty Remote Address
```
unknown - - [10/Oct/2000:13:55:36 +0000] "GET / HTTP/1.1" 200 1234
```

**Solution**: Configure your reverse proxy to send proper headers.

#### Missing Logs
1. Check file permissions: `ls -la logs/`
2. Verify directory exists: `mkdir -p logs`
3. Check disk space: `df -h`
4. Review application logs for errors

#### Large Log Files
```bash
# Check log file sizes
du -h logs/

# Compress old logs
gzip logs/access.log.1

# Set up log rotation
```

### Debug Mode

Enable debug logging to troubleshoot access log issues:

```jsonc
{
  "logging": {
    "level": "debug",
    "access_log": {
      "enabled": true,
      "also_log_to_tracing": true
    }
  }
}
```

## Production Recommendations

### Security
- Ensure log files are not publicly accessible
- Consider log sanitization for sensitive data
- Use appropriate file permissions (644 or 640)

### Performance
- Use external log rotation
- Monitor disk usage
- Consider log sampling for high-traffic sites

### Compliance
- Access logs may contain personal data (IP addresses)
- Implement retention policies per GDPR/CCPA requirements
- Consider anonymizing IP addresses

```rust
// Example: Anonymize last octet of IPv4 addresses
fn anonymize_ip(ip: &str) -> String {
    if let Some(last_dot) = ip.rfind('.') {
        format!("{}.0", &ip[..last_dot])
    } else {
        ip.to_string()
    }
}
```

## Examples

### Complete Setup Example

```rust
use axum::{routing::get, Router};
use server::logging::{AccessLogConfig, AccessLogFormat, AccessLogger};

#[tokio::main]
async fn main() {
    // Load config
    let config = AppConfig::from_file("config.jsonc").unwrap();

    // Set up access logger
    let access_logger = if config.logging.access_log.enabled {
        Some(AccessLogger::new(AccessLogConfig {
            file_path: config.logging.access_log.file_path,
            format: AccessLogFormat::CombinedLog,
            also_log_to_tracing: false,
        }).unwrap())
    } else {
        None
    };

    // Build router
    let mut app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // Add access logging middleware
    if let Some(logger) = access_logger {
        app = app.layer(axum::middleware::from_fn(
            server::logging::access_log_middleware_with_logger(logger)
        ));
    }

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

This documentation covers everything you need to implement comprehensive access logging in your Axum application. The system is designed to be performant, configurable, and compatible with standard log analysis tools.
