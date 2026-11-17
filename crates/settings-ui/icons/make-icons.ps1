# Renders the procedural app icon (same geometry as assets/make-icon.ps1) to PNGs
# at the sizes Tauri needs, so they are crisp instead of upscaled from 48 px.
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$palette = @(
    @(14, 68, 41),
    @(0, 109, 50),
    @(38, 166, 65),
    @(57, 211, 83)
)
$bg = @(13, 17, 23)

function New-Pixels([int]$S) {
    $px = New-Object 'byte[]' ($S * $S * 4)
    $r = [math]::Max(2, [int]($S * 0.20))
    for ($y = 0; $y -lt $S; $y++) {
        for ($x = 0; $x -lt $S; $x++) {
            $cx = [math]::Min([math]::Max($x, $r), $S - 1 - $r)
            $cy = [math]::Min([math]::Max($y, $r), $S - 1 - $r)
            $dx = $x - $cx
            $dy = $y - $cy
            if (($dx * $dx + $dy * $dy) -le $r * $r) {
                $i = ($y * $S + $x) * 4
                $px[$i] = $bg[2]; $px[$i + 1] = $bg[1]; $px[$i + 2] = $bg[0]; $px[$i + 3] = 255
            }
        }
    }
    $cell = [math]::Max(2, [int]($S * 0.13))
    $gap = [math]::Max(1, [int]($S * 0.035))
    $total = 4 * $cell + 3 * $gap
    $startX = [int](($S - $total) / 2)
    $startY = $startX
    for ($gy = 0; $gy -lt 4; $gy++) {
        for ($gx = 0; $gx -lt 4; $gx++) {
            $c = $palette[(($gx * 3 + $gy * 5) % 4)]
            $x0 = $startX + $gx * ($cell + $gap)
            $y0 = $startY + $gy * ($cell + $gap)
            for ($y = $y0; $y -lt $y0 + $cell; $y++) {
                for ($x = $x0; $x -lt $x0 + $cell; $x++) {
                    if ($x -ge 0 -and $y -ge 0 -and $x -lt $S -and $y -lt $S) {
                        $i = ($y * $S + $x) * 4
                        $px[$i] = $c[2]; $px[$i + 1] = $c[1]; $px[$i + 2] = $c[0]; $px[$i + 3] = 255
                    }
                }
            }
        }
    }
    return , $px
}

function Save-Png([int]$S, [string]$Target) {
    $px = New-Pixels $S
    $bmp = New-Object System.Drawing.Bitmap($S, $S, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    for ($y = 0; $y -lt $S; $y++) {
        for ($x = 0; $x -lt $S; $x++) {
            $i = ($y * $S + $x) * 4
            $c = [System.Drawing.Color]::FromArgb($px[$i + 3], $px[$i], $px[$i + 1], $px[$i + 2])
            $bmp.SetPixel($x, $y, $c)
        }
    }
    $bmp.Save($Target, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    Write-Host ("{0} -> {1}x{1} ({2} bytes)" -f (Split-Path $Target -Leaf), $S, (Get-Item $Target).Length)
}

$dir = Join-Path $PSScriptRoot 'icons'
$dir = $PSScriptRoot
Save-Png 32  (Join-Path $dir '32x32.png')
Save-Png 128 (Join-Path $dir '128x128.png')
Save-Png 256 (Join-Path $dir '128x128@2x.png')
Save-Png 256 (Join-Path $dir 'icon.png')
