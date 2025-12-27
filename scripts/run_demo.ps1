# Demo script: start server, run one demo scan, stop server
$env:SPORE_SIGNATURE = "demo"
$env:FRONTEND_DOMAIN = "http://localhost:3000"

$server = Start-Process -FilePath "cargo" -ArgumentList "run" -PassThru
Write-Output "Server started (PID: $($server.Id)). Waiting 2s for startup..."
Start-Sleep -Seconds 2

$response = Invoke-RestMethod -Uri "http://127.0.0.1:8080/v1/scan?query=test" -Headers @{ "X-Spore-Signature" = "demo" }
Write-Output "Demo scan response:" 
$response | ConvertTo-Json -Depth 5

# Stop the server
Stop-Process -Id $server.Id -Force
Write-Output "Server stopped."