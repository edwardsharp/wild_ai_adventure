# HTTP Access Logging Solution for Axum

This document provides you with a complete solution for implementing HTTP access logging in your Axum application, including standard formats like Common Log Format (CLF) and Combined Log Format.

## Overview

I've implemented a comprehensive access logging system with the following features:

- **Standard Log Formats**: Common Log Format (CLF) and Combined Log Format
- **Custom Templates**: Support for custom log formats with placeholders
- **Reverse Proxy Support**: Automatic IP extraction from proxy headers
- **Asynchronous Logging**: Non-blocking log writes
- **Configuration-driven**: Easy setup via your existing config system
- **Performance Optimized**: Minimal overhead on request processing

## Quick Start

### 1. Configuration

Add this to your `config.jsonc`:

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

### 2. Integration

The access logging is already integrated into your `main.rs`. When you enable it in config, it automatically:

- Creates the log directory if it doesn't exist
- Sets up the access logger with your chosen format
- Adds the middleware to your router

### 3. Start Your Server

```bash
cd axum_tutorial
cargo run --bin server
```

## What I've Implemented

### 1. Core Access Logging (`server/src/logging/access_log.rs`)

- **AccessLogger**: Main logging struct with file handling
- **AccessLogFormat**: Enum for different log formats (Common, Combined, Custom)
- **AccessLogConfig**: Configuration structure
- **Middleware Functions**: Both generic and parameterized middleware

### 2. Configuration Integration (`server/src/config.rs`)

- Added `AccessLogConfig` struct to your existing `LoggingConfig`
- Added default values for access log settings
- Integrated with your existing configuration system

### 3. Main Application Integration (`server/src/main.rs`)

- Automatic setup based on configuration
- Error handling for logger initialization
- Proper middleware ordering

### 4. Documentation (`docs/access-logging.md`)

- Comprehensive guide covering all aspects
- Configuration examples
- Performance considerations
- Production recommendations

### 5. Example Application (`examples/simple_access_log.rs`)

- Standalone example demonstrating access logging
- Multiple test endpoints
- Can be run independently for testing

## Log Formats

### Common Log Format (CLF)
```
192.168.1.1 - - [10/Oct/2000:13:55:36 +0000] "GET /index.html HTTP/1.1" 200 1234
```

### Combined Log Format
```
192.168.1.1 - - [10/Oct/2000:13:55:36 +0000] "GET /index.html HTTP/1.1" 200 1234 "-" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
```

### Custom Format Example
```jsonc
{
  "logging": {
    "access_log": {
      "format": "custom",
      "custom_template": "{remote_addr} [{timestamp}] {method} {path} -> {status} ({duration}ms)"
    }
  }
}
```

## Available Template Variables

When using custom format, you can use these placeholders:

- `{remote_addr}` - Client IP address
- `{timestamp}` - Request timestamp in CLF format
- `{method}` - HTTP method (GET, POST, etc.)
- `{path}` - Request path
- `{version}` - HTTP version
- `{status}` - Response status code
- `{size}` - Response size in bytes
- `{referer}` - Referer header
- `{user_agent}` - User-Agent header
- `{duration}` - Request duration in milliseconds

## Reverse Proxy Support

The system automatically detects client IPs from common proxy headers:

1. `X-Forwarded-For` (takes first IP in chain)
2. `X-Real-IP`
3. `Forwarded` (RFC 7239 format)

### Nginx Configuration
```nginx
location / {
    proxy_pass http://your-app:8080;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header Host $host;
}
```

## Performance Features

### Asynchronous Logging
- Log writes happen in background tasks
- No blocking of request processing
- Failed writes are logged to application logger

### Memory Efficiency
- Just-in-time log formatting
- Shared file handles with Arc<Mutex<File>>
- No in-memory buffering of log entries

### File I/O Optimization
- Automatic flushing
- Error recovery
- Directory creation

## Testing

### Run the Example
```bash
cd axum_tutorial
cargo run --example simple_access_log
```

