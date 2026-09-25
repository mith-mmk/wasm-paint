[CmdletBinding()]
param(
    [Parameter()]
    [ValidateRange(1, 65535)]
    [int] $Port = 8000
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path -LiteralPath $PSScriptRoot).Path
$packageDir = Join-Path $repoRoot 'wasm-paint/pkg'
$requiredFiles = @(
    (Join-Path $packageDir 'paint.js'),
    (Join-Path $packageDir 'paint_bg.wasm'),
    (Join-Path $packageDir 'paint.d.ts')
)

if (@($requiredFiles | Where-Object { -not (Test-Path -LiteralPath $_) }).Count -gt 0) {
    throw 'Generated WASM files are missing. Build them first: Push-Location wasm-paint; wasm-pack build -t web; Pop-Location'
}

$bindings = Get-Content -LiteralPath (Join-Path $packageDir 'paint.d.ts') -Raw
foreach ($method in @('deleteLayer', 'getLayerImageData', 'setLayerImageData')) {
    if ($bindings -notmatch "\b$method\s*\(") {
        throw "Generated WASM bindings are outdated. Rebuild them with: Push-Location wasm-paint; wasm-pack build -t web; Pop-Location"
    }
}

$python = Get-Command -Name 'py' -ErrorAction SilentlyContinue
$pythonPrefix = @()
if ($python) {
    $pythonPrefix = @('-3')
} else {
    $python = Get-Command -Name 'python' -ErrorAction SilentlyContinue
}
if (-not $python) {
    throw 'Python 3 is required to run the test server. Install Python 3 and retry.'
}

$url = "http://127.0.0.1:$Port/test/web-paint-ui.html"
Write-Host "Serving $repoRoot at http://127.0.0.1:$Port/"
Write-Host "Open $url"
Write-Host 'Press Ctrl+C to stop the server.'

$serverArgs = @($pythonPrefix) + @(
    '-m', 'http.server', [string] $Port,
    '--bind', '127.0.0.1',
    '--directory', $repoRoot
)
& $python.Source @serverArgs
if ($LASTEXITCODE -ne 0) {
    throw "The test server exited with code $LASTEXITCODE."
}
