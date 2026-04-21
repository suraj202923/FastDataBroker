/**
 * Enhanced FastDataBroker Admin Dashboard Server
 * With QUIC Protocol Support & Pre-Shared Key (PSK) Authentication
 */

const http = require('http');
const fs = require('fs');
const path = require('path');
const url = require('url');
const querystring = require('querystring');
const TenantStorage = require('./lib/tenantStorage');
const BrokerClient = require('./lib/brokerClient');
const { QuicPskManager, QuicConnectionManager } = require('./lib/quicPskAdapter');

const PORT = process.env.PORT || 3000;
const HOSTNAME = '0.0.0.0';
const QUIC_PORT = process.env.QUIC_PORT || 9092;

// Initialize encrypted tenant storage
const storage = new TenantStorage();

// Initialize QUIC PSK authentication
const quicPskManager = new QuicPskManager();
const quicConnManager = new QuicConnectionManager();

// Initialize broker client (connects to FastDataBroker on port 9092 via QUIC)
const brokerClient = new BrokerClient({
    host: 'localhost',
    port: 9092,
    quicPort: QUIC_PORT,
    protocol: 'quic'
});

// Debug: Load initial sample data if empty
if (storage.getAllTenants().length === 0) {
    console.log('\n📦 Loading sample tenants...');
    storage.addTenant({
        name: 'Acme Corp',
        email: 'contact@acme.com',
        tier: 'pro',
        maxConnections: 100
    });
    storage.addTenant({
        name: 'Tech Startup',
        email: 'admin@techstartup.com',
        tier: 'starter',
        maxConnections: 50
    });
}

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

// Helper functions
function parseRequestBody(req) {
    return new Promise((resolve, reject) => {
        let body = '';
        req.on('data', chunk => {
            body += chunk.toString();
        });
        req.on('end', () => {
            try {
                resolve(JSON.parse(body));
            } catch (err) {
                resolve({});
            }
        });
        req.on('error', reject);
    });
}

function sendJSON(res, statusCode, data) {
    res.writeHead(statusCode, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify(data, null, 2));
}

function sendError(res, statusCode, message) {
    sendJSON(res, statusCode, { error: true, message });
}

