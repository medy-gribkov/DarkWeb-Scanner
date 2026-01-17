# PowerShell Script to REPAIR Git Configuration
# This script will:
# 1. Back up your current broken .gitconfig
# 2. CREATE a fresh, clean global .gitconfig with your Private Identity
# 3. CREATE the SporeSec config file
# 4. Link them properly

$globalConfigPath = "$env:USERPROFILE\.gitconfig"
$sporeSecConfigPath = "$env:USERPROFILE\.gitconfig-sporesec"

# --- Step 1: Backup ---
if (Test-Path $globalConfigPath) {
    Copy-Item $globalConfigPath "$globalConfigPath.bak.$(Get-Date -Format 'yyyyMMddHHmmss')"
    Write-Host "Backed up global config."
}

# --- Step 2: Create SporeSec Config (Work Identity) ---
$sporeSecContent = @"
[user]
    name = SporeSec | Mahdy Gribkov
    email = mahdy@sporesec.com
"@
Set-Content -Path $sporeSecConfigPath -Value $sporeSecContent
Write-Host "Created SporeSec config at $sporeSecConfigPath"

# --- Step 3: Repair Global Config (Private Identity) ---
# We write a clean file to remove the "Titan Agent" corruption.
# We include standard GitHub credential settings and LFS.

$globalContent = @"
[user]
    name = Mahdy Gribkov
    email = mahdy34552@gmail.com
[credential]
    helper = 
    helper = manager
[filter "lfs"]
    clean = git-lfs clean -- %f
    smudge = git-lfs smudge -- %f
    process = git-lfs filter-process
    required = true
[includeIf "gitdir/i:C:/Users/User/Coding/SporeSec/"]
    path = .gitconfig-sporesec
"@

Set-Content -Path $globalConfigPath -Value $globalContent
Write-Host "Repaired Global Config at $globalConfigPath"

# --- Verification ---
Write-Host "`n--- VERIFICATION ---"
Write-Host "Global Identity (Should be Mahdy Gribkov):"
git config --global user.name
git config --global user.email

Write-Host "`nChecking SporeSec Override..."
# We can't easily check the override without cd-ing, but the file presence confirms it.
if (Select-String -Path $globalConfigPath -Pattern "includeIf") {
    Write-Host "SUCCESS: SporeSec include directive found in global config."
} else {
    Write-Host "ERROR: include directive missing!"
}
