# NPM Publishing Setup - FastDataBroker JavaScript SDK

**Date**: April 7, 2026  
**Status**: ✅ Ready for npm publishing

---

## 📦 What Was Created

### File: `.github/workflows/npm-publish.yml`

A complete GitHub Actions workflow for:
- ✅ Testing on Node 16, 18, 20
- ✅ TypeScript compilation and strict checking
- ✅ ESLint linting and Prettier formatting
- ✅ Jest test execution with coverage
- ✅ Security audits (npm audit + snyk)
- ✅ Publishing to npm registry
- ✅ Package verification
- ✅ GitHub Release creation

---

## 🔧 Setup Instructions

### Step 1: Create npm Account (If You Don't Have One)

1. Go to https://www.npmjs.com
2. Click "Sign Up"
3. Complete registration
4. Verify email

---

### Step 2: Create npm Access Token

1. Go to https://www.npmjs.com
2. Login to your account
3. Click your **profile icon** (top right)
4. Select **Access Tokens**
5. Click **Generate New Token**
6. Select **Automation** (for CI/CD)
7. Copy the token (starts with `npm_`)

**IMPORTANT**: Keep this token secure! Only GitHub CI/CD will use it.

---

### Step 3: Add GitHub Secret

1. Go to **GitHub → FastDataBroker Repo → Settings**
2. Navigate to **Secrets and variables → Actions**
3. Click **New repository secret**
4. Name: `NPM_TOKEN`
5. Value: Paste your npm token
6. Click **Add secret**

---

### Step 4: Update package.json (Optional)

Your current package.json is good, but you might want to update:

```json
{
  "name": "@FastDataBroker/sdk",  // ✅ Scoped package (@org/name)
  "version": "0.4.0",
  "description": "High-performance message queue SDK for JavaScript/TypeScript",
  "repository": {
    "type": "git",
    "url": "https://github.com/suraj202923/FastDataBroker.git",
    "directory": "sdks/javascript"
  }
}
```

---

## 🚀 Workflow Execution Flow

```
On: Git tag push (e.g., git tag v0.4.0)
         ↓
    ┌─────────────────────────────────────┐
    │ build-and-test (Node 16, 18, 20)   │
    │ ├─ npm ci (install)                 │
    │ ├─ npm run build (TypeScript)       │
    │ ├─ npm run lint (ESLint)            │
    │ ├─ Prettier format check            │
    │ ├─ npm test (Jest)                  │
    │ └─ Coverage upload                  │
    └────────────┬────────────────────────┘
                 │
    ┌────────────┴───────────────────────┐
    │ build-quality (Node 20 strict)     │
    │ ├─ Strict TypeScript build         │
    │ ├─ npm audit                       │
    │ └─ Snyk security check             │
    └────────────┬────────────────────────┘
                 │
    ┌────────────┴───────────────────────────────┐
    │ publish-to-npm                            │
    │ ├─ Install dependencies                   │
    │ ├─ Build package                          │
    │ ├─ Run final tests                        │
    │ ├─ npm publish (@FastDataBroker/sdk)      │
    │ ├─ Verify npm package                     │
    │ └─ Install from npm and test              │
    └────────────┬────────────────────────────────┘
                 │
    ┌────────────┴───────────────────────────────┐
    │ publish-github-release                    │
    │ └─ Create GitHub Release with docs        │
    └──────────────────────────────────────────────┘
```

---

## 📊 Workflow Jobs

### 1. build-and-test
**Runs on**: Node 16.x, 18.x, 20.x (parallel)

```yaml
Steps:
  ✅ Checkout code
  ✅ Setup Node.js
  ✅ npm ci (clean install)
  ✅ npm run build (TypeScript compilation)
  ✅ npm run lint (ESLint)
  ✅ npm format check (Prettier)
  ✅ npm test (Jest with coverage)
  ✅ Upload coverage to Codecov
```

**Time**: ~4-5 minutes per Node version (parallel)

---

