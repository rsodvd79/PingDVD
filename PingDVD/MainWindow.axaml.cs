using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net;
using System.Net.NetworkInformation;
using System.Net.Sockets;
using System.Text;
using System.Text.Json;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Collections;
using Avalonia.Controls;
using Avalonia.Controls.Shapes;
using Avalonia.Interactivity;
using Avalonia.Media;
using Avalonia.Platform;
using Avalonia.Threading;

namespace PingDVD;

public partial class MainWindow : Window
{
    // Constants for chart initialization
    private const int InitialSampleCount = 200;
    private const double BaseValueOffset = 12.0;
    private const double SinusoidalAmplitude = 3.0;
    private const double SinusoidalFrequency = 8.0;
    private const double NoiseRange = 2.0;
    
    // Constants for chart display
    private const int MaxHistorySize = 500;
    private const double MinYRange = 10.0; // Minimum range for Y axis in milliseconds
    
    private readonly List<long> _values = new();
    private bool _running;
    private int _realPingCount;
    private readonly Polyline _polyline;
    private readonly Line _avgLine;
    private readonly string _settingsPath;

    public MainWindow()
    {
        InitializeComponent();
        Icon = new WindowIcon(AssetLoader.Open(new Uri("avares://PingDVD/Assets/pingdvd.png")));

        _settingsPath = System.IO.Path.Combine(AppContext.BaseDirectory, "pingdvd.settings.json");

        _polyline = new Polyline
        {
            Stroke = Brushes.DeepSkyBlue,
            StrokeThickness = 2,
            Points = new AvaloniaList<Point>(),
        };

        _avgLine = new Line
        {
            Stroke = Brushes.OrangeRed,
            StrokeThickness = 1,
            StrokeDashArray = new AvaloniaList<double> { 4, 4 },
        };

        PlotCanvas.Children.Add(_avgLine);
        PlotCanvas.Children.Add(_polyline);
        PlotCanvas.AttachedToVisualTree += (_, _) => UpdatePlot();

        LoadSettings();
        InitializeValues();
        UpdatePlot();
    }

    private void InitializeValues()
    {
        // Pre-populate chart with a descending "idle" baseline clamped at 9 ms,
        // so the chart looks non-empty before the first real ping run.
        var rand = new Random();
        for (int i = 0; i < InitialSampleCount; i++)
        {
            var baseValue = BaseValueOffset + SinusoidalAmplitude * Math.Sin(i / SinusoidalFrequency);
            var noisy = baseValue + rand.NextDouble() * NoiseRange - 1;
            _values.Add(Math.Max(0, (long)Math.Round(noisy)));
        }
    }

    private void ButtonStartStop_Click(object? sender, RoutedEventArgs e)
    {
        // Validate host before starting
        string host = TextBoxHost.Text?.Trim() ?? AppSettings.DefaultHost;
        if (!IsValidHost(host))
        {
            // Show error - for now just use default and continue
            host = AppSettings.DefaultHost;
            TextBoxHost.Text = host;
        }

        _running = !_running;
        ButtonStartStop.Content = _running ? "■ Stop" : "▶ Start";
        UpdateStatusIndicator(_running);

        if (_running)
        {
            _realPingCount = 0;
            _ = RunPingLoopAsync();
        }
    }

    private void ButtonApply_Click(object? sender, RoutedEventArgs e)
    {
        SaveSettings();
    }

    private void ButtonReset_Click(object? sender, RoutedEventArgs e)
    {
        TextBoxHost.Text = AppSettings.DefaultHost;
        NumericInterval.Value = (decimal)AppSettings.DefaultInterval;
        NumericTimeOut.Value = (decimal)AppSettings.DefaultTimeOut;
        SaveSettings();
    }

    private void UpdateStatusIndicator(bool running, long pingResult = 0)
    {
        if (!running)
        {
            StatusLED.Fill = Brushes.Red;
            StatusText.Text = "Stopped";
            return;
        }

        // Running state
        if (pingResult >= 0)
        {
            // Success or timeout (timeout treated as normal high value)
            StatusLED.Fill = Brushes.Green;
            StatusText.Text = $"Running - Last: {pingResult} ms";
        }
        else
        {
            // Error conditions based on IPStatus mapping
            switch (pingResult)
            {
                case -1:
                    StatusLED.Fill = Brushes.Orange;
                    StatusText.Text = "Error: Host unreachable";
                    break;
                case -2:
                    StatusLED.Fill = Brushes.Orange;
                    StatusText.Text = "Error: Access denied";
                    break;
                default:
                    // Should not happen with current implementation, but handle just in case
                    StatusLED.Fill = Brushes.Orange;
                    StatusText.Text = "Error: Unknown";
                    break;
            }
        }
    }

    private bool IsValidHost(string host)
    {
        if (string.IsNullOrWhiteSpace(host))
            return false;

        // Basic validation for hostname or IP address
        // This is a simple check - in production you might want to use Uri.CheckHostName or similar
        if (host.Length > 255)
            return false;

        // Check if it's an IP address
        if (System.Net.IPAddress.TryParse(host, out _))
            return true;

        // Basic hostname validation (letters, digits, hyphens, dots)
        // Each label between dots should be 1-63 chars, start/end with alphanumeric
        var labels = host.Split('.');
        if (labels.Length < 1)
            return false;

        foreach (var label in labels)
        {
            if (string.IsNullOrEmpty(label) || label.Length > 63)
                return false;

            if (!char.IsLetterOrDigit(label[0]) || !char.IsLetterOrDigit(label[label.Length - 1]))
                return false;

            foreach (char c in label)
            {
                if (!char.IsLetterOrDigit(c) && c != '-')
                    return false;
            }
        }

        return true;
    }

