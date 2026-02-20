using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.NetworkInformation;
using System.Text;
using System.Text.Json;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Threading;
using OxyPlot;
using OxyPlot.Annotations;
using OxyPlot.Axes;
using OxyPlot.Series;

namespace PingDVD;

public partial class MainWindow : Window
{
    private readonly List<long> _values = new();
    private bool _running;
    private readonly PlotModel _plotModel;
    private readonly LineSeries _series;
    private readonly LineAnnotation _avgLine;
    private readonly string _settingsPath;

    private const long PingFailureValue = -1;

    public MainWindow()
    {
        InitializeComponent();

        _settingsPath = Path.Combine(AppContext.BaseDirectory, "pingdvd.settings.json");

        _plotModel = BuildPlotModel(out _series, out _avgLine);
        PlotMain.Model = _plotModel;

        LoadSettings();
        InitializeValues();
        UpdatePlot();
    }

    private static PlotModel BuildPlotModel(out LineSeries series, out LineAnnotation avgLine)
    {
        var model = new PlotModel
        {
            Background = OxyColor.FromRgb(30, 30, 30),
            PlotAreaBackground = OxyColor.FromRgb(30, 30, 30),
            TextColor = OxyColor.FromRgb(230, 230, 230),
            PlotAreaBorderColor = OxyColor.FromRgb(100, 100, 100),
        };

        model.Axes.Add(new LinearAxis
        {
            Position = AxisPosition.Left,
            Minimum = 0,
            Maximum = AppSettings.DefaultTimeOut,
            AxislineColor = OxyColor.FromRgb(230, 230, 230),
            TicklineColor = OxyColor.FromRgb(230, 230, 230),
            TextColor = OxyColor.FromRgb(230, 230, 230),
            MajorGridlineStyle = LineStyle.Solid,
            MajorGridlineColor = OxyColor.FromArgb(50, 230, 230, 230),
            IsZoomEnabled = false,
            IsPanEnabled = false,
        });

        model.Axes.Add(new LinearAxis
        {
            Position = AxisPosition.Bottom,
            IsAxisVisible = false,
            IsZoomEnabled = false,
            IsPanEnabled = false,
        });

        series = new LineSeries
        {
            Color = OxyColor.FromRgb(36, 92, 179),
            StrokeThickness = 3,
            MarkerType = MarkerType.None,
        };
        model.Series.Add(series);

        avgLine = new LineAnnotation
        {
            Type = LineAnnotationType.Horizontal,
            Y = 0,
            Color = OxyColor.FromArgb(200, 252, 62, 54),
            StrokeThickness = 2,
            LineStyle = LineStyle.Solid,
        };
        model.Annotations.Add(avgLine);

        return model;
    }

    private void InitializeValues()
    {
        // Pre-populate chart with a descending "idle" baseline clamped at 9 ms,
        // so the chart looks non-empty before the first real ping run.
        long lv = 11;
        for (int i = 0; i < 500; i++)
        {
            _values.Add(Math.Max(lv, 9));
            lv--;
        }
    }

    private void ButtonStartStop_Click(object? sender, RoutedEventArgs e)
    {
        _running = !_running;

        if (_running)
        {
            var timeout = (double)(NumericTimeOut.Value ?? (decimal)AppSettings.DefaultTimeOut);
            var yAxis = _plotModel.Axes.FirstOrDefault(a => a.Position == AxisPosition.Left);
            if (yAxis != null)
                yAxis.Maximum = timeout;

            _ = RunPingLoopAsync();
        }
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
            return PingFailureValue;
        }
    }

    private void UpdatePlot()
    {
        _series.Points.Clear();
        for (int i = 0; i < _values.Count; i++)
            _series.Points.Add(new DataPoint(i, _values[i]));

        double avg = _values.Average();
        _avgLine.Y = avg;

        double interval = (double)(NumericInterval.Value ?? (decimal)AppSettings.DefaultInterval);
        var elapsed = TimeSpan.FromMilliseconds(interval * _values.Count);
        Title = $"PingDVD - AVG: {Math.Round(avg, 2)} msec - LAST: {_values.Last()} msec - " +
                $"MIN: {_values.Min()} msec - MAX: {_values.Max()} msec - " +
                $"{elapsed:hh\\:mm\\:ss}";

        _plotModel.InvalidatePlot(true);
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
