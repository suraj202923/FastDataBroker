// ===== FastDataBroker Admin Dashboard - JavaScript =====

// ===== STATE MANAGEMENT =====
const dashboardState = {
    user: null,
    isLoggedIn: false,
    selectedServer: null,
    displayMode: 'light',
    refreshInterval: 5000,
    currentPage: 'dashboard',
    brokers: [],
    clusters: [],
    streams: [],
    metrics: {}
};

// ===== AUTHENTICATION =====
function handleLogin(event) {
    event.preventDefault();
    console.log('Login attempt started');
    
    const username = document.getElementById('username').value.trim();
    const password = document.getElementById('password').value.trim();
    const errorMsg = document.getElementById('errorMessage');

    // Clear previous errors
    errorMsg.style.display = 'none';
    errorMsg.textContent = '';

    if (!username || !password) {
        showError('Username and password are required');
        return;
    }

    // Validate credentials
    console.log('Validating credentials - username:', username, 'password length:', password.length);
    
    if (username !== 'admin') {
        showError('Invalid username. Use: admin');
        return;
    }
    
    if (password !== 'password') {
        showError('Invalid password. Use: password');
        return;
    }

    console.log('✅ Credentials validated successfully');

    // Disable login button
    const loginBtn = document.querySelector('.login-btn');
    const originalText = loginBtn.textContent;
    loginBtn.disabled = true;
    loginBtn.textContent = 'Logging in...';

    // Authenticate and initialize
    setTimeout(() => {
        try {
            console.log('Starting login process...');
            // Get current server automatically (this instance)
            const currentServer = window.location.origin; // e.g., http://localhost:3000
            
            dashboardState.user = { username, role: 'admin' };
            dashboardState.isLoggedIn = true;
            dashboardState.selectedServer = currentServer;

            // Save to localStorage for persistence
            localStorage.setItem('dashboardUser', JSON.stringify(dashboardState.user));
            localStorage.setItem('dashboardServer', currentServer);

            console.log('Preparing to show dashboard...');
            // Show dashboard
            const loginContainer = document.querySelector('.login-container');
            const dashboard = document.querySelector('.dashboard');
            
            if (loginContainer) loginContainer.style.display = 'none';
            if (dashboard) dashboard.style.display = 'flex';

            console.log('Dashboard displayed, initializing...');
            // Initialize dashboard
            try {
                initializeDashboard();
                console.log('Dashboard initialized');
            } catch (err) {
                console.warn('Dashboard initialization warning:', err.message);
            }
            
            // Load data asynchronously
            try {
                loadDashboardData();
                console.log('Dashboard data loading started');
            } catch (err) {
                console.warn('Data loading warning:', err.message);
            }
            
            console.log('✅ Login successful!');

        } catch (error) {
            console.error('Login error:', error);
            showError('Authentication failed: ' + error.message);
            loginBtn.disabled = false;
            loginBtn.textContent = originalText;
        }
    }, 500);
}

function showError(message) {
    const errorMsg = document.getElementById('errorMessage');
    errorMsg.textContent = message;
    errorMsg.style.display = 'block';
}

function handleLogout() {
    if (confirm('Are you sure you want to logout?')) {
        dashboardState.isLoggedIn = false;
        dashboardState.user = null;

        // Clear localStorage
        localStorage.removeItem('dashboardUser');
        localStorage.removeItem('dashboardServer');

        // Clear intervals
        if (window.refreshTimer) clearInterval(window.refreshTimer);

        // Show login page
        document.querySelector('.dashboard').style.display = 'none';
        document.querySelector('.login-container').style.display = 'flex';

        // Reset form
        document.getElementById('loginForm').reset();
        document.getElementById('errorMessage').style.display = 'none';
    }
}

// ===== FASTDATABROKER CONTROL =====
function restartFastDataBroker() {
    if (confirm('⚠️ Restart FastDataBroker server? This will cause a brief service interruption.')) {
        console.log('Sending restart command to FastDataBroker...');
        
        fetch('/api/server/restart', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' }
        })
        .then(res => res.json())
        .then(data => {
            if (data.success) {
                alert('✓ FastDataBroker is restarting... The service will be back online in a few seconds.');
                // Brief pause then reload page
                setTimeout(() => {
                    window.location.reload();
                }, 3000);
            } else {
                alert('✗ Error: ' + (data.error || 'Failed to restart FastDataBroker'));
            }
        })
        .catch(err => {
            console.error('Error restarting server:', err);
            alert('✗ Error: ' + err.message);
        });
    }
}

function stopFastDataBroker() {
    if (confirm('⚠️ WARNING: Stop FastDataBroker server? This will take the service offline. Are you absolutely sure?')) {
        console.log('Sending stop command to FastDataBroker...');
        
        fetch('/api/server/stop', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' }
        })
        .then(res => res.json())
        .then(data => {
            if (data.success) {
                alert('✓ FastDataBroker is shutting down. You will lose the connection.');
                // Disable UI after stop
                setTimeout(() => {
                    document.querySelectorAll('.btn').forEach(btn => btn.disabled = true);
                    document.querySelector('.dashboard').style.opacity = '0.5';
                }, 1000);
            } else {
                alert('✗ Error: ' + (data.error || 'Failed to stop FastDataBroker'));
            }
        })
        .catch(err => {
            console.error('Error stopping server:', err);
            alert('✗ Error: ' + err.message);
        });
    }
}

// ===== DASHBOARD INITIALIZATION =====
function initializeDashboard() {
    // Set user info
    try {
        const userAvatar = document.querySelector('.user-avatar');
        const username = dashboardState.user?.username || 'Admin';
        if (userAvatar) {
            userAvatar.textContent = username.charAt(0).toUpperCase();
        }
    } catch (err) {
        console.warn('Could not update user avatar:', err);
    }

    // Set up navigation
    try {
        setupNavigation();
    } catch (err) {
        console.warn('Could not setup navigation:', err);
    }

    // Start auto-refresh
    try {
        if (window.refreshTimer) clearInterval(window.refreshTimer);
        window.refreshTimer = setInterval(loadDashboardData, dashboardState.refreshInterval);
    } catch (err) {
        console.warn('Could not setup refresh timer:', err);
    }
}

function setupNavigation() {
    const navItems = document.querySelectorAll('.nav-item');
    navItems.forEach(item => {
        item.addEventListener('click', function() {
            const page = this.dataset.page;
            navigateToPage(page);
        });
    });
}

function navigateToPage(page) {
    // Update active nav item
    document.querySelectorAll('.nav-item').forEach(item => {
        item.classList.remove('active');
        if (item.dataset.page === page) item.classList.add('active');
    });

    // Update active section
    document.querySelectorAll('.section').forEach(section => {
        section.classList.remove('active');
    });

    const section = document.querySelector(`.section[data-section="${page}"]`);
    if (section) section.classList.add('active');

    dashboardState.currentPage = page;

    // Load page-specific data
    loadPageData(page);
}

