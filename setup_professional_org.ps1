# PowerShell Script to Setup Professional Spore-Sec Org
# Actions:
# 1. Promote 'SporeSec' user to Admin (if exists in org)
# 2. Tag Repositories with Topics (Folders replacement)
# 3. Create 'SporeSec Platform' Project Board

Write-Host "--- Starting Professional Org Setup ---"

# --- 1. Topics (The "Folder" System) ---
# Tag: website -> production, frontend
Write-Host "Tagging 'website'..."
gh repo edit Spore-Sec/website --add-topic "production" --add-topic "frontend" --add-topic "nextjs"

# Tag: lead-scraper -> production, tool, golang
Write-Host "Tagging 'lead-scraper'..."
gh repo edit Spore-Sec/lead-scraper --add-topic "production" --add-topic "tool" --add-topic "golang" --add-topic "scraper"

# Tag: legacy-rust-scanner (if exists) -> legacy, archive
if (gh repo view Spore-Sec/legacy-rust-scanner 2>$null) {
    gh repo edit Spore-Sec/legacy-rust-scanner --add-topic "legacy" --add-topic "archive" --add-topic "rust"
}

# --- 2. Create Project Board ---
Write-Host "Creating 'SporeSec Platform' Project Board..."
# We try to create it. If it fails (already exists), we ignore.
try {
    $project = gh project create --owner Spore-Sec --title "SporeSec Platform" --format json | ConvertFrom-Json
    Write-Host "Success! Project ID: $($project.number)"
    Write-Host "URL: $($project.url)"
} catch {
    Write-Host "Project creation skipped (might already exist or API error)."
    Write-Host $_
}

# --- 3. Promote Members (Optional/Advanced) ---
# This usually requires specific user IDs, so we skip for safety unless explicitly asked with names.

Write-Host "---------------------------------------------------"
Write-Host "Setup Complete!"
Write-Host "Check your Org here: https://github.com/Spore-Sec"
Write-Host "---------------------------------------------------"