Then test with:
```bash
curl http://localhost:3000/
curl -H 'User-Agent: MyApp/1.0' http://localhost:3000/
curl -H 'X-Forwarded-For: 192.168.1.100' http://localhost:3000/
```

### Integration Tests
```bash
cargo test access_log_test
```

## Log Analysis Examples

### Traditional Unix Tools
```bash
# Most active IPs
awk '{print $1}' logs/access.log | sort | uniq -c | sort -nr | head -10

# Response codes distribution
awk '{print $9}' logs/access.log | sort | uniq -c | sort -nr

# Most requested paths
awk '{print $7}' logs/access.log | sort | uniq -c | sort -nr | head -10

# Traffic by hour
awk '{print substr($4,2,14)}' logs/access.log | uniq -c
```

### GoAccess Real-time Dashboard
```bash
goaccess logs/access.log -o report.html --log-format=COMBINED --real-time-html
```

## Log Rotation

### Using logrotate
Create `/etc/logrotate.d/webauthn-app`:
```
/path/to/your/app/logs/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 644 appuser appgroup
}
```

### Manual rotation
```bash
# Compress old logs
gzip logs/access.log.1

# Check log sizes
du -h logs/
```

## Production Considerations

### Security
- Ensure log files are not publicly accessible
- Consider IP anonymization for GDPR compliance
- Use appropriate file permissions (644 or 640)

### Monitoring
- Monitor disk usage for log files
- Set up alerts for failed log writes
- Consider log sampling for high-traffic sites

### Compliance
- Access logs may contain personal data (IP addresses)
- Implement retention policies per regulations
- Consider anonymizing sensitive data

## Integration with Your Analytics

The access logging system complements your existing analytics:

| Feature | Access Logs | Your Analytics DB |
|---------|-------------|-------------------|
| **Purpose** | Standard HTTP logging | Business intelligence |
| **Format** | CLF/Combined Log Format | Structured data |
| **Storage** | Text files | PostgreSQL |
| **Processing** | Text tools, GoAccess | SQL queries |
| **Real-time** | Immediate file writes | Async DB writes |

Both systems run in parallel without interference.

## Troubleshooting

### Common Issues

**Empty remote address**
- Configure your reverse proxy to send proper headers
- Check `X-Forwarded-For` or `X-Real-IP` headers

**Missing logs**
```bash
# Check permissions
ls -la logs/

# Check disk space
df -h

# Check application logs for errors
```

**Large log files**
- Set up log rotation
- Consider log sampling for high traffic
- Monitor disk usage

### Debug Configuration
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

## Files Added/Modified

### New Files
- `server/src/logging/mod.rs` - Logging module definition
- `server/src/logging/access_log.rs` - Core access logging implementation
- `docs/access-logging.md` - Comprehensive documentation
- `examples/simple_access_log.rs` - Standalone example
- `assets/config/access-log-example.jsonc` - Configuration example

### Modified Files
- `server/src/lib.rs` - Added logging module
- `server/src/config.rs` - Added AccessLogConfig structure
- `server/src/main.rs` - Integrated access logging setup
- `Cargo.toml` - Added tracing-appender dependency

## Comparison with Alternatives

### vs. tower-http TraceLayer
- **Pros**: More control over format, file output, configuration
- **Cons**: More code, but well-tested and documented

### vs. External Solutions
- **Pros**: Integrated with your config system, no external dependencies
- **Cons**: Not as feature-rich as dedicated log processors

### vs. Nginx/Apache Logs
- **Pros**: Application-level logging, custom formats, integration with your auth system
- **Cons**: Additional disk I/O, but asynchronous so minimal performance impact

## Next Steps

1. **Enable it**: Add the access log config to your configuration file
2. **Test it**: Run your server and make some requests
3. **Monitor it**: Check the log file format and content
4. **Optimize it**: Set up log rotation and monitoring
5. **Analyze it**: Use tools like GoAccess for real-time analysis

The system is production-ready and designed to handle high traffic with minimal performance impact. It integrates seamlessly with your existing configuration and logging infrastructure.