    private async Task RunPingLoopAsync()
    {
        while (_running)
        {
            string host = AppSettings.DefaultHost;
            int timeout = (int)AppSettings.DefaultTimeOut;
            int interval = (int)AppSettings.DefaultInterval;

            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                host = TextBoxHost.Text ?? AppSettings.DefaultHost;
                timeout = (int)(NumericTimeOut.Value ?? (decimal)AppSettings.DefaultTimeOut);
                interval = (int)(NumericInterval.Value ?? (decimal)AppSettings.DefaultInterval);
            });

            long roundtrip = await PingAsync(host, timeout);

            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                _values.Add(roundtrip);
                _realPingCount++;
                if (_values.Count > MaxHistorySize)
                    _values.RemoveAt(0);
                UpdatePlot();
                UpdateStatusIndicator(_running, roundtrip);
            });

            if (_running)
                await Task.Delay(interval);
        }
    }

    private static async Task<long> PingAsync(string host, int timeout)
    {
        try
        {
            using var pingSender = new Ping();
            var options = new PingOptions { DontFragment = true };
            var buffer = Encoding.ASCII.GetBytes(new string('\0', 32));
            var reply = await pingSender.SendPingAsync(host, timeout, buffer, options);
            
            // Map IPStatus to our return values for better error handling
            return reply.Status switch
            {
                IPStatus.Success => reply.RoundtripTime,
                IPStatus.TimedOut => timeout,
                IPStatus.DestinationHostUnreachable => -1,  // Host not found/unreachable
                IPStatus.DestinationNetworkUnreachable => -1, // Network unreachable
                IPStatus.DestinationProhibited => -2,       // Access denied/prohibited
                _ => timeout // For other errors, return timeout (treat as high latency)
            };
        }
        catch
        {
            // When ping is not permitted or host is invalid, show a timeout bar instead of disappearing data.
            return timeout;
        }
    }

    private void UpdatePlot()
    {
        if (_values.Count == 0)
            return;

        double avg = _values.Average();
        var minVal = _values.Min();
        var maxVal = _values.Max();

        double interval = (double)(NumericInterval.Value ?? (decimal)AppSettings.DefaultInterval);
        var elapsed = TimeSpan.FromMilliseconds(interval * _realPingCount);
        Title = $"PingDVD - AVG: {Math.Round(avg, 2)} msec - LAST: {_values.Last()} msec - " +
                $"MIN: {minVal} msec - MAX: {maxVal} msec - " +
                $"{elapsed:hh\\:mm\\:ss}";

        var bounds = PlotCanvas.Bounds;
        if (bounds.Width <= 1 || bounds.Height <= 1)
            return;

        // Ensure a minimum Y range to prevent division by zero and to keep chart readable when values are similar
        const double minRange = 10.0; // milliseconds
        double actualRange = maxVal - minVal;
        double range = Math.Max(actualRange, minRange);
        // If actualRange is zero, we still want to center around the value
        double rangeMin = actualRange < minRange ? (minVal + maxVal) / 2 - minRange / 2 : minVal;

        var xScale = bounds.Width / Math.Max(1, _values.Count - 1);
        var yScale = bounds.Height / range;

        var points = new AvaloniaList<Point>();
        for (int i = 0; i < _values.Count; i++)
        {
            var x = i * xScale;
            var y = bounds.Height - ((_values[i] - rangeMin) * yScale);
            points.Add(new Point(x, y));
        }
        _polyline.Points = points;

        var avgY = bounds.Height - ((avg - rangeMin) * yScale);
        _avgLine.StartPoint = new Point(0, avgY);
        _avgLine.EndPoint = new Point(bounds.Width, avgY);
    }

    private void LoadSettings()
    {
        AppSettings settings = new();
        try
        {
            if (File.Exists(_settingsPath))
            {
                var json = File.ReadAllText(_settingsPath);
                settings = JsonSerializer.Deserialize<AppSettings>(json) ?? new AppSettings();
            }
        }
        catch { }

        TextBoxHost.Text = settings.Host;
        NumericInterval.Value = (decimal)settings.Interval;
        NumericTimeOut.Value = (decimal)settings.TimeOut;
    }

    private void SaveSettings()
    {
        var settings = new AppSettings
        {
            Host = TextBoxHost.Text ?? AppSettings.DefaultHost,
            Interval = (double)(NumericInterval.Value ?? (decimal)AppSettings.DefaultInterval),
            TimeOut = (double)(NumericTimeOut.Value ?? (decimal)AppSettings.DefaultTimeOut),
        };
        try
        {
            var json = JsonSerializer.Serialize(settings);
            File.WriteAllText(_settingsPath, json);
        }
        catch { }
    }

    protected override void OnClosing(WindowClosingEventArgs e)
    {
        _running = false;
        SaveSettings();
        base.OnClosing(e);
    }
}
