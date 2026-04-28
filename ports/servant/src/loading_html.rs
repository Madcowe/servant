pub const LOADING_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Loading from Autonomi...</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh; margin: 0; background-color: #f5f5f7; color: #1d1d1f; }
        .container { width: 400px; padding: 40px; background: white; border-radius: 20px; box-shadow: 0 10px 30px rgba(0,0,0,0.05); text-align: center; }
        h1 { margin-top: 0; font-weight: 600; font-size: 24px; }
        .address { font-family: monospace; font-size: 12px; color: #86868b; word-break: break-all; margin-bottom: 30px; }
        .progress-container { width: 100%; height: 8px; background: #e5e5e7; border-radius: 4px; overflow: hidden; margin-bottom: 20px; }
        .progress-bar { height: 100%; background: #0071e3; width: 0%; transition: width 0.3s ease; }
        .status { font-size: 14px; margin-bottom: 10px; font-weight: 500; }
        .stats { font-size: 12px; color: #86868b; }
        .spinner { border: 3px solid #f3f3f3; border-top: 3px solid #0071e3; border-radius: 50%; width: 30px; height: 30px; animation: spin 1s linear infinite; margin: 0 auto 20px; }
        @keyframes spin { 0% { transform: rotate(0deg); } 100% { transform: rotate(360deg); } }
    </style>
</head>
<body>
    <div class="container">
        <div class="spinner"></div>
        <h1>Retrieving Content</h1>
        <div class="address" id="address"></div>
        <div class="status" id="status">Connecting to Autonomi network...</div>
        <div class="progress-container">
            <div class="progress-bar" id="progress-bar"></div>
        </div>
        <div class="stats" id="stats">0 bytes loaded</div>
    </div>

    <script>
        const address = window.location.pathname.substring(1).split('?')[0];
        document.getElementById('address').innerText = 'ant://' + address;
        
        async function checkStatus() {
            try {
                const response = await fetch('ant://loading-status/' + address);
                const data = await response.json();
                
                document.getElementById('status').innerText = data.status;
                
                if (data.total_bytes) {
                    const percent = Math.round((data.bytes_loaded / data.total_bytes) * 100);
                    document.getElementById('progress-bar').style.width = percent + '%';
                    document.getElementById('stats').innerText = `${data.bytes_loaded} / ${data.total_bytes} items`;
                } else {
                    document.getElementById('stats').innerText = `${data.bytes_loaded} items loaded`;
                }
                
                if (data.finished) {
                    if (data.error) {
                        document.getElementById('status').innerText = 'Error: ' + data.error;
                        document.getElementById('status').style.color = '#ff3b30';
                    } else {
                        // Success! Refresh to get the actual content
                        const url = new URL(window.location.href);
                        url.searchParams.set('servant_raw', '1');
                        window.location.href = url.toString();
                    }
                    return;
                }
                
                setTimeout(checkStatus, 500);
            } catch (e) {
                console.error('Failed to check status:', e);
                setTimeout(checkStatus, 1000);
            }
        }
        
        checkStatus();
    </script>
</body>
</html>
"#;