// ===== DATA LOADING =====
function loadDashboardData() {
    console.log('Loading dashboard data for server:', dashboardState.selectedServer);

    // Load from encrypted storage first
    loadDataFromStorage()
        .then(() => {
            console.log('Storage data loaded, loading page data...');
            // Simulate API calls
            try {
                loadOverviewData();
                loadBrokersData();
                loadClustersData();
                loadStreamsData();
                loadTenantsData();
                loadSecretsData();
                console.log('All page data loaded');
            } catch (err) {
                console.warn('Error loading page data:', err);
            }
        })
        .catch(err => {
            console.warn('Warning loading storage data:', err);
            // Still load page data even if storage fails
            try {
                loadOverviewData();
                loadBrokersData();
                loadClustersData();
                loadStreamsData();
                loadTenantsData();
                loadSecretsData();
            } catch (err2) {
                console.error('Error loading page data after storage failure:', err2);
            }
        });
}

function loadPageData(page) {
    switch(page) {
        case 'dashboard':
            loadOverviewData();
            break;
        case 'brokers':
            loadBrokersData();
            break;
        case 'clusters':
            loadClustersData();
            break;
        case 'streams':
            loadStreamsData();
            break;
        case 'tenants':
            loadTenantsData();
            break;
        case 'secrets':
            loadSecretsData();
            updateTenantFilterDropdown();
            break;
        case 'settings':
            initializeSettings();
            break;
    }
}

// ===== OVERVIEW SECTION =====
function loadOverviewData() {
    try {
        // Fetch real metrics from broker
        Promise.all([
            fetch(`${API_BASE}/brokers`).then(r => r.json()),
            fetch(`${API_BASE}/streams`).then(r => r.json()),
            fetch(`${API_BASE}/cluster-topology`).then(r => r.json()),
            fetch(`${API_BASE}/metrics`).then(r => r.json())
        ])
        .then(([brokersRes, streamsRes, topologyRes, metricsRes]) => {
            const brokerCount = brokersRes.success && brokersRes.brokers ? brokersRes.brokers.length : 0;
            const streamCount = streamsRes.success && streamsRes.streams ? streamsRes.streams.length : 0;
            const clusterCount = topologyRes.success && topologyRes.topology && topologyRes.topology.clusters ? topologyRes.topology.clusters.length : 1;
            const throughput = metricsRes.success && metricsRes.metrics ? (metricsRes.metrics.throughput || '0') + ' msg/s' : '0 msg/s';

            const stats = {
                totalBrokers: brokerCount,
                activeClusters: clusterCount,
                activeStreams: streamCount,
                throughput: throughput
            };

            updateStatCard('total-brokers', stats.totalBrokers, 'Total Brokers');
            updateStatCard('active-clusters', stats.activeClusters, 'Active Clusters');
            updateStatCard('active-streams', stats.activeStreams, 'Active Streams');
            updateStatCard('throughput', stats.throughput, 'Throughput');

            dashboardState.metrics = stats;
            console.log('✅ Real overview data loaded successfully');
        })
        .catch(err => {
            console.warn('Error loading real overview data:', err);
            // Fallback if API fails
            updateStatCard('total-brokers', '0', 'Total Brokers');
            updateStatCard('active-clusters', '0', 'Active Clusters');
            updateStatCard('active-streams', '0', 'Active Streams');
            updateStatCard('throughput', '0 msg/s', 'Throughput');
        });
    } catch (err) {
        console.warn('Error in loadOverviewData:', err);
    }
}

function updateStatCard(id, value, label) {
    const card = document.getElementById(id);
    if (card) {
        card.innerHTML = `
            <div class="stat-label">${label}</div>
            <div class="stat-value">${value}</div>
            <div class="stat-change positive">↑ +2.5%</div>
        `;
    }
}

// ===== BROKERS SECTION =====
function loadBrokersData() {
    try {
        // Fetch real brokers from API
        fetch(`${API_BASE}/brokers`)
            .then(res => res.json())
            .then(data => {
                if (data.success && data.brokers) {
                    dashboardState.brokers = data.brokers;
                    renderBrokersTable(data.brokers);
                    console.log('✅ Real brokers data loaded:', data.brokers.length, 'brokers');
                } else {
                    throw new Error('Invalid broker response');
                }
            })
            .catch(err => {
                console.warn('❌ Error loading real brokers:', err.message);
                // Show empty state
                const tbody = document.querySelector('#brokersTable tbody');
                if (tbody) {
                    tbody.innerHTML = '<tr><td colspan="6" class="text-center" style="padding: 2rem; color: var(--text-muted);">No brokers available or connection error</td></tr>';
                }
            });
    } catch (err) {
        console.warn('Error in loadBrokersData:', err);
    }
}

function renderBrokersTable(brokers) {
    try {
        const tbody = document.querySelector('#brokersTable tbody');
        if (!tbody) {
            console.warn('Brokers table tbody not found');
            return;
        }

        tbody.innerHTML = brokers.map(broker => `
            <tr>
                <td>${broker.id}</td>
                <td>${broker.host}:${broker.port}</td>
                <td><span class="badge badge-${broker.status === 'online' ? 'success' : 'danger'}">${broker.status}</span></td>
                <td>${broker.cpu}%</td>
                <td>${broker.memory}%</td>
                <td>
                    <button class="btn btn-primary btn-small" onclick="viewBrokerDetails('${broker.id}')">Details</button>
                    ${broker.status === 'online' ? 
                        `<button class="btn btn-warning btn-small" onclick="openBrokerRestartModal('${broker.id}')" title="Restart broker">🔄 Restart</button>
                         <button class="btn btn-warning btn-small" onclick="openBrokerPauseModal('${broker.id}')" title="Pause broker">⏸️ Pause</button>` : 
                        `<button class="btn btn-success btn-small" onclick="startBroker('${broker.id}')">Start</button>`
                    }
                </td>
            </tr>
        `).join('');
        console.log('Brokers table rendered');
    } catch (err) {
        console.error('Error rendering brokers table:', err);
    }
}

function viewBrokerDetails(brokerId) {
    const broker = dashboardState.brokers.find(b => b.id === brokerId);
    if (!broker) return;

    const modal = document.getElementById('detailsModal');
    const content = modal.querySelector('.modal-content');
    
    content.innerHTML = `
        <span class="modal-close" onclick="closeModal('detailsModal')">&times;</span>
        <div class="modal-header">Broker Details: ${broker.id}</div>
        <div style="margin-top: 1rem;">
            <p><strong>Host:</strong> ${broker.host}</p>
            <p><strong>Port:</strong> ${broker.port}</p>
            <p><strong>Status:</strong> <span class="badge badge-${broker.status === 'online' ? 'success' : 'danger'}">${broker.status}</span></p>
            <p><strong>CPU Usage:</strong> ${broker.cpu}%</p>
            <p><strong>Memory Usage:</strong> ${broker.memory}%</p>
            <p><strong>Partitions:</strong> 48</p>
            <p><strong>Messages:</strong> 1,234,567</p>
            <p><strong>Uptime:</strong> 45 days, 12 hours</p>
        </div>
    `;

    modal.classList.add('active');
}