function generateRandomString(length) {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    let result = '';
    for (let i = 0; i < length; i++) {
        result += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return result;
}

// API Routes Handler
function handleAPI(pathname, req, res) {
    // CORS headers
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
    
    if (req.method === 'OPTIONS') {
        res.writeHead(200);
        res.end();
        return;
    }
    
    try {
        // ===== TENANT ENDPOINTS =====
        
        if (pathname === '/api/tenants' && req.method === 'GET') {
            const tenants = storage.getAllTenants();
            sendJSON(res, 200, { success: true, tenants });
            return;
        }
        
        if (pathname === '/api/tenants' && req.method === 'POST') {
            parseRequestBody(req).then(body => {
                const tenant = storage.addTenant(body);
                sendJSON(res, 201, { success: true, tenant });
            });
            return;
        }
        
        if (pathname.match(/^\/api\/tenants\/[a-zA-Z0-9_]+$/) && req.method === 'GET') {
            const tenantId = pathname.split('/')[3];
            const tenant = storage.getTenant(tenantId);
            if (!tenant) {
                sendError(res, 404, 'Tenant not found');
                return;
            }
            sendJSON(res, 200, { success: true, tenant });
            return;
        }
        
        if (pathname.match(/^\/api\/tenants\/[a-zA-Z0-9_]+$/) && req.method === 'PUT') {
            const tenantId = pathname.split('/')[3];
            parseRequestBody(req).then(body => {
                const tenant = storage.updateTenant(tenantId, body);
                sendJSON(res, 200, { success: true, tenant });
            });
            return;
        }
        
        if (pathname.match(/^\/api\/tenants\/[a-zA-Z0-9_]+$/) && req.method === 'DELETE') {
            const tenantId = pathname.split('/')[3];
            const tenant = storage.deleteTenant(tenantId);
            sendJSON(res, 200, { success: true, deleted: tenant });
            return;
        }
        
        // ===== SECRETS/API KEYS ENDPOINTS =====
        
        if (pathname === '/api/secrets' && req.method === 'GET') {
            const secrets = storage.getAllSecrets();
            sendJSON(res, 200, { success: true, secrets });
            return;
        }
        
        if (pathname === '/api/secrets' && req.method === 'POST') {
            parseRequestBody(req).then(body => {
                const secret = storage.addSecret(body);
                sendJSON(res, 201, { success: true, secret });
            });
            return;
        }
        
        if (pathname.match(/^\/api\/secrets\/tenant\/[a-zA-Z0-9_]+$/) && req.method === 'GET') {
            const tenantId = pathname.split('/')[4];
            const secrets = storage.getSecretsByTenant(tenantId);
            sendJSON(res, 200, { success: true, secrets });
            return;
        }
        
        if (pathname.match(/^\/api\/secrets\/[a-zA-Z0-9_]+$/) && req.method === 'DELETE') {
            const secretId = pathname.split('/')[3];
            const secret = storage.deleteSecret(secretId);
            sendJSON(res, 200, { success: true, deleted: secret });
            return;
        }
        
        // Generate new API key for tenant
        if (pathname === '/api/generate-key' && req.method === 'POST') {
            parseRequestBody(req).then(body => {
                if (!body.tenantId) {
                    sendError(res, 400, 'tenantId required');
                    return;
                }
                
                const apiKey = 'fdb_key_' + generateRandomString(16);
                const apiSecret = 'fdb_secret_' + generateRandomString(24);
                
                const secret = storage.addSecret({
                    name: body.name || 'Generated Key',
                    tenant: body.tenantId,
                    key: apiKey,
                    secret: apiSecret
                });
                
                sendJSON(res, 201, {
                    success: true,
                    secret,
                    endpoint: 'localhost:9092',
                    connectionString: `fdb://${apiKey}:${apiSecret}@localhost:9092`
                });
            });
            return;
        }
        
        // ===== BACKUP & RESTORE =====
        
        if (pathname === '/api/backup/export' && req.method === 'GET') {
            const backup = storage.exportBackup();
            sendJSON(res, 200, { success: true, backup });
            return;
        }
        
        if (pathname === '/api/backup/import' && req.method === 'POST') {
            parseRequestBody(req).then(body => {
                if (!body.backup) {
                    sendError(res, 400, 'backup data required');
                    return;
                }
                const result = storage.importBackup(body.backup);
                sendJSON(res, 200, { success: true, result });
            });
            return;
        }
        
        if (pathname === '/api/backup/clear' && req.method === 'POST') {
            storage.clearAll();
            sendJSON(res, 200, { success: true, message: 'All data cleared' });
            return;
        }
        
        // ===== STATS & INFO =====
        
        if (pathname === '/api/stats' && req.method === 'GET') {
            const stats = storage.getStats();
            sendJSON(res, 200, { success: true, stats });
            return;
        }
        
        // ===== BROKER MONITORING ENDPOINTS =====
        
        if (pathname === '/api/brokers' && req.method === 'GET') {
            brokerClient.getBrokers().then(brokers => {
                sendJSON(res, 200, { success: true, brokers });
            }).catch(err => {
                sendError(res, 500, 'Failed to fetch brokers: ' + err.message);
            });
            return;
        }
        
        if (pathname === '/api/streams' && req.method === 'GET') {
            brokerClient.getStreams().then(streams => {
                sendJSON(res, 200, { success: true, streams });
            }).catch(err => {
                sendError(res, 500, 'Failed to fetch streams: ' + err.message);
            });
            return;
        }
        
        if (pathname === '/api/queue-stats' && req.method === 'GET') {
            brokerClient.getQueueStats().then(stats => {
                sendJSON(res, 200, { success: true, stats });
            }).catch(err => {
                sendError(res, 500, 'Failed to fetch queue stats: ' + err.message);
            });
            return;
        }
        
        if (pathname === '/api/metrics' && req.method === 'GET') {
            brokerClient.getMetrics().then(metrics => {
                sendJSON(res, 200, { success: true, metrics });
            }).catch(err => {
                sendError(res, 500, 'Failed to fetch metrics: ' + err.message);
            });
            return;
        }
        
        if (pathname === '/api/health' && req.method === 'GET') {
            brokerClient.getBrokerHealth().then(health => {
                sendJSON(res, 200, { success: true, health });
            }).catch(err => {
                sendError(res, 500, 'Failed to fetch health: ' + err.message);
            });
            return;
        }
        
        if (pathname === '/api/cluster-topology' && req.method === 'GET') {
            brokerClient.getClusterTopology().then(topology => {
                sendJSON(res, 200, { success: true, topology });
            }).catch(err => {
                sendError(res, 500, 'Failed to fetch cluster topology: ' + err.message);
            });
            return;
        }
        
        // ===== BROKER ACTION ENDPOINTS (POST) =====
        
        // Restart a broker
        if (pathname.match(/^\/api\/brokers\/[a-zA-Z0-9_\-]+\/restart$/) && req.method === 'POST') {
            const brokerId = pathname.split('/')[3];
            brokerClient.restartBroker(brokerId).then(result => {
                sendJSON(res, 200, { success: true, message: 'Broker restart initiated', brokerId, result });
            }).catch(err => {
                sendError(res, 500, 'Failed to restart broker: ' + err.message);
            });
            return;
        }
        
        // Pause a broker
        if (pathname.match(/^\/api\/brokers\/[a-zA-Z0-9_\-]+\/pause$/) && req.method === 'POST') {
            const brokerId = pathname.split('/')[3];
            brokerClient.pauseBroker(brokerId).then(result => {
                sendJSON(res, 200, { success: true, message: 'Broker paused', brokerId, result });
            }).catch(err => {
                sendError(res, 500, 'Failed to pause broker: ' + err.message);
            });
            return;
        }
        
        // ===== STREAM CREATION ENDPOINT (POST) =====
        
        if (pathname === '/api/streams' && req.method === 'POST') {
            parseRequestBody(req).then(body => {
                if (!body.name) {
                    sendError(res, 400, 'Stream name required');
                    return;
                }
                brokerClient.createStream(body).then(stream => {
                    sendJSON(res, 201, { success: true, message: 'Stream created', stream });
                }).catch(err => {
                    sendError(res, 500, 'Failed to create stream: ' + err.message);
                });
            });
            return;
        }
        
        // ===== QUEUE ACTION ENDPOINT (POST) =====
        
        if (pathname === '/api/queue-action' && req.method === 'POST') {
            parseRequestBody(req).then(body => {
                if (!body.queueName || !body.action) {
                    sendError(res, 400, 'queueName and action required');
                    return;
                }
                brokerClient.performQueueAction(body.queueName, body.action).then(result => {
                    sendJSON(res, 200, { success: true, message: 'Queue action completed', action: body.action, result });
                }).catch(err => {
                    sendError(res, 500, 'Failed to perform queue action: ' + err.message);
                });
            });
            return;
        }
        
        // ===== CLUSTER REBALANCE ENDPOINT (POST) =====
        
        if (pathname === '/api/cluster/rebalance' && req.method === 'POST') {
            brokerClient.rebalanceCluster().then(result => {
                sendJSON(res, 200, { success: true, message: 'Cluster rebalancing initiated', result });
            }).catch(err => {
                sendError(res, 500, 'Failed to rebalance cluster: ' + err.message);
            });
            return;
        }

        // ===== QUIC PSK MANAGEMENT ENDPOINTS =====

        // GET PSK list for tenant
        if (pathname.match(/^\/api\/quic\/psks\/tenant\/[a-zA-Z0-9_-]+$/) && req.method === 'GET') {
            const tenantId = pathname.split('/')[5];
            const tenant = storage.getTenant(tenantId);
            
            if (!tenant) {
                sendError(res, 404, 'Tenant not found');
                return;
            }

            // Return PSKs registered for this tenant
            const pskList = Array.from(quicPskManager.psks.entries())
                .filter(([_, psk]) => psk.tenantId === tenantId)
                .map(([identity, psk]) => ({
                    identity,
                    pskId: psk.pskId,
                    clientId: psk.clientId,
                    createdAt: psk.createdAt,
                    isActive: psk.isActive
                }));

            sendJSON(res, 200, { 
                success: true, 
                tenantId,
                psks: pskList,
                count: pskList.length
            });
            return;
        }

        // POST - Generate new PSK for client
        if (pathname === '/api/quic/psks' && req.method === 'POST') {
            parseRequestBody(req).then(body => {
                if (!body.tenantId || !body.clientId) {
                    sendError(res, 400, 'tenantId and clientId required');
                    return;
                }

                const tenant = storage.getTenant(body.tenantId);
                if (!tenant) {
                    sendError(res, 404, 'Tenant not found');
                    return;
                }

                // Generate PSK secret
                const secret = QuicPskManager.generateSecret();
                
                // Register PSK
                const psk = quicPskManager.registerPsk(body.tenantId, body.clientId, secret);

                sendJSON(res, 201, {
                    success: true,
                    psk: {
                        identity: psk.identity,
                        pskId: psk.pskId,
                        secret: secret,  // Return secret only at creation time
                        clientId: body.clientId,
                        tenantId: body.tenantId,
                        createdAt: new Date()
                    },
                    message: 'PSK generated successfully. Store the secret securely - it cannot be recovered!'
                });
            });
            return;
        }

        // DELETE - Revoke PSK
        if (pathname.match(/^\/api\/quic\/psks\/[a-zA-Z0-9_:-]+$/) && req.method === 'DELETE') {
            const identity = pathname.split('/').slice(4).join('/');
            
            const revoked = quicPskManager.revokePsk(decodeURIComponent(identity));
            
            if (!revoked) {
                sendError(res, 404, 'PSK not found');
                return;
            }

            sendJSON(res, 200, {
                success: true,
                message: 'PSK revoked successfully',
                identity: decodeURIComponent(identity)
            });
            return;
        }

        // GET - QUIC connection statistics
        if (pathname === '/api/quic/stats' && req.method === 'GET') {
            const stats = quicConnManager.getStats();
            
            sendJSON(res, 200, {
                success: true,
                quic: {
                    protocol: 'QUIC 1.0 (RFC 9000)',
                    authentication: 'Pre-Shared Key (PSK) with TLS 1.3',
                    ...stats
                }
            });
            return;
        }

        // GET - QUIC PSK Authentication status
        if (pathname === '/api/quic/status' && req.method === 'GET') {
            const pskStats = quicPskManager.getStats();
            
            sendJSON(res, 200, {
                success: true,
                quic: {
                    enabled: true,
                    protocol: 'QUIC with PSK authentication',
                    port: QUIC_PORT,
                    pskStats,
                    connectionStats: quicConnManager.getStats()
                }
            });
            return;
        }
        
        // 404 for unknown API routes
        sendError(res, 404, 'API route not found');
        
    } catch (err) {
        console.error('API Error:', err);
        sendError(res, 500, err.message);
    }
}

// Main server
const server = http.createServer((req, res) => {
    const pathname = url.parse(req.url).pathname;
    
    // Handle API routes
    if (pathname.startsWith('/api/')) {
        handleAPI(pathname, req, res);
        return;
    }
    
    // Default to dashboard.html for root
    let filePath;
    if (pathname === '/' || pathname === '') {
        filePath = path.join(__dirname, 'dashboard.html');
    } else {
        filePath = path.join(__dirname, pathname);
    }
    
    // Security: prevent directory traversal attacks
    const realPath = path.resolve(filePath);
    const baseDir = path.resolve(__dirname);
    
    if (!realPath.startsWith(baseDir)) {
        res.writeHead(403, { 'Content-Type': 'text/plain' });
        res.end('403 Forbidden');
        return;
    }
    
    // Get file extension
    const extname = path.extname(filePath).toLowerCase();
    const contentType = mimeTypes[extname] || 'application/octet-stream';
    
    // Read and serve file
    fs.readFile(filePath, (err, data) => {
        if (err) {
            if (err.code === 'ENOENT') {
                res.writeHead(404, { 'Content-Type': 'text/html' });
                res.end(`
                    <!DOCTYPE html>
                    <html>
                    <head><title>404 Not Found</title></head>
                    <body>
                        <h1>404 - File Not Found</h1>
                        <p>The file <code>${pathname}</code> could not be found.</p>
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
        
        // Set cache headers
        if (['.js', '.css', '.png', '.jpg', '.gif', '.svg', '.ico'].includes(extname)) {
            res.setHeader('Cache-Control', 'public, max-age=3600');
        }
        
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
    console.log('   ✓ Multi-User Support');
    console.log('   ✓ Encrypted Tenant Storage');
    console.log('   ✓ Backup & Restore\n');
    console.log('📁 Storage Details:');
    const stats = storage.getStats();
    console.log(`   📦 Tenants: ${stats.tenants}`);
    console.log(`   🔑 Secrets: ${stats.secrets}`);
    console.log(`   💾 Storage Dir: ${stats.storage_dir}`);
    console.log(`   🔐 Encryption: AES-256-GCM\n`);
    console.log('🛑 To stop server: Press Ctrl+C\n');
});

// Error handling
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

// Uncaught exceptions
process.on('uncaughtException', (err) => {
    console.error('❌ Uncaught Exception:', err.message);
    process.exit(1);
});

module.exports = { server, storage };
