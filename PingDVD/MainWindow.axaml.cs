using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.NetworkInformation;
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
    private readonly List<long> _values = new();
    private bool _running;
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
        for (int i = 0; i < 200; i++)
        {
            var baseValue = 12 + 3 * Math.Sin(i / 8.0);
            var noisy = baseValue + rand.NextDouble() * 2 - 1;
            _values.Add(Math.Max(0, (long)Math.Round(noisy)));
        }
    }

    private void ButtonStartStop_Click(object? sender, RoutedEventArgs e)
    {
        _running = !_running;

        if (_running)
            _ = RunPingLoopAsync();
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
                if (_values.Count > 500)
                    _values.RemoveAt(0);
                UpdatePlot();
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
            return reply.Status == IPStatus.Success ? reply.RoundtripTime : timeout;
        }
        catch
        {
            // When ping is not permitted or host is invalid, show a timeout bar instead of disappearing data.
            return timeout;
        }
    }

    private void UpdatePlot()
    {
        double avg = _values.Average();
        var minVal = _values.Min();
        var maxVal = _values.Max();

        double interval = (double)(NumericInterval.Value ?? (decimal)AppSettings.DefaultInterval);
        var elapsed = TimeSpan.FromMilliseconds(interval * _values.Count);
        Title = $"PingDVD - AVG: {Math.Round(avg, 2)} msec - LAST: {_values.Last()} msec - " +
                $"MIN: {minVal} msec - MAX: {maxVal} msec - " +
                $"{elapsed:hh\\:mm\\:ss}";

        var bounds = PlotCanvas.Bounds;
        if (bounds.Width <= 1 || bounds.Height <= 1)
            return;

        var xScale = bounds.Width / Math.Max(1, _values.Count - 1);
        var yScale = bounds.Height / Math.Max(1, (maxVal - minVal == 0 ? 1 : maxVal - minVal));

        var points = new AvaloniaList<Point>();
        for (int i = 0; i < _values.Count; i++)
        {
            var x = i * xScale;
            var y = bounds.Height - ((_values[i] - minVal) * yScale);
            points.Add(new Point(x, y));
        }
        _polyline.Points = points;

        var avgY = bounds.Height - ((avg - minVal) * yScale);
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