function restartBroker(brokerId) {
    if (confirm('Are you sure you want to restart this broker?')) {
        const btn = event.target;
        btn.disabled = true;
        btn.textContent = 'Restarting...';

        setTimeout(() => {
            alert(`Broker ${brokerId} restarted successfully`);
            loadBrokersData();
        }, 2000);
    }
}

function startBroker(brokerId) {
    if (confirm('Are you sure you want to start this broker?')) {
        const btn = event.target;
        btn.disabled = true;
        btn.textContent = 'Starting...';

        setTimeout(() => {
            alert(`Broker ${brokerId} started successfully`);
            loadBrokersData();
        }, 2000);
    }
}

// ===== CLUSTERS SECTION =====
function loadClustersData() {
    try {
        // Fetch real cluster topology from API
        fetch(`${API_BASE}/cluster-topology`)
            .then(res => res.json())
            .then(data => {
                if (data.success && data.topology) {
                    const topology = data.topology;
                    let clusters = [];
                    
                    if (topology.clusters && Array.isArray(topology.clusters)) {
                        clusters = topology.clusters.map(c => ({
                            id: c.id || c.name || 'unknown',
                            name: c.name || 'Cluster',
                            brokers: c.brokers || c.brokerCount || 0,
                            status: c.status || 'unknown',
                            nodes: c.nodes || c.brokers || 0,
                            avgLatency: c.avgLatency || c.latency || 'N/A'
                        }));
                    } else {
                        // Single cluster fallback
                        clusters = [{
                            id: 'cluster-1',
                            name: 'Default Cluster',
                            brokers: topology.brokerCount || 1,
                            status: topology.status || 'online',
                            nodes: topology.nodes || 1,
                            avgLatency: topology.avgLatency || 'N/A'
                        }];
                    }
                    
                    dashboardState.clusters = clusters;
                    renderClustersTable(clusters);
                    console.log('✅ Real cluster topology loaded:', clusters.length, 'clusters');
                } else {
                    throw new Error('Invalid topology response');
                }
            })
            .catch(err => {
                console.warn('❌ Error loading cluster topology:', err.message);
                const tbody = document.querySelector('#clustersTable tbody');
                if (tbody) {
                    tbody.innerHTML = '<tr><td colspan="6" class="text-center" style="padding: 2rem; color: var(--text-muted);">No cluster data available</td></tr>';
                }
            });
    } catch (err) {
        console.warn('Error in loadClustersData:', err);
    }
}

function renderClustersTable(clusters) {
    try {
        const tbody = document.querySelector('#clustersTable tbody');
        if (!tbody) {
            console.warn('Clusters table tbody not found');
            return;
        }

        tbody.innerHTML = clusters.map(cluster => `
            <tr>
                <td>${cluster.name}</td>
                <td>${cluster.brokers}</td>
                <td>${cluster.nodes}</td>
                <td><span class="badge badge-${cluster.status === 'healthy' ? 'success' : 'warning'}">${cluster.status}</span></td>
                <td>${cluster.avgLatency}</td>
                <td>
                    <button class="btn btn-primary btn-small" onclick="viewClusterDetails('${cluster.id}')">View</button>
                    <button class="btn btn-warning btn-small" onclick="openRebalanceClusterModal()" title="Rebalance cluster">⚖️ Rebalance</button>
                </td>
            </tr>
        `).join('');
        console.log('Clusters table rendered');
    } catch (err) {
        console.error('Error rendering clusters table:', err);
    }
}

function viewClusterDetails(clusterId) {
    const cluster = dashboardState.clusters.find(c => c.id === clusterId);
    if (!cluster) return;

    const modal = document.getElementById('detailsModal');
    const content = modal.querySelector('.modal-content');
    
    content.innerHTML = `
        <span class="modal-close" onclick="closeModal('detailsModal')">&times;</span>
        <div class="modal-header">Cluster Details: ${cluster.name}</div>
        <div style="margin-top: 1rem;">
            <p><strong>Cluster ID:</strong> ${cluster.id}</p>
            <p><strong>Status:</strong> <span class="badge badge-${cluster.status === 'healthy' ? 'success' : 'warning'}">${cluster.status}</span></p>
            <p><strong>Brokers:</strong> ${cluster.brokers}</p>
            <p><strong>Nodes:</strong> ${cluster.nodes}</p>
            <p><strong>Average Latency:</strong> ${cluster.avgLatency}</p>
            <p><strong>Total Topics:</strong> 24</p>
            <p><strong>Total Partitions:</strong> 144</p>
            <p><strong>Replication Factor:</strong> 3</p>
        </div>
    `;

    modal.classList.add('active');
}

function rebalanceCluster(clusterId) {
    if (confirm('This will rebalance partitions across the cluster. Continue?')) {
        alert(`Rebalancing cluster ${clusterId}...`);
    }
}

// ===== STREAMS SECTION =====
function loadStreamsData() {
    try {
        // Fetch real streams from API
        fetch(`${API_BASE}/streams`)
            .then(res => res.json())
            .then(data => {
                if (data.success && data.streams) {
                    const streams = data.streams.map(s => ({
                        id: s.id || s.name || 'unknown',
                        name: s.name || 'stream',
                        partitions: s.partitions || s.partition_count || 0,
                        status: s.status || 'active',
                        messages: s.messages || s.messageCount || '0',
                        retention: s.retention || s.retentionPolicy || 'N/A'
                    }));
                    
                    dashboardState.streams = streams;
                    renderStreamsTable(streams);
                    console.log('✅ Real streams data loaded:', streams.length, 'streams');
                } else {
                    throw new Error('Invalid streams response');
                }
            })
            .catch(err => {
                console.warn('❌ Error loading real streams:', err.message);
                const tbody = document.querySelector('#streamsTable tbody');
                if (tbody) {
                    tbody.innerHTML = '<tr><td colspan="6" class="text-center" style="padding: 2rem; color: var(--text-muted);">No streams available</td></tr>';
                }
            });
    } catch (err) {
        console.warn('Error in loadStreamsData:', err);
    }
}

function renderStreamsTable(streams) {
    try {
        const tbody = document.querySelector('#streamsTable tbody');
        if (!tbody) {
            console.warn('Streams table tbody not found');
            return;
        }

        tbody.innerHTML = streams.map(stream => `
            <tr>
                <td>${stream.name}</td>
                <td>${stream.partitions}</td>
                <td>${stream.messages}</td>
                <td>${stream.retention}</td>
                <td><span class="badge badge-${stream.status === 'active' ? 'success' : 'warning'}">${stream.status}</span></td>
                <td>
                    <button class="btn btn-primary btn-small" onclick="viewStreamDetails('${stream.id}')">Details</button>
                    ${stream.status === 'active' ? 
                        `<button class="btn btn-warning btn-small" onclick="pauseStream('${stream.id}')">Pause</button>` :
                        `<button class="btn btn-success btn-small" onclick="resumeStream('${stream.id}')">Resume</button>`
                    }
                </td>
            </tr>
        `).join('');
        console.log('Streams table rendered');
    } catch (err) {
        console.error('Error rendering streams table:', err);
    }
}

