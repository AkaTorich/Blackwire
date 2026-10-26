# Снимок монитора в файл (с учётом масштаба DPI каждого монитора).
#   pwsh tools\shot.ps1 -Out shot.png            # основной монитор
#   pwsh tools\shot.ps1 -Out shot.png -Screen 1  # второй монитор
# Для проверки окон игры: Blackwire.exe --load --open terminal --open mail --arrange grid, затем этот снимок.
param([string]$Out, [int]$Screen = 0)
$code = @'
using System; using System.Runtime.InteropServices;
public static class DpiCtx {
  [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr ctx);
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr ctx);
}
'@
if (-not ('DpiCtx' -as [type])) { Add-Type -TypeDefinition $code }
# per-monitor v2: процесс pwsh уже мог выбрать режим, поэтому ставим и потоку
[DpiCtx]::SetProcessDpiAwarenessContext([IntPtr]::new(-4)) | Out-Null
[DpiCtx]::SetThreadDpiAwarenessContext([IntPtr]::new(-4)) | Out-Null
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms
$s = [System.Windows.Forms.Screen]::AllScreens[$Screen]
$b = $s.Bounds
$bmp = New-Object System.Drawing.Bitmap $b.Width, $b.Height
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($b.Location, [System.Drawing.Point]::Empty, $b.Size)
$bmp.Save($Out)
$g.Dispose(); $bmp.Dispose()
"$Out $($b.Width)x$($b.Height)"
