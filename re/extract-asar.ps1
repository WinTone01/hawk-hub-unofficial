param([string]$Asar, [string]$Out)
$fs = [IO.File]::OpenRead($Asar)
$br = New-Object IO.BinaryReader($fs)
$null = $br.ReadUInt32(); $headerSize = $br.ReadUInt32()
$null = $br.ReadUInt32(); $jsonLen = $br.ReadUInt32()
$json = [Text.Encoding]::UTF8.GetString($br.ReadBytes($jsonLen))
$base = 8 + $headerSize
Add-Type -AssemblyName System.Web.Extensions
$ser = New-Object System.Web.Script.Serialization.JavaScriptSerializer
$ser.MaxJsonLength = [int]::MaxValue
$root = $ser.DeserializeObject($json)
function Walk($node, $path) {
  foreach ($k in $node['files'].Keys) {
    $n = $node['files'][$k]; $p = Join-Path $path $k
    if ($n.ContainsKey('files')) { New-Item -ItemType Directory -Force $p | Out-Null; Walk $n $p }
    elseif ($n.ContainsKey('unpacked') -and $n['unpacked']) { }
    elseif ($n.ContainsKey('offset')) {
      $fs.Position = $base + [int64]$n['offset']
      [IO.File]::WriteAllBytes($p, $br.ReadBytes([int]$n['size']))
    }
  }
}
New-Item -ItemType Directory -Force $Out | Out-Null
Walk $root $Out
$fs.Close()