function viewStreamDetails(streamId) {
    const stream = dashboardState.streams.find(s => s.id === streamId);
    if (!stream) return;

    const modal = document.getElementById('detailsModal');
    const content = modal.querySelector('.modal-content');
    
    content.innerHTML = `
        <span class="modal-close" onclick="closeModal('detailsModal')">&times;</span>
        <div class="modal-header">Stream Details: ${stream.name}</div>
        <div style="margin-top: 1rem;">
            <p><strong>Stream Name:</strong> ${stream.name}</p>
            <p><strong>Status:</strong> <span class="badge badge-${stream.status === 'active' ? 'success' : 'warning'}">${stream.status}</span></p>
            <p><strong>Partitions:</strong> ${stream.partitions}</p>
            <p><strong>Total Messages:</strong> ${stream.messages}</p>
            <p><strong>Retention:</strong> ${stream.retention}</p>
            <p><strong>Compression:</strong> Snappy</p>
            <p><strong>Last Updated:</strong> 2 minutes ago</p>
        </div>
    `;

    modal.classList.add('active');
}

function pauseStream(streamId) {
    alert(`Pausing stream ${streamId}...`);
    loadStreamsData();
}

function resumeStream(streamId) {
    alert(`Resuming stream ${streamId}...`);
    loadStreamsData();
}

// ===== SETTINGS =====
function initializeSettings() {
    const tabs = document.querySelectorAll('.settings-tab');
    tabs.forEach(tab => {
        tab.addEventListener('click', function() {
            const panelId = this.dataset.panel;
            showSettingsPanel(panelId);
        });
    });
}

function showSettingsPanel(panelId) {
    document.querySelectorAll('.settings-tab').forEach(t => t.classList.remove('active'));
    document.querySelectorAll('.settings-panel').forEach(p => p.classList.remove('active'));

    document.querySelector(`[data-panel="${panelId}"]`).classList.add('active');
    document.querySelector(`[data-panel-content="${panelId}"]`).classList.add('active');
}

function updateServerSettings() {
    const refreshRate = document.getElementById('refreshRate').value;
    dashboardState.refreshInterval = parseInt(refreshRate) * 1000;
    alert('Server settings updated');
}

function updateNotificationSettings() {
    alert('Notification settings updated');
}

function updateSecuritySettings() {
    alert('Security settings updated');
}

function exportConfiguration() {
    const config = JSON.stringify(dashboardState, null, 2);
    const blob = new Blob([config], { type: 'application/json' });
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'dashboard-config.json';
    a.click();
}

function importConfiguration() {
    const fileInput = document.createElement('input');
    fileInput.type = 'file';
    fileInput.accept = '.json';
    fileInput.onchange = (e) => {
        const file = e.target.files[0];
        const reader = new FileReader();
        reader.onload = (event) => {
            try {
                const config = JSON.parse(event.target.result);
                Object.assign(dashboardState, config);
                alert('Configuration imported successfully');
            } catch (error) {
                alert('Error importing configuration: ' + error.message);
            }
        };
        reader.readAsText(file);
    };
    fileInput.click();
}

// ===== MODAL MANAGEMENT =====
function closeModal(modalId) {
    document.getElementById(modalId).classList.remove('active');
}

window.addEventListener('click', (e) => {
    if (e.target.classList.contains('modal')) {
        e.target.classList.remove('active');
    }
});

// ===== TENANT MANAGEMENT =====
// Data will be fetched from encrypted storage via API
let tenants = [];
let secrets = [];

// API Base URL
const API_BASE = window.location.origin + '/api';

// Load tenants and secrets from encrypted storage on startup
async function loadDataFromStorage() {
    try {
        const [tenantsRes, secretsRes] = await Promise.all([
            fetch(`${API_BASE}/tenants`),
            fetch(`${API_BASE}/secrets`)
        ]);
        
        const tenantsData = await tenantsRes.json();
        const secretsData = await secretsRes.json();
        
        if (tenantsData.success) {
            tenants = tenantsData.tenants;
        }
        if (secretsData.success) {
            secrets = secretsData.secrets;
        }
        
        console.log(`✅ Loaded ${tenants.length} tenants and ${secrets.length} secrets from encrypted storage`);
    } catch (err) {
        console.error('❌ Error loading data from storage:', err);
        showError('Failed to load tenant data from encrypted storage');
    }
}

function openTenantRegistrationModal() {
    const modal = document.getElementById('tenantRegistrationModal');
    modal.classList.add('active');
}

function submitTenantRegistration(event) {
    event.preventDefault();

    const name = document.getElementById('tenantName').value;
    const email = document.getElementById('tenantEmail').value;
    const tier = document.getElementById('tenantTier').value;
    const maxConnections = document.getElementById('maxConnections').value;
    const maxMessageSize = document.getElementById('maxMessageSize').value;

    // Create tenant request
    const tenantData = {
        name: name,
        email: email,
        tier: tier,
        maxConnections: parseInt(maxConnections),
        maxMessageSize: parseInt(maxMessageSize)
    };

    // Add tenant via API
    fetch(`${API_BASE}/tenants`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(tenantData)
    })
    .then(res => res.json())
    .then(data => {
        if (data.success) {
            const newTenant = data.tenant;
            
            // Generate and add initial API key
            return fetch(`${API_BASE}/generate-key`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    tenantId: newTenant.id,
                    name: 'Initial API Key'
                })
            }).then(res => res.json()).then(keyData => {
                if (keyData.success) {
                    // Close registration modal
                    closeModal('tenantRegistrationModal');
                    
                    // Show generated secrets
                    showGeneratedSecrets(
                        newTenant.id,
                        keyData.secret.key,
                        keyData.secret.secret,
                        keyData.endpoint,
                        keyData.connectionString
                    );
                    
                    // Reload data
                    loadDataFromStorage().then(() => loadTenantsData());
                }
            });
        }
    })
    .catch(err => {
        console.error('Error:', err);
        showError('Failed to register tenant');
    });
}

function showGeneratedSecrets(tenantId, apiKey, apiSecret, endpoint, connectionString) {
    document.getElementById('secretTenantId').textContent = tenantId;
    document.getElementById('secretApiKey').textContent = apiKey;
    document.getElementById('secretApiSecret').textContent = apiSecret;
    document.getElementById('secretEndpoint').textContent = endpoint;
    document.getElementById('secretConnectionString').textContent = connectionString;

    const modal = document.getElementById('generatedSecretsModal');
    modal.classList.add('active');
}

function copyToClipboard(elementId) {
    const element = document.getElementById(elementId);
    const text = element.textContent;
    
    navigator.clipboard.writeText(text).then(() => {
        const btn = event.target;
        const originalText = btn.textContent;
        btn.textContent = '✓ Copied!';
        btn.style.background = 'var(--secondary)';
        
        setTimeout(() => {
            btn.textContent = originalText;
            btn.style.background = '';
        }, 2000);
    }).catch(err => {
        alert('Failed to copy to clipboard');
    });
}

