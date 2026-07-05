# PingDVD — agent instructions

## Commands
```bash
dotnet build
dotnet run
dotnet build --configuration Release --no-restore
dotnet publish PingDVD/PingDVD.csproj -c Release -r win-x64 --self-contained false
dotnet publish PingDVD/PingDVD.csproj -c Release -r osx-x64 --self-contained false
```

## Branch
Default branch is `Avalonia_dotnet_8` (not `main`). CI triggers on pushes/PRs to that branch and on `v*` tags.

## Stack
- .NET 8.0, Avalonia 11.2.3 (Fluent Dark theme), single-project (no solution multi-target)
- Entry: `PingDVD/Program.cs` → `MainWindow.axaml.cs`
- Code-behind UI logic, no strict MVVM framework

## Settings
Persisted as `pingdvd.settings.json` JSON in `AppContext.BaseDirectory` (next to the binary).

## CI
Builds + publishes for `windows-latest` and `macos-latest` only. No Linux build. Auto-creates a release with timestamp tag on push to `Avalonia_dotnet_8`.

## Quirks
- Runtime identifiers: `win-x64;osx-arm64;osx-x64`
- macOS builds stamp a custom icon via sips/DeRez/Rez (see csproj)
- No tests, no linter, no formatter config in the repo
