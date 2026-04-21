const express = require('express');
const cors = require('cors');
const http = require('http');
const path = require('path');
const fs = require('fs');
const axios = require('axios');

const app = express();
const PORT = 3000;
const ADMIN_API_URL = 'http://127.0.0.1:8080';

// Middleware
app.use(cors());
app.use(express.json());
app.use(express.static(path.join(__dirname, '.')));

// ==================== AXIOS CONFIG ====================
const apiClient = axios.create({
    baseURL: ADMIN_API_URL,
    headers: {
        'Content-Type': 'application/json'
    },
    timeout: 5000
});

// ==================== HELPER FUNCTIONS ====================
async function callAdminAPI(method, endpoint, data = null) {
    try {
        const config = {
            method: method.toLowerCase(),
            url: endpoint,
            headers: {
                'Content-Type': 'application/json'
            }
        };
        
        if (data && ['post', 'put'].includes(method.toLowerCase())) {
            config.data = data;
        }
        
        const response = await axios(ADMIN_API_URL + endpoint, config);
        return response.data;
    } catch (error) {
        console.error(`API Error [${method} ${endpoint}]:`, error.message);
        throw error;
    }
}

// ==================== AUTHENTICATION MIDDLEWARE ====================
function authenticateRequest(req, res, next) {
    const apiKey = req.headers['x-api-key'];
    const authHeader = req.headers['authorization'];

    if (apiKey === 'admin-key' || authHeader === 'Bearer admin-token') {
        next();
    } else {
        res.status(401).json({ error: 'Unauthorized' });
    }
}

// ==================== ERROR HANDLER ====================
function handleError(res, error, defaultMessage = 'Failed to fetch data') {
    console.error('Error details:', error.message);
    const status = error.response?.status || 500;
    const message = error.response?.data?.message || error.message || defaultMessage;
    res.status(status).json({
        status: 'error',
        message: message,
        error: error.message
    });
}

// ==================== ROUTES ====================

// Serve dashboard HTML
app.get('/', (req, res) => {
    res.sendFile(path.join(__dirname, 'admin-dashboard.html'));
});

// Health check
app.get('/health', async (req, res) => {
    try {
        const response = await axios.get(`${ADMIN_API_URL}/health`, {
            timeout: 3000
        });
        res.json(response.data);
    } catch (error) {
        handleError(res, error, 'Admin API is unreachable');
    }
});

// Get server information
app.get('/api/v1/server/info', authenticateRequest, async (req, res) => {
    try {
        const response = await axios.get(`${ADMIN_API_URL}/health`);
        res.json({
            version: response.data.version || '0.1.16',
            uptime: response.data.uptime || 'Unknown',
            memory: 'Real-time',
            cpu: 'Real-time',
            status: response.data.status || 'unknown',
            endpoints: 20,
            timestamp: new Date().toISOString()
        });
    } catch (error) {
        handleError(res, error, 'Failed to fetch server info');
    }
});

// Get all tenants
app.get('/api/v1/tenants', async (req, res) => {
    try {
        const response = await axios.get(`${ADMIN_API_URL}/api/v1/tenants`);
        res.json(response.data);
    } catch (error) {
        handleError(res, error, 'Failed to fetch tenants');
    }
});

// Get specific tenant
app.get('/api/v1/tenants/:id', async (req, res) => {
    try {
        const response = await axios.get(`${ADMIN_API_URL}/api/v1/tenants/${req.params.id}`);
        res.json(response.data);
    } catch (error) {
        handleError(res, error, 'Tenant not found');
    }
});

// Create tenant
app.post('/api/v1/tenants', async (req, res) => {
    try {
        const response = await axios.post(`${ADMIN_API_URL}/api/v1/tenants`, req.body);
        res.status(201).json(response.data);
    } catch (error) {
        handleError(res, error, 'Failed to create tenant');
    }
});

// Update tenant
app.put('/api/v1/tenants/:id', async (req, res) => {
    try {
        const response = await axios.put(`${ADMIN_API_URL}/api/v1/tenants/${req.params.id}`, req.body);
        res.json(response.data);
    } catch (error) {
        handleError(res, error, 'Failed to update tenant');
    }
});

// Delete tenant
app.delete('/api/v1/tenants/:id', async (req, res) => {
    try {
        const response = await axios.delete(`${ADMIN_API_URL}/api/v1/tenants/${req.params.id}`);
        res.json(response.data);
    } catch (error) {
        handleError(res, error, 'Failed to delete tenant');
    }
});