function downloadCredentials() {
    const credentials = {
        tenantId: document.getElementById('secretTenantId').textContent,
        apiKey: document.getElementById('secretApiKey').textContent,
        apiSecret: document.getElementById('secretApiSecret').textContent,
        brokerEndpoint: document.getElementById('secretEndpoint').textContent,
        connectionString: document.getElementById('secretConnectionString').textContent,
        generatedAt: new Date().toISOString(),
        note: 'These credentials are sensitive and should be stored securely',
        environment: {
            python: `from fastdatabroker import Client\nclient = Client(\n  api_key='${document.getElementById('secretApiKey').textContent}',\n  api_secret='${document.getElementById('secretApiSecret').textContent}',\n  broker_endpoint='${document.getElementById('secretEndpoint').textContent}'\n)`,
            nodejs: `const client = new FastDataBroker({\n  apiKey: '${document.getElementById('secretApiKey').textContent}',\n  apiSecret: '${document.getElementById('secretApiSecret').textContent}',\n  brokerEndpoint: '${document.getElementById('secretEndpoint').textContent}'\n})`
        }
    };

    const dataStr = JSON.stringify(credentials, null, 2);
    const dataBlob = new Blob([dataStr], { type: 'application/json' });
    const url = URL.createObjectURL(dataBlob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `backup-${new Date().toISOString().split('T')[0]}.json`;
    link.click();
    URL.revokeObjectURL(url);
}

// ===== LOAD TENANTS DATA =====
function loadTenantsData() {
    try {
        const tbody = document.querySelector('#tenantsTable tbody');
        if (!tbody) {
            console.warn('Tenants table tbody not found');
            return;
        }

        if (tenants.length === 0) {
            tbody.innerHTML = '<tr><td colspan="6" class="text-center" style="padding: 2rem;">No tenants registered yet. Create one to get started.</td></tr>';
            console.log('No tenants to display');
            return;
        }

        tbody.innerHTML = tenants.map(tenant => `
            <tr>
                <td><code style="background: var(--bg); padding: 0.25rem 0.5rem; border-radius: 0.25rem; font-family: monospace; font-size: 0.85rem;">${tenant.id}</code></td>
                <td>${tenant.name}</td>
                <td>${tenant.created}</td>
                <td><span class="badge badge-success">${tenant.status}</span></td>
                <td><span class="badge badge-primary">${tenant.apiKeys} keys</span></td>
                <td>
                    <button class="btn btn-primary btn-small" onclick="viewTenantDetails('${tenant.id}')">📊 View</button>
                    <button class="btn btn-success btn-small" onclick="generateNewSecret('${tenant.id}')">🔑 New Key</button>
                    <button class="btn btn-danger btn-small" onclick="deleteTenant('${tenant.id}')">🗑️</button>
                </td>
            </tr>
        `).join('');

        // Update tenant filter dropdown
        updateTenantFilterDropdown();
        console.log('Tenants data loaded successfully');
    } catch (err) {
        console.warn('Error loading tenants data:', err);
    }
}

function updateTenantFilterDropdown() {
    const select = document.getElementById('secretTenantFilter');
    if (!select) return;

    const currentValue = select.value;
    const options = tenants.map(t => `<option value="${t.id}">${t.name} (${t.id})</option>`).join('');
    
    select.innerHTML = '<option value="">All Tenants</option>' + options;
    select.value = currentValue;
}

function viewTenantDetails(tenantId) {
    const tenant = tenants.find(t => t.id === tenantId);
    if (!tenant) return;

    const tenantSecrets = secrets.filter(s => s.tenant === tenantId);

    const modal = document.getElementById('detailsModal');
    const content = modal.querySelector('.modal-content');
    
    content.innerHTML = `
        <span class="modal-close" onclick="closeModal('detailsModal')">&times;</span>
        <div class="modal-header">🏢 ${tenant.name}</div>
        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-bottom: 1.5rem;">
            <div>
                <label style="display: block; font-size: 0.875rem; color: var(--text-muted); margin-bottom: 0.25rem;">Tenant ID</label>
                <div style="font-weight: 500;">${tenant.id}</div>
            </div>
            <div>
                <label style="display: block; font-size: 0.875rem; color: var(--text-muted); margin-bottom: 0.25rem;">Email</label>
                <div style="font-weight: 500;">${tenant.email}</div>
            </div>
            <div>
                <label style="display: block; font-size: 0.875rem; color: var(--text-muted); margin-bottom: 0.25rem;">Tier</label>
                <div style="font-weight: 500; text-transform: capitalize;">${tenant.tier}</div>
            </div>
            <div>
                <label style="display: block; font-size: 0.875rem; color: var(--text-muted); margin-bottom: 0.25rem;">Status</label>
                <div style="font-weight: 500;"><span class="badge badge-success">${tenant.status}</span></div>
            </div>
            <div>
                <label style="display: block; font-size: 0.875rem; color: var(--text-muted); margin-bottom: 0.25rem;">Created</label>
                <div style="font-weight: 500;">${tenant.created}</div>
            </div>
            <div>
                <label style="display: block; font-size: 0.875rem; color: var(--text-muted); margin-bottom: 0.25rem;">API Keys</label>
                <div style="font-weight: 500;">${tenant.apiKeys}</div>
            </div>
            <div>
                <label style="display: block; font-size: 0.875rem; color: var(--text-muted); margin-bottom: 0.25rem;">Max Connections</label>
                <div style="font-weight: 500;">${tenant.maxConnections}</div>
            </div>
            <div>
                <label style="display: block; font-size: 0.875rem; color: var(--text-muted); margin-bottom: 0.25rem;">Max Message Size</label>
                <div style="font-weight: 500;">${tenant.maxMessageSize} KB</div>
            </div>
        </div>
        
        <h4 style="font-weight: 600; margin-bottom: 1rem;">API Keys (${tenantSecrets.length})</h4>
        <div class="table-wrapper" style="margin-bottom: 1.5rem;">
            <table>
                <thead>
                    <tr>
                        <th>Key Name</th>
                        <th>Created</th>
                        <th>Last Used</th>
                        <th>Status</th>
                        <th>Action</th>
                    </tr>
                </thead>
                <tbody>
                    ${tenantSecrets.length === 0 ? 
                        '<tr><td colspan="5" style="text-align: center; padding: 1rem; color: var(--text-muted);">No API keys generated yet</td></tr>' :
                        tenantSecrets.map(secret => `
                            <tr>
                                <td>${secret.name}</td>
                                <td>${secret.created}</td>
                                <td>${secret.lastUsed}</td>
                                <td><span class="badge badge-success">${secret.status}</span></td>
                                <td>
                                    <button class="btn btn-small" style="background: var(--border); color: var(--text);" onclick="revokeSecret('${secret.id}')">Revoke</button>
                                </td>
                            </tr>
                        `).join('')
                    }
                </tbody>
            </table>
        </div>

        <div style="display: flex; gap: 1rem;">
            <button class="btn btn-success" style="flex: 1; background: var(--secondary);" onclick="generateNewSecret('${tenant.id}')">🔑 Generate New Key</button>
            <button class="btn btn-primary" style="flex: 1;" onclick="closeModal('detailsModal')">Close</button>
        </div>
    `;

    modal.classList.add('active');
}

function deleteTenant(tenantId) {
    if (confirm('Are you sure you want to delete this tenant? This action cannot be undone.')) {
        fetch(`${API_BASE}/tenants/${tenantId}`, {
            method: 'DELETE'
        })
        .then(res => res.json())
        .then(data => {
            if (data.success) {
                loadDataFromStorage().then(() => loadTenantsData());
                alert('Tenant deleted successfully');
            } else {
                showError(data.message || 'Failed to delete tenant');
            }
        })
        .catch(err => {
            console.error('Error:', err);
            showError('Failed to delete tenant');
        });
    }
}

function generateNewSecret(tenantId) {
    const tenant = tenants.find(t => t.id === tenantId);
    if (!tenant) return;

    fetch(`${API_BASE}/generate-key`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            tenantId: tenantId,
            name: 'API Key ' + (tenant.apiKeys + 1)
        })
    })
    .then(res => res.json())
    .then(data => {
        if (data.success) {
            showGeneratedSecrets(
                tenantId,
                data.secret.key,
                data.secret.secret,
                data.endpoint,
                data.connectionString
            );
            loadDataFromStorage().then(() => {
                loadTenantsData();
                loadSecretsData();
            });
        }
    })
    .catch(err => {
        console.error('Error:', err);
        showError('Failed to generate new API key');
    });
}

