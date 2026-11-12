# P0 issue #49: collect native window/child HWND geometry only.
# Run against an Abiding Word instance using invented/disposable data.
# No source text, window titles, screenshots, file paths or credentials logged.
# Coordinates are subject to caller DPI awareness: execute from a DPI-aware
# PowerShell host and compare the reported DPI with the native application.
[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)][int]$ProcessId,
  [Parameter(Mandatory = $true)][string]$OutputPath
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;

public static class AwWindowMetrics {
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT {
        public int Left, Top, Right, Bottom;
        public int Width => Right - Left;
        public int Height => Bottom - Top;
    }
    public sealed class Record {
        public string ClassName { get; set; } = "";
        public bool Visible { get; set; }
        public int Dpi { get; set; }
        public int ClientWidth { get; set; }
        public int ClientHeight { get; set; }
        public int ClientScreenX { get; set; }
        public int ClientScreenY { get; set; }
        public int ScreenX { get; set; }
        public int ScreenY { get; set; }
        public int ScreenWidth { get; set; }
        public int ScreenHeight { get; set; }
    }
    public sealed class Report {
        public string Platform { get; set; } = "Windows HWND geometry";
        public Record Main { get; set; }
        public List<Record> Descendants { get; set; } = new List<Record>();
    }
    private delegate bool EnumChildProc(IntPtr hwnd, IntPtr lparam);
    [DllImport("user32.dll")] private static extern bool GetClientRect(IntPtr hwnd, out RECT rect);
    [DllImport("user32.dll")] private static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
    [StructLayout(LayoutKind.Sequential)] private struct POINT { public int X, Y; }
    [DllImport("user32.dll")] private static extern bool ClientToScreen(IntPtr hwnd, ref POINT point);
    [DllImport("user32.dll")] private static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    private static extern int GetClassName(IntPtr hwnd, StringBuilder str, int max);
    [DllImport("user32.dll")] private static extern uint GetDpiForWindow(IntPtr hwnd);
    [DllImport("user32.dll")] private static extern bool EnumChildWindows(
        IntPtr hwnd, EnumChildProc callback, IntPtr param);

    private static Record Read(IntPtr hwnd) {
        RECT client, window;
        if (!GetClientRect(hwnd, out client) || !GetWindowRect(hwnd, out window))
            throw new InvalidOperationException("Unable to measure an HWND");
        var origin = new POINT { X = 0, Y = 0 };
        if (!ClientToScreen(hwnd, ref origin))
            throw new InvalidOperationException("Unable to translate client origin");
        var name = new StringBuilder(256);
        GetClassName(hwnd, name, name.Capacity);
        return new Record {
            ClassName = name.ToString(),
            Visible = IsWindowVisible(hwnd),
            Dpi = unchecked((int)GetDpiForWindow(hwnd)),
            ClientWidth = client.Width,
            ClientHeight = client.Height,
            ClientScreenX = origin.X,
            ClientScreenY = origin.Y,
            ScreenX = window.Left,
            ScreenY = window.Top,
            ScreenWidth = window.Width,
            ScreenHeight = window.Height
        };
    }
    public static Report Capture(IntPtr hwnd) {
        if (hwnd == IntPtr.Zero)
            throw new ArgumentException("No main native HWND: an interactive Windows desktop is required");
        var report = new Report { Main = Read(hwnd) };
        EnumChildProc callback = (child, ignored) => {
            // Capture the browser host as well as its child compositor HWNDs.
            // No titles, URLs or text contents are read.
            report.Descendants.Add(Read(child));
            return true;
        };
        if (!EnumChildWindows(hwnd, callback, IntPtr.Zero))
            throw new InvalidOperationException("Could not enumerate child HWNDs");
        return report;
    }
}
'@

$process = Get-Process -Id $ProcessId -ErrorAction Stop
$report = [AwWindowMetrics]::Capture($process.MainWindowHandle)
$report | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $OutputPath -Encoding UTF8
Write-Host "Wrote native window geometry only. No document or user content was captured."
Write-Host "Compare client, child HWND and WebView2 sizes with the native 'Initial main-window geometry' log."
Write-Host "This does NOT prove DOM viewport height: independently measure CSS innerHeight, app-shell and pane rects in a disposable Windows test."
