# Обновить сайт https://game.globthe.top (сервер 199.80.55.10, ISPConfig, сайт web2) из папки site.
# Запуск (пароль root — средой, в одинарных кавычках):
#   $env:SRV_PASS='<пароль>'; powershell -ExecutionPolicy Bypass -File tools\server\update-site.ps1
# Связь — встроенный OpenSSH Windows, пароль подаётся через SSH_ASKPASS (временный .cmd удаляется сразу).
# Так же устроено обновление сайта обложек (Code\Sites\Cover\tools\server).

$ErrorActionPreference = 'Stop'
if (-not $env:SRV_PASS) { throw 'Задайте $env:SRV_PASS' }
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$site = Join-Path $root 'site'
$tmp = Join-Path $env:LOCALAPPDATA 'Temp\blackwire-site-deploy'   # не $env:TEMP: там короткий путь вида OVC12F~1
New-Item -ItemType Directory -Force $tmp | Out-Null
$tgz = Join-Path $tmp 'site.tgz'
$ask = Join-Path $tmp 'askpass.cmd'
$empty = Join-Path $tmp 'empty.txt'
Set-Content -Path $empty -Value '' -Encoding ASCII
Set-Content -Path $ask -Value ('@echo ' + $env:SRV_PASS) -Encoding ASCII
$env:SSH_ASKPASS = $ask; $env:DISPLAY = 'localhost:0'; $env:SSH_ASKPASS_REQUIRE = 'force'
$opt = @('-o', 'StrictHostKeyChecking=accept-new', '-o', 'PreferredAuthentications=password,keyboard-interactive')

function Run($exe, $argList) {
  $p = Start-Process -FilePath "C:\Windows\System32\OpenSSH\$exe" -ArgumentList $argList -NoNewWindow -Wait -PassThru -RedirectStandardInput $empty
  if ($p.ExitCode -ne 0) { throw ("{0}: код {1}" -f $exe, $p.ExitCode) }
}

try {
  # Архив из самих файлов (без «.»), чтобы при распаковке не тронуть права корня сайта
  $items = Get-ChildItem -Force $site | ForEach-Object Name
  Push-Location $site; & tar.exe -czf $tgz @items; Pop-Location
  Run 'scp.exe' ($opt + @($tgz, 'root@199.80.55.10:/var/tmp/blackwire-site.tgz'))
  Run 'scp.exe' ($opt + @((Join-Path $PSScriptRoot 'unpack.sh'), 'root@199.80.55.10:/var/tmp/blackwire-site-unpack.sh'))
  Run 'ssh.exe' ($opt + @('root@199.80.55.10', 'bash /var/tmp/blackwire-site-unpack.sh'))
  Write-Output 'Готово: https://game.globthe.top/'
} finally {
  Remove-Item $ask -Force -ErrorAction SilentlyContinue
  Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
}