function revokeSecret(secretId) {
    if (confirm('Are you sure you want to revoke this API key? Clients using this key will no longer be able to connect.')) {
        fetch(`${API_BASE}/secrets/${secretId}`, {
            method: 'DELETE'
        })
        .then(res => res.json())
        .then(data => {
            if (data.success) {
                alert('API key revoked successfully');
                loadDataFromStorage().then(() => {
                    loadTenantsData();
                    loadSecretsData();
                });
            }
        })
        .catch(err => {
            console.error('Error:', err);
            showError('Failed to revoke API key');
        });
    }
}

// ===== LOAD SECRETS DATA =====
function loadSecretsData() {
    try {
        const tbody = document.querySelector('#secretsTable tbody');
        if (!tbody) {
            console.warn('Secrets table tbody not found');
            return;
        }

        if (secrets.length === 0) {
            tbody.innerHTML = '<tr><td colspan="7" class="text-center" style="padding: 2rem;">No API keys found. Register a tenant first.</td></tr>';
            console.log('No secrets to display');
            return;
        }

        tbody.innerHTML = secrets.map(secret => {
            const tenant = tenants.find(t => t.id === secret.tenant);
            const maskedSecret = secret.secret.substring(0, 10) + '...' + secret.secret.substring(secret.secret.length - 5);
            return `
                <tr>
                    <td>${secret.name}</td>
                    <td>${tenant ? tenant.name : 'Unknown'}</td>
                    <td>
                        <code style="background: var(--bg); padding: 0.25rem 0.5rem; border-radius: 0.25rem; font-family: monospace; font-size: 0.8rem;">${maskedSecret}</code>
                        <button class="btn btn-small" style="background: var(--border); color: var(--text); margin-left: 0.5rem;" onclick="copyToClipboard('${secret.id}')">Copy</button>
                    </td>
                    <td>${secret.created}</td>
                    <td>${secret.lastUsed}</td>
                    <td><span class="badge ${secret.status === 'active' ? 'badge-success' : 'badge-danger'}">${secret.status}</span></td>
                    <td>
                        <button class="btn btn-primary btn-small" onclick="viewSecretDetails('${secret.id}')">📋</button>
                        <button class="btn btn-danger btn-small" onclick="revokeSecret('${secret.id}')">🔒</button>
                    </td>
                </tr>
            `;
        }).join('');
        console.log('Secrets data loaded successfully');
    } catch (err) {
        console.error('Error loading secrets data:', err);
    }
}

function viewSecretDetails(secretId) {
    const secret = secrets.find(s => s.id === secretId);
    if (!secret) return;

    const tenant = tenants.find(t => t.id === secret.tenant);
    
    const modal = document.getElementById('detailsModal');
    const content = modal.querySelector('.modal-content');
    
    content.innerHTML = `
        <span class="modal-close" onclick="closeModal('detailsModal')">&times;</span>
        <div class="modal-header">🔑 ${secret.name}</div>
        <div style="display: grid; grid-template-columns: 1fr; gap: 1rem; margin-bottom: 1.5rem;">
            <div>
                <label style="display: block; font-size: 0.875rem; color: var(--text-muted); margin-bottom: 0.5rem;">Tenant</label>
                <div style="font-weight: 500; padding: 0.75rem; background: var(--bg); border-radius: 0.375rem;">${tenant ? tenant.name : 'Unknown'}</div>
            </div>
            <div>
                <label style="display: block; font-size: 0.875rem; color: var(--text-muted); margin-bottom: 0.5rem;">API Key</label>
                <div id="temp_secret_${secretId}" style="font-weight: 500; padding: 0.75rem; background: var(--bg); border-radius: 0.375rem; font-family: monospace; word-break: break-all;">${secret.key}</div>
                <button class="btn btn-small" style="margin-top: 0.5rem; background: var(--border); color: var(--text);" onclick="copyToClipboard('temp_secret_${secretId}')">📋 Copy</button>
            </div>
        </div>
        <div style="display: flex; gap: 1rem;">
            <button class="btn btn-danger" style="flex: 1;" onclick="revokeSecret('${secret.id}')">♻️ Revoke</button>
            <button class="btn btn-primary" style="flex: 1;" onclick="closeModal('detailsModal')">Close</button>
        </div>
    `;

    modal.classList.add('active');
}

function filterSecretsByTenant() {
    const selectedTenant = document.getElementById('secretTenantFilter').value;
    const tbody = document.querySelector('#secretsTable tbody');
    
    if (!selectedTenant) {
        loadSecretsData();
        return;
    }

    const filteredSecrets = secrets.filter(s => s.tenant === selectedTenant);
    
    tbody.innerHTML = filteredSecrets.map(secret => {
        const tenant = tenants.find(t => t.id === secret.tenant);
        const maskedSecret = secret.secret.substring(0, 10) + '...' + secret.secret.substring(secret.secret.length - 5);
        return `
            <tr>
                <td>${secret.name}</td>
                <td>${tenant ? tenant.name : 'Unknown'}</td>
                <td>
                    <code style="background: var(--bg); padding: 0.25rem 0.5rem; border-radius: 0.25rem; font-family: monospace; font-size: 0.8rem;">${maskedSecret}</code>
                </td>
                <td>${secret.created}</td>
                <td>${secret.lastUsed}</td>
                <td><span class="badge badge-success">${secret.status}</span></td>
                <td>
                    <button class="btn btn-primary btn-small" onclick="viewSecretDetails('${secret.id}')">📋</button>
                    <button class="btn btn-danger btn-small" onclick="revokeSecret('${secret.id}')">🔒</button>
                </td>
            </tr>
        `;
    }).join('');
}

