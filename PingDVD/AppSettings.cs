namespace PingDVD;

public class AppSettings
{
    public const string DefaultHost = "www.google.it";
    public const double DefaultInterval = 500;
    public const double DefaultTimeOut = 500;

    public string Host { get; set; } = DefaultHost;
    public double Interval { get; set; } = DefaultInterval;
    public double TimeOut { get; set; } = DefaultTimeOut;
}
