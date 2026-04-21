const http = require('http');
const fs = require('fs');
const path = require('path');
const url = require('url');

const PORT = process.env.PORT || 3000;
const HOSTNAME = '0.0.0.0';

// MIME types
const mimeTypes = {
    '.html': 'text/html; charset=utf-8',
    '.js': 'text/javascript; charset=utf-8',
    '.css': 'text/css; charset=utf-8',
    '.json': 'application/json',
    '.png': 'image/png',
    '.jpg': 'image/jpeg',
    '.jpeg': 'image/jpeg',
    '.gif': 'image/gif',
    '.svg': 'image/svg+xml',
    '.ico': 'image/x-icon',
    '.txt': 'text/plain; charset=utf-8'
};

// Create server
const server = http.createServer((req, res) => {
    // Parse URL
    let pathname = url.parse(req.url).pathname;
    
    // Default to dashboard.html for root
    if (pathname === '/' || pathname === '') {
        pathname = '/dashboard.html';
    }
    
    // Construct full file path
    let filePath = path.join(__dirname, pathname);
    
    // Security: prevent directory traversal attacks
    const realPath = path.resolve(filePath);
    const baseDir = path.resolve(__dirname);
    
    if (!realPath.startsWith(baseDir)) {
        res.writeHead(403, { 'Content-Type': 'text/plain' });
        res.end('403 Forbidden');
        return;
    }
    
    // Get file extension
    let extname = path.extname(filePath).toLowerCase();
    let contentType = mimeTypes[extname] || 'application/octet-stream';
    
    // Read and serve file
    fs.readFile(filePath, (err, data) => {
        if (err) {
            if (err.code === 'ENOENT') {
                // File not found - try with trailing slash or serve 404
                res.writeHead(404, { 'Content-Type': 'text/html' });
                res.end(`
                    <!DOCTYPE html>
                    <html>
                    <head><title>404 Not Found</title></head>
                    <body>
                        <h1>404 - File Not Found</h1>
                        <p>The file <code>${pathname}</code> could not be found on this server.</p>
                        <p><a href="/">Go to Dashboard</a></p>
                    </body>
                    </html>
                `);
            } else if (err.code === 'EACCES') {
                res.writeHead(403, { 'Content-Type': 'text/plain' });
                res.end('403 Forbidden');
            } else {
                res.writeHead(500, { 'Content-Type': 'text/plain' });
                res.end('500 Internal Server Error');
            }
            return;
        }
        
        // Set cache headers for static assets
        if (['.js', '.css', '.png', '.jpg', '.gif', '.svg', '.ico'].includes(extname)) {
            res.setHeader('Cache-Control', 'public, max-age=3600');
        }
        
        // Send file
        res.writeHead(200, { 'Content-Type': contentType });
        res.end(data);
    });
});

// Start server
server.listen(PORT, HOSTNAME, () => {
    console.log('\n');
    console.log('╔════════════════════════════════════════════════════════╗');
    console.log('║     🚀 FastDataBroker Admin Dashboard Server 🚀        ║');
    console.log('╚════════════════════════════════════════════════════════╝');
    console.log('\n✅ Server Status: RUNNING');
    console.log(`📍 Address: http://localhost:${PORT}`);
    console.log(`🌐 Hostname: ${HOSTNAME}`);
    console.log(`🔧 Port: ${PORT}\n`);
    console.log('📊 Features Available:');
    console.log('   ✓ Dashboard Overview');
    console.log('   ✓ Tenant Registration & Management');
    console.log('   ✓ API Key Generation & Revocation');
    console.log('   ✓ Credentials Display & Download');
    console.log('   ✓ Multi-User Support\n');
    console.log('📝 Test Cases:');
    console.log('   1. Open Dashboard: http://localhost:' + PORT);
    console.log('   2. Navigate to "Tenants" section');
    console.log('   3. Click "+ Register New Tenant"');
    console.log('   4. Fill form and submit');
    console.log('   5. View generated credentials\n');
    console.log('🛑 To stop server: Press Ctrl+C\n');
});

// Handle errors
server.on('error', (err) => {
    if (err.code === 'EADDRINUSE') {
        console.error(`❌ Port ${PORT} is already in use!`);
        console.error(`Try: PORT=3001 node server.js`);
    } else {
        console.error('Server error:', err.message);
    }
    process.exit(1);
});

// Graceful shutdown
process.on('SIGINT', () => {
    console.log('\n\n🛑 Server shutting down gracefully...');
    server.close(() => {
        console.log('✅ Server stopped');
        process.exit(0);
    });
});

// Handle uncaught errors
process.on('uncaughtException', (err) => {
    console.error('❌ Uncaught Exception:', err.message);
    process.exit(1);
});
