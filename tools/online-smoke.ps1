# Проверка онлайн-сервера без игры: регистрирует (или входит) игрока, открывает
# терминал, выполняет команды, пишет в чат. Только для сервера без шифрования.
# Пример: powershell -File tools\online-smoke.ps1 -Port 7790 -User alice -Pass secret1 -Register -Commands "whoami;ls ~"
param(
  [string]$HostName = "127.0.0.1",
  [int]$Port = 7777,
  [string]$User = "tester",
  [string]$Pass = "secret1",
  [switch]$Register,
  [string]$Commands = "whoami;hostname"
)
$ErrorActionPreference = "Stop"
$client = New-Object System.Net.Sockets.TcpClient($HostName, $Port)
$stream = $client.GetStream()
$utf8 = New-Object System.Text.UTF8Encoding($false)
$reader = New-Object System.IO.StreamReader($stream, $utf8)
$writer = New-Object System.IO.StreamWriter($stream, $utf8)
$writer.AutoFlush = $true
$script:id = 0

function Call($cmd, $argsObj) {
  $script:id++
  $line = @{ id = $script:id; cmd = $cmd; args = $argsObj } | ConvertTo-Json -Compress -Depth 10
  $writer.WriteLine($line)
  while ($true) {
    $resp = $reader.ReadLine()
    if ($null -eq $resp) { throw "соединение закрыто" }
    $o = $resp | ConvertFrom-Json
    if ($o.id -eq $script:id) { return $o }
  }
}

$r = Call "auth" @{ user = $User; pass = $Pass; register = [bool]$Register; lang = "ru"; proto = 1 }
if ($r.err) { Write-Output "ВХОД: ошибка: $($r.err)"; exit 1 }
Write-Output "ВХОД: $($r.ok.handle), в сети: $($r.ok.online -join ', ')"
$sid = (Call "shell_open" @{}).ok
foreach ($c in ($Commands -split ";")) {
  $res = Call "shell_exec" @{ sessionId = $sid; line = $c }
  if ($res.err) { Write-Output "> $c  ОШИБКА $($res.err)"; continue }
  Write-Output "> $c"
  foreach ($l in $res.ok.lines) { Write-Output "  $($l.text)" }
}
$m = Call "mail_list" @{ folder = "inbox" }
Write-Output "ПОЧТА: писем $(@($m.ok).Count)"
$st = Call "sys_status" @{}
Write-Output "МАШИНА: $($st.ok.hostname) $($st.ok.ip)"
$null = Call "chat_send" @{ text = "привет от $User" }
$w = Call "who" @{}
Write-Output "В СЕТИ: $($w.ok -join ', ')"
$client.Close()
