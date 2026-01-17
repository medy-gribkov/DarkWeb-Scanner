# PowerShell Script to Populate SporeSec Project Board
# Actions:
# 1. Inspect Project 2 "SporeSec Platform"
# 2. Create Issues in 'website' repo -> Link to Project
# 3. Create Issues in 'lead-scraper' (or platform-api when ready)

$org = "Spore-Sec"
$projectNumber = 2 # From your previous output
$websiteRepo = "website"

Write-Host "--- Populating SporeSec Project Board ---"

function Create-Issue {
    param ($repo, $title, $body, $labels)
    Write-Host "Creating Issue: $title in $repo..."
    # Create Issue and immediately add to Project
    gh issue create --repo "$org/$repo" --title "$title" --body "$body" --label "$labels" --project "SporeSec Platform"
}

# --- Phase 1: Website Tasks (Immediate) ---
Create-Issue $websiteRepo "Setup: Clean Next.js Build" "Initialize clean Next.js project structure in Production/website and push." "backend,setup"
Create-Issue $websiteRepo "Design: Coming Soon Landing Page" "Implement the Coming Soon page design (Waitlist, Branding, Hero Section)." "frontend,design"
Create-Issue $websiteRepo "Infra: Vercel Deployment" "Connect repo to Vercel for automated deployments." "devops"

# --- Phase 2: Platform Tasks (Future) ---
# We use the website repo as a placeholder for general backend tasks if platform-api isn't pushed yet, 
# OR we can create 'Draft Issues' via API (more complex). 
# For now, let's just use 'website' or 'lead-scraper' as the home, or better:
# Create them in 'website' but tagged 'backend' so they appear on the board.

Create-Issue $websiteRepo "Backend: Scaffold .NET 8 Solution" "Initialize SporeSec.sln with Clean Architecture layers." "backend,csharp"
Create-Issue $websiteRepo "Backend: Auth Implementation" "Setup Identity Framework and JWT handling." "backend,auth"
Create-Issue $websiteRepo "Scraper: Job Queue Integration" "Design RabbitMQ/Redis queue for async scraper jobs." "backend,architecture"

Write-Host "---------------------------------------------------"
Write-Host "Population Complete!"
Write-Host "View Board: https://github.com/orgs/$org/projects/$projectNumber"
Write-Host "---------------------------------------------------"