### 2. build-quality
**Runs on**: Node 20.x only

```yaml
Steps:
  ✅ Strict TypeScript compilation
  ✅ npm audit (dependency vulnerabilities)
  ✅ snyk test (security scanning)
```

**Time**: ~2-3 minutes

---

### 3. publish-to-npm
**Triggers**: Only on git tags (e.g., `v0.4.0`)  
**Depends on**: build-and-test + build-quality

```yaml
Steps:
  ✅ Setup Node.js with npm registry
  ✅ Install dependencies
  ✅ Build TypeScript
  ✅ Run final tests
  ✅ npm publish (to registry)
  ✅ Verify package published
  ✅ Install from npm and test
```

**Time**: ~3-4 minutes

---

### 4. publish-github-release
**Triggers**: Only after npm publish succeeds

```yaml
Steps:
  ✅ Create GitHub Release
  ✅ Include quickstart code
  ✅ Link to npm package
  ✅ Changelog (auto-linked commits)
```

**Time**: ~1 minute

---

## 🔐 Security Notes

### NPM Token Best Practices
- ✅ Token stored in GitHub Secrets (encrypted)
- ✅ Never commit token to repository
- ✅ Use "Automation" type tokens (for CI/CD)
- ✅ Set expiration (optional but recommended)

### Workflow Security
- ✅ `NODE_AUTH_TOKEN` only used in npm publish step
- ✅ Not logged or printed
- ✅ Automatically cleaned up after step

### Scope Package (@FastDataBroker/sdk)
- ✅ Only you can publish to @FastDataBroker scope
- ✅ Prevents name collisions
- ✅ Professional appearance

---

## 📦 NPM Publishing Details

### Publishing Step
```yaml
- name: Publish to npm
  working-directory: sdks/javascript
  run: npm publish
  env:
    NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

**What happens**:
1. Reads `package.json` version
2. Builds distribution files
3. Uploads to npm registry
4. Tags release on npm

---

### Package Name Resolution
```json
{
  "name": "@FastDataBroker/sdk"
}
```

**npm URL**: https://www.npmjs.com/package/@FastDataBroker/sdk

**Installation**:
```bash
npm install @FastDataBroker/sdk
npm install @FastDataBroker/sdk@0.4.0
npm install @FastDataBroker/sdk@latest
```

---

## 🚀 Test the Workflow

### Step 1: Create Test Tag
```bash
git tag v0.4.0-test
git push origin v0.4.0-test
```

### Step 2: Monitor GitHub Actions
Go to: **GitHub → Actions → FastDataBroker JavaScript SDK**

### Step 3: Verify npm Package
After publish succeeds:
```bash
npm view @FastDataBroker/sdk@0.4.0-test
npm search fastdatabroker
```

### Step 4: Test Installation
```bash
npm install @FastDataBroker/sdk@0.4.0-test

# Or test in separate directory
mkdir /tmp/test-npm
cd /tmp/test-npm
npm init -y
npm install @FastDataBroker/sdk@0.4.0-test
```

### Step 5: Delete Test Tag (Optional)
```bash
git tag -d v0.4.0-test
git push origin --delete v0.4.0-test
```

---

## ✅ Production Release

When you're ready for real release:

```bash
# Update version in package.json
cd sdks/javascript
npm version minor  # or major/patch

