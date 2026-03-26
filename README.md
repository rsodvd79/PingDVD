# PingDVD
Ping GUI with chart

A real-time network latency monitoring application built with Avalonia .NET that pings a specified host and displays response times in an interactive chart.

## Features

- **Real-time Ping Monitoring**: Continuously ping a host and visualize latency over time
- **Interactive Chart**: DeepSkyBlue polyline showing ping times with OrangeRed average line
- **Status Indicators**: 
  - LED indicator (Red/Green/Orange) showing current state
  - Status text displaying "Stopped", "Running", or error messages
  - Positioned conveniently below the chart
- **Enhanced Error Handling**: 
  - Distinguishes between host unreachable, access denied, and other ping errors
  - Graceful fallback to timeout values for continued visualization
- **User Controls**:
  - Host input (hostname or IP address)
  - Configurable timeout (1-100000 ms)
  - Configurable interval (1-100000 ms)
  - Start/Stop button to control ping operations
  - Apply button to save settings immediately
  - Reset button to restore default values
- **Settings Persistence**: 
  - Automatic saving of settings on application exit
  - Manual saving via Apply button
  - Loading of saved settings on startup
- **Chart Improvements**:
  - Auto-scaling with minimum Y-range (10ms) for readability when values are similar
  - Configurable history size (default 500 samples)
  - Smooth animation and updating

## Technical Details

- **Framework**: .NET 8.0 with Avalonia UI
- **Cross-platform**: Primarily tested on macOS, but designed for cross-platform use
- **Networking**: Uses System.Net.NetworkInformation.Ping class
- **UI**: MVVM-inspired pattern with code-behind for UI logic
- **Persistence**: JSON-based settings storage

## Default Settings

- **Host**: www.google.it
- **Interval**: 500 ms
- **Timeout**: 500 ms
- **History Size**: 500 samples
- **Initial Chart**: Pre-populated with 200 sample points for immediate visualization

## Recent Improvements

1. **Enhanced Error Handling**: Differentiates between various ping failure modes
2. **Better UI Feedback**: LED status indicator with contextual messages
3. **Immediate Settings Apply**: Apply button saves settings without restart
4. **One-click Reset**: Restore defaults instantly
5. **Input Validation**: Basic hostname/IP validation before ping attempts
6. **Chart Readability**: Minimum Y-axis range prevents unreadable charts
7. **Code Quality**: Magic numbers replaced with named constants, optimized calculations

## Usage

1. Launch the application
2. Optionally modify Host, Timeout, and Interval values
3. Click "Start" to begin pinging
4. Observe real-time latency in the chart
5. Use "Apply" to save current settings
6. Use "Reset" to return to factory defaults
7. Click "Stop" to halt pinging
8. Settings are automatically saved on exit

## Building

```bash
dotnet build
dotnet run
```

## License

MIT License - see LICENSE file for details.