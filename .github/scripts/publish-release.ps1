$ErrorActionPreference = 'Stop'
$orbitVersion = python -c "import tomllib; print(tomllib.load(open('Cargo.toml', 'rb'))['workspace']['package']['version'])"
if ($LASTEXITCODE -ne 0) { throw 'Cannot read Cargo version' }
$orbitTag = "v$orbitVersion"
$orbitPages = gh api "repos/$env:GH_REPO/releases?per_page=100" --paginate --slurp
if ($LASTEXITCODE -ne 0) { throw 'Cannot query releases' }
$orbitExisting = $null
foreach ($page in ($orbitPages | ConvertFrom-Json)) {
    foreach ($release in $page) {
        if ($release.tag_name -eq $orbitTag) { $orbitExisting = $release }
    }
}
if ($null -ne $orbitExisting -and !$orbitExisting.draft) {
    "Release $orbitTag already published; keeping its files unchanged." | Write-Output
    "Release already published: $($orbitExisting.html_url)" >> $env:GITHUB_STEP_SUMMARY
    exit 0
}
if ($null -ne $orbitExisting -and $orbitExisting.target_commitish -ne $env:GITHUB_SHA) {
    throw 'Existing draft belongs to another commit; refusing to overwrite it'
}
$orbitAssets = @("dist/OrbitAxion-$orbitVersion.exe", "dist/OrbitAxion-$orbitVersion-x64.msi", 'dist/SHA256SUMS.txt', 'dist/THIRD_PARTY_NOTICES.txt', 'LICENSE')
foreach ($asset in $orbitAssets) {
    if (!(Test-Path -LiteralPath $asset -PathType Leaf)) { throw "Missing release asset: $asset" }
}
if ($null -eq $orbitExisting) {
    gh release create $orbitTag @orbitAssets --draft --target $env:GITHUB_SHA --title "Orbit Axion $orbitTag" --generate-notes
    if ($LASTEXITCODE -ne 0) { throw 'Cannot create release draft' }
} else {
    gh release upload $orbitTag @orbitAssets --clobber
    if ($LASTEXITCODE -ne 0) { throw 'Cannot finish draft uploads' }
}
$orbitPages = gh api "repos/$env:GH_REPO/releases?per_page=100" --paginate --slurp
if ($LASTEXITCODE -ne 0) { throw 'Cannot verify uploaded release' }
$orbitDraft = $null
foreach ($page in ($orbitPages | ConvertFrom-Json)) {
    foreach ($release in $page) {
        if ($release.tag_name -eq $orbitTag) { $orbitDraft = $release }
    }
}
if ($null -eq $orbitDraft -or !$orbitDraft.draft) { throw 'Release draft not found' }
if (@($orbitDraft.assets).Count -ne $orbitAssets.Count) { throw 'Unexpected release asset count' }
foreach ($asset in $orbitAssets) {
    $name = Split-Path $asset -Leaf
    $uploaded = @($orbitDraft.assets | Where-Object { $_.name -eq $name })
    $expected = 'sha256:' + (Get-FileHash -LiteralPath $asset -Algorithm SHA256).Hash.ToLower()
    if ($uploaded.Count -ne 1 -or $uploaded[0].state -ne 'uploaded' -or $uploaded[0].digest -ne $expected) {
        throw "Release asset failed SHA-256 verification: $name"
    }
}
$orbitFlags = @('--latest')
if ($orbitVersion.Contains('-')) { $orbitFlags = @('--prerelease', '--latest=false') }
gh release edit $orbitTag --draft=false @orbitFlags
if ($LASTEXITCODE -ne 0) { throw 'Cannot publish verified release' }
"Published $orbitTag with five verified assets." | Write-Output
"Published release: $($orbitDraft.html_url)" >> $env:GITHUB_STEP_SUMMARY