// Get metrics
app.get('/api/v1/metrics', async (req, res) => {
    try {
        const response = await axios.get(`${ADMIN_API_URL}/api/v1/metrics`);
        res.json(response.data);
    } catch (error) {
        handleError(res, error, 'Failed to fetch metrics');
    }
});

// Get stats
app.get('/api/v1/stats', async (req, res) => {
    try {
        const response = await axios.get(`${ADMIN_API_URL}/api/v1/stats`);
        res.json(response.data);
    } catch (error) {
        handleError(res, error, 'Failed to fetch statistics');
    }
});

// Get endpoints
app.get('/api/v1/endpoints', async (req, res) => {
    try {
        const response = await axios.get(`${ADMIN_API_URL}/api/v1/endpoints`);
        res.json(response.data);
    } catch (error) {
        handleError(res, error, 'Failed to fetch endpoints');
    }
});

// Send message
app.post('/api/v1/send', async (req, res) => {
    try {
        const response = await axios.post(`${ADMIN_API_URL}/api/v1/send`, req.body);
        res.json(response.data);
    } catch (error) {
        handleError(res, error, 'Failed to send message');
    }
});

// Consume message
app.post('/api/v1/consume', async (req, res) => {
    try {
        const response = await axios.post(`${ADMIN_API_URL}/api/v1/consume`, req.body);
        res.json(response.data);
    } catch (error) {
        handleError(res, error, 'Failed to consume message');
    }
});

// Get configuration
app.get('/api/v1/config', async (req, res) => {
    try {
        const response = await axios.get(`${ADMIN_API_URL}/api/v1/config`);
        res.json(response.data);
    } catch (error) {
        handleError(res, error, 'Failed to fetch configuration');
    }
});

// Update configuration
app.post('/api/v1/config', async (req, res) => {
    try {
        const response = await axios.post(`${ADMIN_API_URL}/api/v1/config`, req.body);
        res.json(response.data);
    } catch (error) {
        handleError(res, error, 'Failed to update configuration');
    }
});

// Authentication
app.post('/api/v1/auth/login', async (req, res) => {
    try {
        const response = await axios.post(`${ADMIN_API_URL}/api/v1/auth/login`, req.body, {
            headers: { 'Content-Type': 'application/json' }
        });
        res.json(response.data);
    } catch (error) {
        handleError(res, error, 'Login failed');
    }
});

// Test endpoint
app.post('/api/v1/test', async (req, res) => {
    try {
        const response = await axios.post(`${ADMIN_API_URL}/api/v1/test`, req.body);
        res.json(response.data);
    } catch (error) {
        handleError(res, error, 'Test request failed');
    }
});

// Get tenant usage
app.get('/api/v1/tenants/:id/usage', async (req, res) => {
    try {
        const response = await axios.get(`${ADMIN_API_URL}/api/v1/tenants/${req.params.id}/usage`);
        res.json(response.data);
    } catch (error) {
        handleError(res, error, 'Failed to fetch tenant usage');
    }
});

// 404 handler
app.use((req, res) => {
    res.status(404).json({
        status: 'error',
        message: 'Endpoint not found',
        path: req.path,
        hint: 'This dashboard proxies requests to the Admin API at ' + ADMIN_API_URL
    });
});

// Error handler
app.use((err, req, res, next) => {
    console.error('Unhandled error:', err);
    res.status(500).json({
        status: 'error',
        message: 'Internal server error',
        error: process.env.NODE_ENV === 'development' ? err.message : undefined
    });
});

// Start server
http.createServer(app).listen(PORT, '127.0.0.1', () => {
    console.log(`
â•”â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•—
â•‘                                                        â•‘
â•‘   âš¡ FastDataBroker Admin Dashboard Server             â•‘
â•‘                                                        â•‘
â•‘   Status: Running âœ“                                    â•‘
â•‘   URL: http://127.0.0.1:${PORT}                        â•‘
â•‘   API Base: http://127.0.0.1:${PORT}/api/v1            â•‘
â•‘   Connected to: ${ADMIN_API_URL}                       â•‘
â•‘                                                        â•‘
â•‘   Demo Credentials:                                    â•‘
â•‘   Username: admin                                      â•‘
â•‘   Password: admin                                      â•‘
â•‘   API Key: admin-key                                   â•‘
â•‘                                                        â•‘
â•‘   ðŸ’¡ NOTE: Getting REAL DATA from Admin API            â•‘
â•‘                                                        â•‘
â•šâ•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
    `);
});

module.exports = app;

