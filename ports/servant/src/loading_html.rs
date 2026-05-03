/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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
        
        .actions { display: none; flex-direction: column; gap: 12px; margin-top: 30px; }
        .btn { padding: 12px 24px; border-radius: 12px; font-weight: 600; cursor: pointer; border: none; font-size: 14px; transition: all 0.2s ease; }
        .btn-primary { background-color: #0071e3; color: white; }
        .btn-primary:hover { background-color: #0077ed; }
        .btn-secondary { background-color: #e8e8ed; color: #0071e3; }
        .btn-secondary:hover { background-color: #d2d2d7; }
        .mime-badge { display: inline-block; padding: 4px 8px; background: #f5f5f7; border-radius: 6px; font-family: monospace; font-size: 11px; margin-top: 10px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="spinner" id="spinner"></div>
        <h1 id="title">Retrieving Content</h1>
        <div class="address" id="address"></div>
        <div id="loading-ui">
            <div class="status" id="status">Connecting to Autonomi network...</div>
            <div class="progress-container">
                <div class="progress-bar" id="progress-bar"></div>
            </div>
            <div class="stats" id="stats">0 bytes loaded</div>
        </div>
        <div id="unsupported-ui" class="actions">
            <div class="status">This file type cannot be rendered in-browser.</div>
            <div class="mime-badge" id="mime-display"></div>
            <button class="btn btn-primary" onclick="handleOpen()">Open with System Handler</button>
            <button class="btn btn-secondary" onclick="handleSave()">Save to Disk</button>
        </div>
    </div>

    <script>
        const address = window.location.hostname;
        const path = window.location.pathname;
        const fullPath = address + (path === '/' ? '' : path);
        
        document.getElementById('address').innerText = 'ant://' + fullPath;
        
        const RENDERABLE_MIMES = [
            'text/html', 'text/css', 'text/javascript', 'application/javascript', 'application/x-javascript',
            'text/plain', 'image/png', 'image/jpeg', 'image/gif', 'image/webp', 'image/svg+xml',
            'application/json', 'application/xml', 'text/xml'
        ];

        function isRenderable(mime) {
            if (!mime) return false;
            return RENDERABLE_MIMES.includes(mime.split(';')[0].toLowerCase());
        }

        function handleOpen() {
            fetch('ant://system-open/' + fullPath);
        }

        function handleSave() {
            fetch('ant://save-as/' + fullPath);
        }

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
                        document.getElementById('title').innerText = 'Error';
                        document.getElementById('status').innerText = data.error;
                        document.getElementById('status').style.color = '#ff3b30';
                        document.getElementById('spinner').style.display = 'none';
                    } else {
                        if (isRenderable(data.mime)) {
                            // Success! Refresh to get the actual content
                            const url = new URL(window.location.href);
                            url.searchParams.set('servant_raw', '1');
                            window.location.href = url.toString();
                        } else {
                            // Unsupported content
                            document.getElementById('title').innerText = 'Unsupported Content';
                            document.getElementById('loading-ui').style.display = 'none';
                            document.getElementById('spinner').style.display = 'none';
                            document.getElementById('unsupported-ui').style.display = 'flex';
                            document.getElementById('mime-display').innerText = data.mime || 'application/octet-stream';
                        }
                    }
                    return;
                }
                
                setTimeout(checkStatus, 500);
            } catch (e) {
                console.error('Failed to check status:', e);
                document.getElementById('title').innerText = 'Error';
                document.getElementById('status').innerText = 'Error fetching status';
                document.getElementById('status').style.color = '#ff3b30';
                document.getElementById('spinner').style.display = 'none';
            }
        }
        
        checkStatus();
    </script>
</body>
</html>
"#;
