# Powershell Script to Clean Up Spore-Sec GitHub Org
# Targeted Org: Spore-Sec

Write-Host "Starting GitHub Cleanup for Organization: Spore-Sec"

function Rename-Repo {
    param ($old, $new)
    Write-Host "Renaming $old -> $new ..."
    gh repo rename $new --repo "Spore-Sec/$old" --yes
}

function Archive-Repo {
    param ($name)
    Write-Host "Archiving $name ..."
    gh repo archive "Spore-Sec/$name" --yes
}

# --- 1. Fix Typo ---
if ((gh repo view "Spore-Sec/Lead-Scarper" 2>$null)) {
    Rename-Repo "Lead-Scarper" "lead-scraper"
}

# --- 2. Archive Legacy ---
$reposToArchive = @(
    "SporeSec-DarkWeb-Scanner", 
    "SporeSec-Generative-Chat", 
    "SporeSec-CRM-Legacy",
    "SporeSec-WebSite-Scanner-Engine",
    "Fuck-Hezi-WhatsAppBot",
    "Python-Anti-Theft-Tool",
    "SproeSecBrandKitEngine"
)

foreach ($repo in $reposToArchive) {
    if ((gh repo view "Spore-Sec/$repo" 2>$null)) {
        Archive-Repo $repo
    }
}

# --- 3. Handle Broken Website Repo ---
if ((gh repo view "Spore-Sec/SporeSec-WebStie" 2>$null)) {
    # We rename it to legacy-broken-website before archiving so we can use "website" for the new one
    Rename-Repo "SporeSec-WebStie" "legacy-broken-website"
    Archive-Repo "legacy-broken-website"
}

Write-Host "---------------------------------------------------"
Write-Host "Spore-Sec Org Cleanup Complete!"
Write-Host "---------------------------------------------------"
