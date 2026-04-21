# FastDataBroker Admin Dashboard - Complete Getting Started Guide

## ðŸ“– Table of Contents
1. [System Requirements](#system-requirements)
2. [Installation Steps](#installation-steps)
3. [Starting the Dashboard](#starting-the-dashboard)
4. [First Login](#first-login)
5. [Main Features Overview](#main-features-overview)
6. [Common Operations](#common-operations)
7. [Testing & Verification](#testing--verification)
8. [Troubleshooting](#troubleshooting)

---

## âœ… System Requirements

### Software Requirements
- **Node.js**: 14.0.0 or higher
- **npm**: 6.0.0 or higher
- **Windows 10+** or **macOS 10.14+** or **Linux** (Ubuntu 18+)
- **Web Browser**: Chrome, Firefox, Edge, Safari (latest versions)

### Running Services Required
- **FastDataBroker Server**: Running on port 6379
- **Admin API**: Running on port 8080

### Recommended Hardware
- **RAM**: 512 MB minimum (1 GB recommended)
- **CPU**: Dual-core or better
- **Disk Space**: 500 MB free

### Verify Requirements

Check Node.js is installed:
```bash
node --version
# Output should show: v14.0.0 or higher

npm --version
# Output should show: 6.0.0 or higher
```

If not installed, download from: https://nodejs.org/

---

## ðŸ”§ Installation Steps

### Step 1: Navigate to FastDataBroker Directory

Open Command Prompt (or PowerShell) and navigate to the project:
```bash
cd d:\suraj202923\FastDataBroker
```

Expected output:
```
d:\suraj202923\FastDataBroker>
```

### Step 2: Verify Dashboard Files

Check that these files exist:
```bash
# Windows Command Prompt
dir admin-dashboard.html dashboard-server.js package.json

# PowerShell
Get-ChildItem admin-dashboard.html, dashboard-server.js, package.json
```

Expected files:
- âœ“ admin-dashboard.html (20 KB)
- âœ“ dashboard-server.js (15 KB)
- âœ“ package.json (1 KB)

### Step 3: Install Dependencies

Run npm install to download required packages:
```bash
npm install
```

Expected output:
```
added 50 packages, and audited 51 packages
```

This creates a `node_modules` folder with all dependencies.

### Step 4: Verify Installation

List installed packages:
```bash
npm list --depth=0
```

Expected packages:
```
â”œâ”€â”€ cors@2.8.5
â”œâ”€â”€ express@4.18.2
â””â”€â”€ nodemon@3.0.1 (dev)
```

---

## ðŸš€ Starting the Dashboard

### Method 1: Using npm (Recommended)

```bash
npm start
```

### Method 2: Using Batch Script (Windows)

Double-click:
```
start-dashboard.bat
```

Or from Command Prompt:
```bash
start-dashboard.bat
```

### Method 3: Using PowerShell Script

```bash
powershell -ExecutionPolicy Bypass -File start-dashboard.ps1
```

### Expected Output

You should see this message:
```
â•”â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•—
â•‘                                                        â•‘
â•‘   âš¡ FastDataBroker Admin Dashboard Server             â•‘
â•‘                                                        â•‘
â•‘   Status: Running âœ“                                    â•‘
â•‘   URL: http://127.0.0.1:3000                          â•‘
â•‘   API Base: http://127.0.0.1:3000/api/v1              â•‘
â•‘                                                        â•‘
â•‘   Demo Credentials:                                    â•‘
â•‘   Username: admin                                      â•‘
â•‘   Password: admin                                      â•‘
â•‘   API Key: admin-key                                   â•‘
â•‘                                                        â•‘
â•šâ•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
```

**The dashboard is now running!** âœ“

---

## ðŸŒ First Login

### 1. Open Web Browser

Open your web browser (Chrome, Firefox, Edge, Safari):
```
http://127.0.0.1:3000
```

### 2. Login Page Appears

You'll see the login screen with:
- FastDataBroker logo and title
- "Admin Dashboard" subtitle
- Username input field
- Password input field
- Login button
- Demo credentials hint box

### 3. Enter Demo Credentials

**Username**: type `admin`  
**Password**: type `admin`

### 4. Click Login Button

The system validates credentials and logs you in.

### 5. Dashboard Home Page

You now see the main dashboard with:
- **Purple gradient background**
- **Left sidebar** with menu items (Dashboard, Server Info, Tenants, API Explorer, Metrics, Configuration)
- **Top navigation bar** showing "Dashboard" title and your user avatar
- **Main content area** with stats cards and recent activity

---

## ðŸ“Š Main Features Overview

### Dashboard Home (Default View)

**Status Cards** (4 large cards):
1. **Server Status** - Shows "Online" (Purple)
2. **Total Tenants** - Shows "3" (Green)
3. **API Health** - Shows "Healthy" (Blue)
4. **Uptime** - Shows "99.9%" (Orange)

**Quick Action Buttons**:
- ðŸ”„ **Refresh Status** - Update metrics
- ðŸ§ª **Test API** - Go to API Explorer
- ðŸ“– **View Docs** - Open documentation

**Recent Activity Table**:
Shows recent system events with timestamps and status badges.

---

## ðŸ–¥ï¸ Server Information Section

**Click**: ðŸ–¥ï¸ Server Info in sidebar

**What you'll see**:
- Server version (0.1.16)
- Uptime (24h 30m)
- Memory usage (245 MB)
- CPU usage (12%)
- Table of available endpoints

**Available Endpoints**:
| Endpoint | Method | Status | Test |
|----------|--------|--------|------|
| /health | GET | Online | Button |
| /api/v1/tenants | GET | Online | Button |
| /api/v1/auth/login | POST | Online | Button |
| /api/v1/stats | GET | Online | Button |

---

## ðŸ‘¥ Tenant Management

**Click**: ðŸ‘¥ Tenants in sidebar

**Pre-existing Tenants**:
1. **default** - Rate limit: 10,000 req/s
2. **acme-corp** - Rate limit: 5,000 req/s
3. **startup-xyz** - Rate limit: 1,000 req/s

**Actions**:
- **ðŸ“‹ Load Tenants** - Refresh tenant list
- **âž• Create Tenant** - Add new tenant
- **View** button - See tenant details
- **Edit** button - Modify settings

---

## ðŸ”Œ API Explorer

**Click**: ðŸ”Œ API Explorer in sidebar

**This panel lets you test APIs**:

1. **HTTP Method Selector** (top-left)
   - GET
   - POST
   - PUT
   - DELETE

2. **Endpoint Field** (top-right)
   - Enter: `/api/v1/tenants`
   - Or choose from examples

3. **Headers Editor** (middle-left)
   - Pre-filled with:
     ```json
     {
       "Content-Type": "application/json",
       "X-API-Key": "admin-key"
     }
     ```

4. **Request Body** (middle-right)
   - For POST/PUT requests
   - Enter JSON data

5. **Send Request Button**
   - Executes the API call
   - Shows real-time response

6. **Response Display** (bottom)
   - Shows HTTP status code
   - Displays full response JSON
   - Color-coded (green = success, red = error)

---

## ðŸ“ˆ Metrics Section

**Click**: ðŸ“ˆ Metrics in sidebar

**Performance Cards**:
- **Requests/Sec**: 12,450 (â†‘ 5% from last hour)
- **Avg Latency**: 2.4ms (â†“ Good performance)
- **Error Rate**: 0.02% (Excellent reliability)
- **Total Requests**: 2.4M (Lifetime)

**Endpoint Performance Table**:
Shows metrics for each API endpoint:
- /api/v1/tenants
- /api/v1/send
- /api/v1/stats
- /health

---

## âš™ï¸ Configuration Section

**Click**: âš™ï¸ Configuration in sidebar

**Server Configuration**:
- Server Port: 6379 (read-only)
- Bind Address: 0.0.0.0 (read-only)
- Max Connections: 10,000 (editable)
- Log Level: INFO (dropdown)
- Data Retention: 30 days (editable)

**Security Settings**:
- Enable TLS: (toggle checkbox)
- Enable CORS: (toggle checkbox)
- Require Authentication: (toggle checkbox)

**Buttons**:
- ðŸ’¾ Save Configuration
- â†º Reset to defaults

---

## ðŸŽ¯ Common Operations

### Operation 1: Check Server Status

```
1. Click "ðŸ“Š Dashboard" in sidebar
2. Look at the purple "Server Status" card
3. Click "ðŸ”„ Refresh Status" if needed
4. Check recent activity table
```

### Operation 2: Test an API Endpoint

```
1. Click "ðŸ”Œ API Explorer"
2. Select method: GET (dropdown)
3. Enter endpoint: /api/v1/tenants
4. Click "ðŸ“¤ Send Request"
5. View response on right side
```

### Operation 3: Create a New Tenant

```
1. Click "ðŸ‘¥ Tenants"
2. Click "âž• Create Tenant" button
3. Enter tenant ID in dialog (e.g., "trial-user")
4. Confirm creation
5. See new tenant in table
```

### Operation 4: View Server Metrics

```
1. Click "ðŸ“ˆ Metrics"
2. Review performance cards (requests/sec, latency, errors)
3. Check endpoint performance table
4. Identify any slow endpoints
```

### Operation 5: Configure Server

```
1. Click "âš™ï¸ Configuration"
2. Modify settings:
   - Increase Max Connections to 20,000
   - Change Log Level to DEBUG
3. Click "ðŸ’¾ Save Configuration"
4. Settings applied immediately
```

---

## ðŸ§ª Testing & Verification

### Test 1: Health Check

```
1. Go to ðŸ–¥ï¸ Server Info
2. Click "âœ“ Health Check" button
3. You should see: "âœ“ Server is healthy"
```

### Test 2: Test API Endpoint

```
1. Go to ðŸ”Œ API Explorer
2. Method: GET
3. Endpoint: /api/v1/tenants
4. Click "ðŸ“¤ Send Request"
5. You should see: Status 200 OK with tenant list
```

### Test 3: Create Tenant via API

```
1. Go to ðŸ”Œ API Explorer
2. Method: POST
3. Endpoint: /api/v1/tenants
4. Add request body:
   {
     "id": "test-tenant",
     "name": "Test Tenant",
     "rateLimit": "5000 req/s"
   }
5. Click "ðŸ“¤ Send Request"
6. You should see: Status 201 Created with tenant details
```

### Test 4: Send Message

```
1. Go to ðŸ”Œ API Explorer
2. Method: POST
3. Endpoint: /api/v1/send
4. Add request body:
   {
     "tenantId": "default",
     "message": "Hello World",
     "priority": "high"
   }
5. Click "ðŸ“¤ Send Request"
6. You should see: Status 200 OK with message ID
```

### Test 5: Verify All Tenants Loaded

```
1. Go to ðŸ‘¥ Tenants
2. Click "ðŸ“‹ Load Tenants"
3. Active Tenants table should show at least 3 tenants
4. All should have "Active" status (green badge)
```

---

## ðŸ› Troubleshooting

### Issue 1: Can't Connect to Dashboard

**Error**: "Connection Refused" or "This site can't be reached"

**Solution**:
1. Check if npm start is running:
   ```bash
   netstat -ano | findstr :3000
   ```
2. If no result, start it:
   ```bash
   npm start
   ```
3. Wait 2-3 seconds for startup
4. Try URL again: http://127.0.0.1:3000

### Issue 2: Login Fails (Wrong Credentials)

**Error**: "âŒ Invalid credentials" message

**Solution**:
1. Verify you typed exactly: `admin` (username)
2. Verify you typed exactly: `admin` (password)
3. Check Caps Lock is OFF
4. Clear browser cookies: Ctrl+Shift+Delete
5. Try again

### Issue 3: API Returns 401 Unauthorized

**Error**: "Unauthorized" in API response

**Solution**:
1. Check headers include: `X-API-Key: admin-key`
2. Or use Bearer token from login endpoint
3. Verify header spelling and capitalization
4. Clear any old authentication tokens

### Issue 4: Metrics Not Showing

**Error**: Metrics section shows no data

**Solution**:
1. Click "ðŸ”„ Refresh Status" button
2. Wait 2-3 seconds for data to load
3. Check browser console for errors: F12 â†’ Console
4. Verify Admin API is running on port 8080

### Issue 5: Can't Find Dashboard Files

**Error**: "admin-dashboard.html not found"

**Solution**:
1. Navigate to correct directory:
   ```bash
   cd d:\suraj202923\FastDataBroker
   ```
2. Verify files exist:
   ```bash
   dir admin-dashboard.html
   dir dashboard-server.js
   ```
3. If missing, re-extract the files
4. Reinstall with: `npm install`

### Issue 6: npm install Falls (Module Not Found)

**Error**: "npm ERR! code ERESOLVE"

**Solution**:
1. Clear npm cache:
   ```bash
   npm cache clean --force
   ```
2. Delete node_modules folder:
   ```bash
   rmdir node_modules /s /q
   ```
3. Try install again:
   ```bash
   npm install
   ```

---

## âœ… Verification Checklist

### Pre-Startup
- [ ] Node.js 14+ installed: `node --version`
- [ ] npm 6+ installed: `npm --version`
- [ ] In correct directory: `d:\suraj202923\FastDataBroker`
- [ ] Files exist: admin-dashboard.html, dashboard-server.js
- [ ] Dependencies installed: `npm list`
- [ ] FastDataBroker server running (port 6379)
- [ ] Admin API running (port 8080)

### After Starting (`npm start`)
- [ ] Server starts without errors
- [ ] Shows "Status: Running âœ“" message
- [ ] Shows correct URL: http://127.0.0.1:3000
- [ ] Shows demo credentials
- [ ] No error messages in console

### After Login
- [ ] Dashboard loads (purple color scheme visible)
- [ ] Sidebar shows 6 menu items
- [ ] Stats cards display data
- [ ] Recent activity table shows entries
- [ ] All buttons are clickable

### Feature Testing
- [ ] Server Info section shows data
- [ ] Tenants table shows 3 tenants
- [ ] API Explorer allows request entry
- [ ] Metrics display performance data
- [ ] Configuration form opens
- [ ] Status indicator shows "Online"

### API Testing
- [ ] Health check returns 200 OK
- [ ] GET /api/v1/tenants returns 200 OK
- [ ] POST /api/v1/send works
- [ ] POST /api/v1/consume works
- [ ] All API keys and headers correct

---

## ðŸŽ‰ Success Indicators

When everything is working correctly, you'll see:

1. âœ… Dashboard loads instantly in browser
2. âœ… Login works with admin/admin
3. âœ… Server status shows "Online"
4. âœ… All 3 sample tenants visible
5. âœ… API Explorer shows responses
6. âœ… Metrics display real numbers
7. âœ… Configuration buttons save changes
8. âœ… No error messages in console

---

## ðŸ“Š Quick Testing Workflow

1. **Start**: `npm start`
2. **Browse**: http://127.0.0.1:3000
3. **Login**: admin/admin
4. **Test API**: 
   - Click ðŸ”Œ API Explorer
   - GET /api/v1/tenants
   - Click Send
   - Verify response
5. **Done**: Dashboard ready for use âœ“

---

## ðŸš€ Next Steps

After successful installation, you can:

1. **Learn More**
   - Read [DASHBOARD_FEATURES.md](DASHBOARD_FEATURES.md) for detailed info
   - Review [DASHBOARD_QUICK_REFERENCE.md](DASHBOARD_QUICK_REFERENCE.md) for commands

2. **Customize**
   - Change default credentials in production
   - Configure rate limits per tenant
   - Set up monitoring and alerting

3. **Integrate**
   - Connect to your FastDataBroker server
   - Set up CI/CD pipelines
   - Create custom API tests

4. **Deploy**
   - Move to production server
   - Enable TLS/HTTPS
   - Set up load balancing

---

## ðŸ“ž Getting Help

### If Something Goes Wrong

1. **Check Logs**: Look for error messages in terminal
2. **Verify Services**: Ensure FastDataBroker and Admin API running
3. **Restart**: Stop dashboard (`Ctrl+C`) and restart (`npm start`)
4. **Check Ports**: Verify ports 3000, 6379, 8080 are available
5. **Read Docs**: Check [DASHBOARD_SETUP_GUIDE.md](DASHBOARD_SETUP_GUIDE.md)

### Documentation Files
- ðŸ“– [ADMIN_DASHBOARD_README.md](ADMIN_DASHBOARD_README.md) - Main README
- ðŸš€ [DASHBOARD_SETUP_GUIDE.md](DASHBOARD_SETUP_GUIDE.md) - Detailed setup
- âš¡ [DASHBOARD_QUICK_REFERENCE.md](DASHBOARD_QUICK_REFERENCE.md) - Quick reference
- âœ¨ [DASHBOARD_FEATURES.md](DASHBOARD_FEATURES.md) - Feature documentation

---

## ðŸŽ“ Example API Calls

### Using curl (Command Line)

**Get all tenants**:
```bash
curl -H "X-API-Key: admin-key" http://127.0.0.1:3000/api/v1/tenants
```

**Create new tenant**:
```bash
curl -X POST http://127.0.0.1:3000/api/v1/tenants ^
  -H "X-API-Key: admin-key" ^
  -H "Content-Type: application/json" ^
  -d "{\"id\": \"new-tenant\", \"name\": \"New Tenant\", \"rateLimit\": \"5000 req/s\"}"
```

**Get server stats**:
```bash
curl -H "X-API-Key: admin-key" http://127.0.0.1:3000/api/v1/stats
```

---

**Congratulations! You're now ready to use the FastDataBroker Admin Dashboard!** ðŸŽ‰

**Questions?** Review the documentation files or check the troubleshooting section above.

**Version**: 1.0.0  
**Status**: âœ… Production Ready  
**Last Updated**: April 13, 2026

