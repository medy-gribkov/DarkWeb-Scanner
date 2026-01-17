# Powershell Script to Organize SporeSec Folder
# USAGE: 
# 1. Open PowerShell
# 2. cd C:\Users\User\Coding\SporeSec
# 3. .\SporeSec-DarkWeb-Scanner\organize_folders.ps1

$root = "C:\Users\User\Coding\SporeSec"
$confirmation = Read-Host "This will move folders in $root to 'Production' and 'Archive'. Type 'YES' to continue"

if ($confirmation -ne 'YES') {
    Write-Host "Aborted."
    exit
}

# Create Directories
New-Item -ItemType Directory -Force -Path "$root\Production"
New-Item -ItemType Directory -Force -Path "$root\Archive"
New-Item -ItemType Directory -Force -Path "$root\Production\website"

# 1. Move Lead Scraper (The Go Project)
if (Test-Path "$root\Lead Scraper") {
    Write-Host "Moving Lead Scraper to Production..."
    Move-Item "$root\Lead Scraper" "$root\Production\lead-scraper"
} else {
    Write-Host "WARNING: 'Lead Scraper' folder not found."
}

# 2. Setup Website (The Coming Soon Page)
if (Test-Path "$root\Coming Soon Page Design") {
    Write-Host "Setting up Production/website..."
    # Copy contents so we don't lose the original until verified
    Copy-Item -Recurse "$root\Coming Soon Page Design\*" "$root\Production\website"
    # Move the original to Archive
    Move-Item "$root\Coming Soon Page Design" "$root\Archive\Coming Soon Page Design"
} else {
    Write-Host "WARNING: 'Coming Soon Page Design' not found."
}

# 3. Archive the rest
$keep = @("Production", "Archive", "desktop.ini", "organize_folders.ps1")

$items = Get-ChildItem -Path $root 
foreach ($item in $items) {
    if ($keep -notcontains $item.Name) {
        Write-Host "Archiving $($item.Name)..."
        Move-Item $item.FullName "$root\Archive"
    }
}

Write-Host "---------------------------------------------------"
Write-Host "Organization Complete!"
Write-Host "Your clean workspace is now in: $root\Production"
Write-Host "---------------------------------------------------"