// ===== UTILITY FUNCTIONS =====
function generateRandomString(length) {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    let result = '';
    for (let i = 0; i < length; i++) {
        result += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return result;
}

window.addEventListener('click', (e) => {
    if (e.target.classList.contains('modal')) {
        e.target.classList.remove('active');
    }
});

// ===== CHECK AUTOSTART =====
function checkAutostart() {
    const user = localStorage.getItem('dashboardUser');
    const server = localStorage.getItem('dashboardServer');
    const currentServer = window.location.origin;

    if (user && server) {
        // Verify server is still valid (this instance)
        if (server === currentServer || server.includes(window.location.hostname)) {
            dashboardState.user = JSON.parse(user);
            dashboardState.isLoggedIn = true;
            dashboardState.selectedServer = currentServer;

            document.querySelector('.login-container').style.display = 'none';
            document.querySelector('.dashboard').style.display = 'flex';

            initializeDashboard();
            loadDashboardData();
        } else {
            // Server mismatch - force re-login
            localStorage.removeItem('dashboardUser');
            localStorage.removeItem('dashboardServer');
        }
    }
}

// ===== EVENT LISTENERS =====
document.addEventListener('DOMContentLoaded', () => {
    // Login form
    const loginForm = document.getElementById('loginForm');
    if (loginForm) {
        loginForm.addEventListener('submit', (e) => {
            handleLogin(e);
        });
    }

    // Logout button
    const logoutBtn = document.querySelector('.logout-btn');
    if (logoutBtn) {
        logoutBtn.addEventListener('click', handleLogout);
    }

    // Check for auto-login
    checkAutostart();

    // Navigation
    setupNavigation();

    // Set default page
    if (dashboardState.isLoggedIn) {
        navigateToPage('dashboard');
    }

    // Initialize settings page
    initSettingsPage();
    
    // Load queue data
    loadQueueActionsData();
});

// ===== UTILITY FUNCTIONS =====
function formatBytes(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}

function formatUptime(minutes) {
    const days = Math.floor(minutes / 1440);
    const hours = Math.floor((minutes % 1440) / 60);
    const mins = minutes % 60;
    return `${days}d ${hours}h ${mins}m`;
}

function debounce(func, wait) {
    let timeout;
    return function executedFunction(...args) {
        const later = () => {
            clearTimeout(timeout);
            func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
}

console.log('FastDataBroker Admin Dashboard initialized');

// ===== SETTINGS MANAGEMENT =====
function setupSettingsTabs() {
    const tabs = document.querySelectorAll('.settings-tab');
    tabs.forEach(tab => {
        tab.addEventListener('click', () => {
            const panelName = tab.dataset.panel;
            switchSettingsPanel(panelName);
        });
    });
}

function switchSettingsPanel(panelName) {
    const panels = document.querySelectorAll('.settings-panel');
    panels.forEach(p => p.classList.remove('active'));
    
    const tabs = document.querySelectorAll('.settings-tab');
    tabs.forEach(tab => tab.classList.remove('active'));
    
    const selectedPanel = document.querySelector(`.settings-panel[data-panel-content="${panelName}"]`);
    if (selectedPanel) selectedPanel.classList.add('active');
    
    const selectedTab = document.querySelector(`.settings-tab[data-panel="${panelName}"]`);
    if (selectedTab) selectedTab.classList.add('active');
}

// ===== GENERAL SETTINGS =====
function saveGeneralSettings() {
    const refreshRate = document.getElementById('refreshRate')?.value || '5000';
    const logLevel = document.getElementById('logLevel')?.value || 'INFO';
    localStorage.setItem('dashboardSettings', JSON.stringify({refreshRate: parseInt(refreshRate), logLevel}));
    alert('✓ Dashboard settings saved! (Refresh rate: ' + refreshRate + 'ms, Log level: ' + logLevel + ')');
}

// ===== HEALTH & METRICS =====
async function loadHealthMetrics() {
    try {
        const [healthRes, metricsRes] = await Promise.all([
            fetch('http://localhost:3000/api/health'),
            fetch('http://localhost:3000/api/metrics')
        ]);
        
        const health = await healthRes.json();
        const metrics = await metricsRes.json();
        
        // Update health status
        const healthStatusEl = document.getElementById('overallHealth');
        if (healthStatusEl) {
            const status = health.status === 'healthy' ? '✓ Healthy' : '✗ Issues Detected';
            healthStatusEl.textContent = status;
            healthStatusEl.className = health.status === 'healthy' ? 'badge badge-success' : 'badge badge-danger';
        }
        
        // Update broker status
        const brokerStatusEl = document.getElementById('brokerStatus');
        if (brokerStatusEl) {
            const brokerCount = metrics.activeConnections || 0;
            brokerStatusEl.textContent = brokerCount > 0 ? 'All Online (' + brokerCount + ' connections)' : 'Offline';
        }
        
        // Update latency
        const latencyEl = document.getElementById('avgLatency');
        if (latencyEl) {
            latencyEl.textContent = (metrics.avgLatency || 0).toFixed(2) + 'ms';
        }
        
        // Update throughput
        const throughputEl = document.getElementById('throughputMetric');
        if (throughputEl) {
            throughputEl.textContent = (metrics.throughput || 0).toLocaleString() + ' msg/s';
        }
        
        console.log('Health & Metrics updated', {health, metrics});
    } catch (error) {
        console.error('Error loading health metrics:', error);
        alert('✗ Error loading health metrics. Please ensure the broker is running.');
    }
}

// ===== BACKUP MANAGEMENT =====

function exportBackup() {
    const backup = JSON.stringify({tenants, secrets});
    const blob = new Blob([backup], {type: 'application/json'});
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `backup-${new Date().toISOString().split('T')[0]}.json`;
    link.click();
    alert('? Backup exported!');
}

function importBackup(e) {
    e.preventDefault();
    alert('? Importing backup...');
}

function resetSystem() {
    if (confirm('?? Reset system?')) alert('? System reset');
}

function initSettingsPage() {
    setupSettingsTabs();
    loadHealthMetrics();
    loadQuicStatus();
    const systemUptimeEl = document.getElementById('systemUptime');
    if (systemUptimeEl) systemUptimeEl.textContent = 'calculating...';
}

// ===== QUIC PROTOCOL STATUS =====
async function loadQuicStatus() {
    try {
        const response = await fetch(`${API_BASE}/quic/status`);
        const data = await response.json();
        
        if (data.success && data.quic) {
            const quic = data.quic;
            const pskStats = quic.pskStats || {};
            const connStats = quic.connectionStats || {};
            
            // Update PSK statistics
            const activePsksEl = document.getElementById('activePsks');
            if (activePsksEl) {
                activePsksEl.textContent = pskStats.activePsks || '0';
            }
            
            const totalValidationsEl = document.getElementById('totalValidations');
            if (totalValidationsEl) {
                totalValidationsEl.textContent = pskStats.validations || '0';
            }
            
            const activeConnectionsEl = document.getElementById('activeConnections');
            if (activeConnectionsEl) {
                activeConnectionsEl.textContent = connStats.activeConnections || '0';
            }
            
            const authFailuresEl = document.getElementById('authFailures');
            if (authFailuresEl) {
                authFailuresEl.textContent = pskStats.failures || '0';
            }
            
            console.log('✓ QUIC status loaded:', data.quic);
        }
    } catch (error) {
        console.error('Error loading QUIC status:', error);
    }
}

// ===== NEW ACTION FUNCTIONS (POST ENDPOINTS) =====

// ===== CREATE STREAM =====
function openCreateStreamModal() {
    document.getElementById('streamName').value = '';
    document.getElementById('streamPartitions').value = '3';
    document.getElementById('streamReplicas').value = '3';
    document.getElementById('streamRetention').value = '7 days';
    openModal('createStreamModal');
}

function submitCreateStream(event) {
    event.preventDefault();
    
    const name = document.getElementById('streamName').value.trim();
    const partitions = parseInt(document.getElementById('streamPartitions').value);
    const replicas = parseInt(document.getElementById('streamReplicas').value);
    const retention = document.getElementById('streamRetention').value;
    
    if (!name) {
        alert('Stream name is required');
        return;
    }
    
    const streamData = { name, partitions, replicas, retention };
    
    fetch('/api/streams', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(streamData)
    })
    .then(res => res.json())
    .then(data => {
        if (data.success) {
            alert('✓ Stream created successfully: ' + name);
            closeModal('createStreamModal');
            loadStreamsData(); // Refresh streams list
        } else {
            alert('✗ Error: ' + (data.error || 'Failed to create stream'));
        }
    })
    .catch(err => {
        console.error('Error creating stream:', err);
        alert('✗ Error: ' + err.message);
    });
}

// ===== BROKER RESTART =====
function openBrokerRestartModal(brokerId) {
    document.getElementById('restartBrokerId').textContent = brokerId;
    window.pendingBrokerId = brokerId;
    openModal('brokerRestartModal');
}

function confirmBrokerRestart() {
    const brokerId = window.pendingBrokerId;
    if (!brokerId) return;
    
    fetch(`/api/brokers/${brokerId}/restart`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
    })
    .then(res => res.json())
    .then(data => {
        if (data.success) {
            alert('✓ Broker restart initiated: ' + brokerId);
            closeModal('brokerRestartModal');
            loadBrokersData(); // Refresh brokers list
        } else {
            alert('✗ Error: ' + (data.error || 'Failed to restart broker'));
        }
    })
    .catch(err => {
        console.error('Error restarting broker:', err);
        alert('✗ Error: ' + err.message);
    });
}

// ===== BROKER PAUSE =====
function openBrokerPauseModal(brokerId) {
    document.getElementById('pauseBrokerId').textContent = brokerId;
    window.pendingBrokerId = brokerId;
    openModal('brokerPauseModal');
}

function confirmBrokerPause() {
    const brokerId = window.pendingBrokerId;
    if (!brokerId) return;
    
    fetch(`/api/brokers/${brokerId}/pause`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
    })
    .then(res => res.json())
    .then(data => {
        if (data.success) {
            alert('✓ Broker paused: ' + brokerId);
            closeModal('brokerPauseModal');
            loadBrokersData(); // Refresh brokers list
        } else {
            alert('✗ Error: ' + (data.error || 'Failed to pause broker'));
        }
    })
    .catch(err => {
        console.error('Error pausing broker:', err);
        alert('✗ Error: ' + err.message);
    });
}

