#requires -Version 7.2
# P0 issue #49: collect native window/child HWND geometry only.
# Run against an Abiding Word instance using invented/disposable data.
# Geometry contains no source text, window titles, URLs or credentials.
# Optional screenshots require a clean desktop and invented content only.
# Capture temporarily establishes per-monitor-v2 thread DPI awareness.
# Compare reported native DPI with the app and browser DPR; do not emulate OS DPI.
[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)][int]$ProcessId,
  [Parameter(Mandatory = $true)][string]$OutputPath,
  [ValidateSet('observe','maximise','restore','resize')][string]$Action = 'observe',
  [int]$Width = 0, [int]$Height = 0,
  [string]$ScreenshotPath
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if ([Environment]::OSVersion.Platform -ne 'Win32NT') {
  throw 'An interactive Windows desktop is required; this script cannot validate Linux Chromium.'
}
Add-Type -AssemblyName System.Drawing

$references = @((Get-ChildItem -LiteralPath (Join-Path $PSHOME 'ref') -Filter '*.dll').FullName) + @([System.Drawing.Bitmap].Assembly.Location) + @((Get-ChildItem -LiteralPath $PSHOME -Filter 'System.Private.Windows.*.dll').FullName)
Add-Type -ReferencedAssemblies $references -TypeDefinition @'
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
        public string Hwnd { get; set; } = "";
        public string ClassName { get; set; } = "";
        public bool Visible { get; set; }
        public bool Maximized { get; set; }
        public bool Minimized { get; set; }
        public int Dpi { get; set; }
        public int ClientWidth { get; set; }
        public int ClientHeight { get; set; }
        public int ClientScreenX { get; set; }
        public int ClientScreenY { get; set; }
        public int ScreenX { get; set; }
        public int ScreenY { get; set; }
        public int ScreenWidth { get; set; }
        public int ScreenHeight { get; set; }
        public RECT Monitor { get; set; }
    }
    public sealed class Report {
        public string Platform { get; set; } = "Windows HWND geometry";
        public Record Main { get; set; }
        public List<Record> Descendants { get; set; } = new List<Record>();
    }
    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
    [DllImport("user32.dll")] private static extern bool ShowWindow(IntPtr hwnd, int command);
    [DllImport("user32.dll", SetLastError = true)] private static extern bool SetWindowPos(IntPtr hwnd, IntPtr after, int x, int y, int w, int h, uint flags);
    [DllImport("user32.dll")] private static extern bool SetForegroundWindow(IntPtr hwnd);
    [DllImport("user32.dll")] private static extern IntPtr GetForegroundWindow();
    [StructLayout(LayoutKind.Sequential)] private struct MONITORINFO { public int Size; public RECT Monitor, Work; public uint Flags; }
    [DllImport("user32.dll")] private static extern IntPtr MonitorFromWindow(IntPtr hwnd, uint flags);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] private static extern bool GetMonitorInfo(IntPtr monitor, ref MONITORINFO info);
    private delegate bool EnumChildProc(IntPtr hwnd, IntPtr lparam);
    [DllImport("user32.dll")] private static extern bool GetClientRect(IntPtr hwnd, out RECT rect);
    [DllImport("user32.dll")] private static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
    [StructLayout(LayoutKind.Sequential)] private struct POINT { public int X, Y; }
    [DllImport("user32.dll")] private static extern bool ClientToScreen(IntPtr hwnd, ref POINT point);
    [DllImport("user32.dll")] private static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll")] private static extern bool IsZoomed(IntPtr hwnd);
    [DllImport("user32.dll")] private static extern bool IsIconic(IntPtr hwnd);
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
        var monitor = new MONITORINFO { Size = Marshal.SizeOf<MONITORINFO>() };
        if (!GetMonitorInfo(MonitorFromWindow(hwnd, 2), ref monitor))
            throw new InvalidOperationException("Monitor geometry unavailable");
        var name = new StringBuilder(256);
        GetClassName(hwnd, name, name.Capacity);
        return new Record {
            Hwnd = hwnd.ToInt64().ToString("x"),
            ClassName = name.ToString(),
            Visible = IsWindowVisible(hwnd),
            Maximized = IsZoomed(hwnd),
            Minimized = IsIconic(hwnd),
            Dpi = unchecked((int)GetDpiForWindow(hwnd)),
            ClientWidth = client.Width,
            ClientHeight = client.Height,
            ClientScreenX = origin.X,
            ClientScreenY = origin.Y,
            ScreenX = window.Left,
            ScreenY = window.Top,
            ScreenWidth = window.Width,
            ScreenHeight = window.Height,
            Monitor = monitor.Monitor
        };
    }
    public static void Transition(IntPtr hwnd, string action, int width, int height) {
        var previous = SetThreadDpiAwarenessContext(new IntPtr(-4));
        if (previous == IntPtr.Zero) throw new InvalidOperationException("DPI awareness unavailable");
        try {
            if (action == "maximise") ShowWindow(hwnd, 3);
            if (action == "restore" || action == "resize") ShowWindow(hwnd, 9);
            if (action == "resize") {
                if (width < 1 || height < 1) throw new ArgumentException("Positive physical outer size required");
                if (!SetWindowPos(hwnd, IntPtr.Zero, 0, 0, width, height, 0x0002 | 0x0004 | 0x0010))
                    throw new InvalidOperationException("SetWindowPos failed");
            }
            SetForegroundWindow(hwnd);
        } finally { SetThreadDpiAwarenessContext(previous); }
    }
    public static void Screenshot(IntPtr hwnd, string path) {
        var previous = SetThreadDpiAwarenessContext(new IntPtr(-4));
        if (previous == IntPtr.Zero) throw new InvalidOperationException("DPI awareness unavailable");
        try {
            if (GetForegroundWindow() != hwnd) throw new InvalidOperationException("App must be unobscured in foreground on a clean test desktop");
            var r = Read(hwnd);
            if (r.ClientScreenX < r.Monitor.Left || r.ClientScreenY < r.Monitor.Top ||
                r.ClientScreenX + r.ClientWidth > r.Monitor.Right || r.ClientScreenY + r.ClientHeight > r.Monitor.Bottom)
                throw new InvalidOperationException("Client is not fully on one monitor; screenshot would be clipped. Use a sufficiently large clean test desktop.");
            using (var bitmap = new System.Drawing.Bitmap(r.ClientWidth, r.ClientHeight)) {
                using (var graphics = System.Drawing.Graphics.FromImage(bitmap))
                    graphics.CopyFromScreen(r.ClientScreenX, r.ClientScreenY, 0, 0, bitmap.Size);
                bitmap.Save(path, System.Drawing.Imaging.ImageFormat.Png);
            }
        } finally { SetThreadDpiAwarenessContext(previous); }
    }
    public static Report Capture(IntPtr hwnd) {
        // PER_MONITOR_AWARE_V2 for this collector call only; restore the caller.
        // Without this, a normal PowerShell host can return virtualised rectangles.
        var previous = SetThreadDpiAwarenessContext(new IntPtr(-4));
        if (previous == IntPtr.Zero)
            throw new InvalidOperationException("Cannot establish per-monitor DPI-aware measurement");
        try { return CapturePhysical(hwnd); }
        finally { SetThreadDpiAwarenessContext(previous); }
    }
    private static Report CapturePhysical(IntPtr hwnd) {
        if (hwnd == IntPtr.Zero)
            throw new ArgumentException("No main native HWND: an interactive Windows desktop is required");
        var report = new Report { Main = Read(hwnd) };
        EnumChildProc callback = (child, ignored) => {
            // Capture the browser host as well as its child compositor HWNDs.
            // No titles, URLs or text contents are read.
            report.Descendants.Add(Read(child));
            return true;
        };
        // EnumChildWindows' return value is not an error/success contract.
        // An empty child list remains evidence, not a fabricated measurement.
        EnumChildWindows(hwnd, callback, IntPtr.Zero);
        return report;
    }
}
'@

$process = Get-Process -Id $ProcessId -ErrorAction Stop
if ($Action -ne 'observe') {
  [AwWindowMetrics]::Transition($process.MainWindowHandle, $Action, $Width, $Height)
}
$report = [AwWindowMetrics]::Capture($process.MainWindowHandle)
$report | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $OutputPath -Encoding UTF8
if ($ScreenshotPath) {
  [AwWindowMetrics]::Screenshot($process.MainWindowHandle, $ScreenshotPath)
}
Write-Host "Wrote native geometry; optional screenshot is only for the disposable invented-data app."
Write-Host "Compare client, child HWND and WebView2 sizes with the native 'Viewport sample' log."
Write-Host "This does NOT prove DOM viewport height: independently measure CSS innerHeight, app-shell and pane rects in a disposable Windows test."
