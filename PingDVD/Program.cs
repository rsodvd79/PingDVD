using Avalonia;
using System;

namespace PingDVD;

class Program
{
    [STAThread]
    public static void Main(string[] args) =>
        BuildAvaloniaApp().StartWithClassicDesktopLifetime(args);

    public static AppBuilder BuildAvaloniaApp()
        => AppBuilder.Configure<App>()
            .UsePlatformDetect()
            .With(new MacOSPlatformOptions
            {
                ShowInDock = true,
                DisableDefaultApplicationMenuItems = true,
                DisableNativeMenus = true,
            })
            .WithInterFont()
            .LogToTrace();
}
