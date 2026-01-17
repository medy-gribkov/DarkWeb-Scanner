# PowerShell Script to Polish SporeSec Org
# Actions:
# 1. Promote AngelRattner and SporeSec to ADMIN
# 2. Pin Repositories to Profile (website, lead-scraper, platform-api)

Write-Host "--- Starting Org Polish ---"

# --- 1. Promote Members ---
$admins = @("AngelRattner", "SporeSec")
foreach ($user in $admins) {
    Write-Host "Promoting $user to ADMIN..."
    # 'admin' role in API is usually sufficient for Owner-like privileges
    gh api -X PUT "orgs/Spore-Sec/memberships/$user" -f role=admin
}

# --- 2. Pin Repositories (The "Clean Look") ---
# This uses the GraphQL API because REST API for pinning is not straightforward.
# We will verify repos exist first.

$reposToPin = @("website", "lead-scraper", "platform-api")

# We need the Node IDs of the repos to pin them.
Write-Host "`nFetching Repository IDs..."
$repoIds = @()
foreach ($repoName in $reposToPin) {
    try {
        $id = gh api "repos/Spore-Sec/$repoName" --jq .node_id
        if ($id) {
            Write-Host "Found $repoName ($id)"
            $repoIds += $id
        }
    } catch {
        Write-Host "Skipping $repoName (Not found)"
    }
}

if ($repoIds.Count -gt 0) {
    Write-Host "`nPinning Repositories..."
    # GraphQL mutation to pin
    # Note: This replaces existing pins.
    
    # We first need the Owner ID (Spore-Sec Org ID)
    $orgId = gh api "orgs/Spore-Sec" --jq .node_id
    
    # Construct GraphQL query
    $idsFormatted = $repoIds | ForEach-Object { "`"$_`"" }
    $idsString = $idsFormatted -join ", "
    
    $query = @"
mutation {
  updateOrganizationPinnedItems(input: {organizationId: `"$orgId`", profileItemIds: [$idsString]}) {
    organization {
      pinnedItems(first: 10) {
        nodes {
          ... on Repository {
            name
          }
        }
      }
    }
  }
}
"@
    
    gh api graphql -f query="$query"
    Write-Host "Pins updated."
} else {
    Write-Host "No active repositories found to pin."
}

Write-Host "---------------------------------------------------"
Write-Host "Polish Complete!"
Write-Host "---------------------------------------------------"