# Creates commit + tag
git log --oneline -5  # Verify
git push origin main --tags
```

Or manually:
```bash
git tag v0.4.1
git push origin v0.4.1
```

---

## 📊 Expected Output

### Successful Test Output
```
✅ Set up Node.js 20.x
✅ npm ci
✅ npm run build
✅ npm run lint
✅ prettier --check
✅ npm test
✅ Coverage uploaded
```

### Successful Publish Output
```
✅ npm publish
npm notice Publishing @FastDataBroker/sdk@0.4.0
npm notice Uploaded 1.2MB
npm notice You have access to this package
npm notice This package has 1 new version
```

### GitHub Release Created
```
Release: FastDataBroker JavaScript SDK v0.4.0
Installation: npm install @FastDataBroker/sdk@0.4.0
Quick start code included
```

---

## 🔧 Troubleshooting

### Issue: "npm 404 not found"
**Solution**: Ensure `@FastDataBroker` scope is published

```bash
npm whoami  # Verify logged in
npm access ls-packages  # Check published packages
```

### Issue: "Invalid authentication token"
**Solution**: Regenerate npm token

1. Go to npmjs.com → Access Tokens
2. Delete old token
3. Create new token
4. Update GitHub secret `NPM_TOKEN`

### Issue: "Package has failing tests"
**Solution**: Fix tests before publishing

```bash
cd sdks/javascript
npm test
npm run lint
npm run build
```

### Issue: "TypeScript compilation errors"
**Solution**: Fix TypeScript errors

```bash
cd sdks/javascript
npm run build  # Shows errors
# Fix errors in src/
npm run build  # Should pass now
```

---

## 📝 Workflow Files

### Created Files
- ✅ `.github/workflows/npm-publish.yml` (120 lines)
- ✅ `NPM_PUBLISHING_SETUP.md` (this file)

### Existing Files (Used by workflow)
- ✅ `sdks/javascript/package.json`
- ✅ `sdks/javascript/tsconfig.json`
- ✅ `sdks/javascript/jest.config.js`
- ✅ `.eslintrc.js` (or `.eslintrc.json`)
- ✅ `prettier.config.js` (or `.prettierrc`)

---

## 🎯 Complete Release Checklist

When releasing JavaScript SDK:

```
Before Release:
  ☐ Update version in package.json (x.y.z)
  ☐ Update CHANGELOG.md with changes
  ☐ Run local tests: npm test
  ☐ Run build: npm run build
  ☐ Run lint: npm run lint

Release:
  ☐ Commit version bump
  ☐ Create git tag: git tag vx.y.z
  ☐ Push tag: git push origin vx.y.z

After Release:
  ☐ Monitor GitHub Actions
  ☐ Verify npm package: npm view @FastDataBroker/sdk@x.y.z
  ☐ Test installation: npm install @FastDataBroker/sdk@x.y.z
  ☐ Check GitHub Release created
  ☐ Update documentation with new version
```

---

## 📚 Related Resources

### npm SCope Registry
- https://docs.npmjs.com/about-scopes
- https://docs.npmjs.com/creating-and-publishing-scoped-packages

### GitHub Actions Node Setup
- https://github.com/actions/setup-node

### npm Publishing Best Practices
- https://docs.npmjs.com/cli/publish
- https://docs.npmjs.com/about-npm-security

### TypeScript Best Practices
- https://www.typescriptlang.org/docs/handbook/declaration-files/publishing.html
- https://www.typescriptlang.org/tsconfig

---

## 🎉 Summary

### What's Ready
✅ Full npm publishing workflow  
✅ Multi-version Node testing  
✅ Strict TypeScript validation  
✅ Security scanning  
✅ Coverage tracking  
✅ Automated GitHub releases  
✅ Package verification  

### What You Need
✅ npm account (created)  
✅ npm access token (generated)  
✅ GitHub secret NPM_TOKEN (added)  
✅ Git tag (push to trigger)  

### What Happens Automatically
✅ Tests on Node 16, 18, 20  
✅ Publishes to npm  
✅ Creates GitHub release  
✅ Verifies package  
✅ Everything documented  

---

## 🚀 You're Ready to Publish!

```bash
# Step 1: Create tag
git tag v0.4.0

# Step 2: Push
git push origin v0.4.0

# Step 3: Watch GitHub Actions
# → npm publish happens automatically
# → Package available at: npm install @FastDataBroker/sdk@0.4.0

# Done! 🎉
```

---

**Last Updated**: April 7, 2026  
**Status**: ✅ NPM Publishing Ready  
**Next Step**: Get npm token and add to GitHub secrets
