# Generates assets/icon.ico (16/32/48/256 px) without any image library.
# Run: powershell -NoProfile -ExecutionPolicy Bypass -File assets/make-icon.ps1
$ErrorActionPreference = 'Stop'
$out = Join-Path $PSScriptRoot 'icon.ico'

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

function New-ImageData([int]$S, [byte[]]$px) {
    $maskRow = [int]([math]::Ceiling($S / 32.0) * 4)
    $ms = New-Object System.IO.MemoryStream
    $bw = New-Object System.IO.BinaryWriter($ms)
    $bw.Write([uint32]40)
    $bw.Write([int32]$S)
    $bw.Write([int32]($S * 2))
    $bw.Write([uint16]1)
    $bw.Write([uint16]32)
    $bw.Write([uint32]0)
    $bw.Write([uint32]($S * $S * 4 + $maskRow * $S))
    $bw.Write([int32]0)
    $bw.Write([int32]0)
    $bw.Write([uint32]0)
    $bw.Write([uint32]0)
    for ($y = $S - 1; $y -ge 0; $y--) {
        for ($x = 0; $x -lt $S; $x++) {
            $i = ($y * $S + $x) * 4
            $bw.Write([byte]$px[$i + 2])
            $bw.Write([byte]$px[$i + 1])
            $bw.Write([byte]$px[$i])
            $bw.Write([byte]$px[$i + 3])
        }
    }
    $bw.Write((New-Object 'byte[]' ($maskRow * $S)))
    $bw.Flush()
    return , $ms.ToArray()
}

$sizes = @(16, 32, 48, 256)
$images = @()
foreach ($s in $sizes) {
    $images += , (New-ImageData $s (New-Pixels $s))
}

$ms = New-Object System.IO.MemoryStream
$bw = New-Object System.IO.BinaryWriter($ms)
$bw.Write([uint16]0)
$bw.Write([uint16]1)
$bw.Write([uint16]$sizes.Count)
$offset = 6 + 16 * $sizes.Count
for ($k = 0; $k -lt $sizes.Count; $k++) {
    $s = $sizes[$k]
    $data = $images[$k]
    $dim = if ($s -ge 256) { 0 } else { $s }
    $bw.Write([byte]$dim)
    $bw.Write([byte]$dim)
    $bw.Write([byte]0)
    $bw.Write([byte]0)
    $bw.Write([uint16]1)
    $bw.Write([uint16]32)
    $bw.Write([uint32]$data.Length)
    $bw.Write([uint32]$offset)
    $offset += $data.Length
}
foreach ($d in $images) { $bw.Write($d) }
$bw.Flush()
[System.IO.File]::WriteAllBytes($out, $ms.ToArray())
Write-Host "wrote $out ($((Get-Item $out).Length) bytes)"