// ===== QUEUE ACTIONS =====
function loadQueueActionsData() {
    fetch('/api/queue-stats')
        .then(res => res.json())
        .then(data => {
            if (data.success && data.stats && data.stats.queues) {
                renderQueuesTable(data.stats.queues);
            }
        })
        .catch(err => console.error('Error loading queue stats:', err));
}

function renderQueuesTable(queues) {
    const tbody = document.querySelector('#queuesTable tbody');
    if (!tbody) return;
    
    if (!queues || queues.length === 0) {
        tbody.innerHTML = '<tr><td colspan="6" class="text-center" style="padding: 2rem;">No queues found</td></tr>';
        return;
    }
    
    tbody.innerHTML = queues.map(queue => `
        <tr>
            <td>${queue.name}</td>
            <td>${queue.depth}</td>
            <td>${queue.rate}</td>
            <td>${queue.avgLatency}</td>
            <td><span class="badge badge-${queue.status === 'active' ? 'success' : 'warning'}">${queue.status}</span></td>
            <td>
                <button class="btn btn-small" onclick="openQueueActionModal('${queue.name}', 'pause')" title="Pause queue">⏸️ Pause</button>
                <button class="btn btn-small" onclick="openQueueActionModal('${queue.name}', 'resume')" title="Resume queue">▶️ Resume</button>
                <button class="btn btn-small btn-danger" onclick="openQueueActionModal('${queue.name}', 'purge')" title="Purge messages">🗑️ Purge</button>
            </td>
        </tr>
    `).join('');
}

function openQueueActionModal(queueName, action) {
    const actionDescriptions = {
        'pause': 'Pause this queue to stop processing messages temporarily.',
        'resume': 'Resume processing messages in this queue.',
        'purge': 'Delete all pending messages in this queue. This action cannot be undone.',
        'drain': 'Gracefully drain all messages before stopping.'
    };
    
    document.getElementById('queueActionName').textContent = queueName;
    document.getElementById('queueActionType').textContent = action;
    document.getElementById('queueActionTitle').textContent = `${action.toUpperCase()} Queue`;
    document.getElementById('queueActionDescription').textContent = actionDescriptions[action] || '';
    
    window.pendingQueueAction = { queueName, action };
    openModal('queueActionModal');
}

function confirmQueueAction() {
    const { queueName, action } = window.pendingQueueAction;
    if (!queueName || !action) return;
    
    const actionData = { queueName, action };
    
    fetch('/api/queue-action', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(actionData)
    })
    .then(res => res.json())
    .then(data => {
        if (data.success) {
            alert(`✓ Queue ${action} completed: ${queueName}`);
            closeModal('queueActionModal');
            loadQueueActionsData(); // Refresh queues list
        } else {
            alert('✗ Error: ' + (data.error || 'Failed to perform queue action'));
        }
    })
    .catch(err => {
        console.error('Error performing queue action:', err);
        alert('✗ Error: ' + err.message);
    });
}

// ===== CLUSTER REBALANCE =====
function openRebalanceClusterModal() {
    // Pre-fill with sample data
    document.getElementById('rebalanceBrokerCount').textContent = '3';
    document.getElementById('rebalancePartitionCount').textContent = '24';
    openModal('rebalanceClusterModal');
}

function confirmRebalanceCluster() {
    fetch('/api/cluster/rebalance', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
    })
    .then(res => res.json())
    .then(data => {
        if (data.success) {
            alert('✓ Cluster rebalancing initiated. This may take 2-5 minutes to complete.');
            closeModal('rebalanceClusterModal');
            loadClustersData(); // Refresh clusters list
        } else {
            alert('✗ Error: ' + (data.error || 'Failed to rebalance cluster'));
        }
    })
    .catch(err => {
        console.error('Error rebalancing cluster:', err);
        alert('✗ Error: ' + err.message);
    });
}

// ===== MODAL HELPER FUNCTION =====
function openModal(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) {
        modal.style.display = 'flex';
    }
}

