# 🔄 FastDataBroker CI/CD GitHub Actions Setup

## What You Need to Change

### 1. **Workflow Name**
```yaml
# OLD:
name: Python Package using Conda

# NEW:
name: FastDataBroker Python Package
```

---

### 2. **Trigger Events** (When workflow runs)
```yaml
# OLD:
on: [push]

# NEW:
on: 
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]
```
✅ Runs on push AND pull requests (safer)  
✅ Only on main/develop branches (not feature branches)

---

### 3. **Python Version Matrix**
```yaml
# OLD:
- name: Set up Python 3.10
  uses: actions/setup-python@v3
  with:
    python-version: '3.10'

# NEW:
strategy:
  max-parallel: 5
  matrix:
    python-version: ['3.8', '3.9', '3.10', '3.11', '3.12']

steps:
- name: Set up Python ${{ matrix.python-version }}
  uses: actions/setup-python@v4
  with:
    python-version: ${{ matrix.python-version }}
```
✅ Tests on all supported Python versions  
✅ Parallel execution (faster)  
✅ Updated action version (v3 → v4)

---

### 4. **Install Dependencies**
```yaml
# OLD:
- name: Install dependencies
  run: |
    conda env update --file environment.yml --name base

# NEW:
- name: Install dependencies
  run: |
    conda env update --file environment.yml --name base
    # Additional FastDataBroker specific dependencies
    conda install -c conda-forge rust maturin
    pip install -e python/
```
✅ Installs Rust (for Cargo)  
✅ Installs maturin (Python ↔ Rust bindings)  
✅ Installs FastDataBroker SDK in dev mode

---

### 5. **Add Type Checking**
```yaml
# NEW SECTION:
- name: Type checking with mypy
  run: |
    pip install mypy
    mypy python/ --ignore-missing-imports || true
```
✅ Validates Python type hints  
✅ Catches potential bugs early  
✅ `|| true` = don't fail CI if mypy errors

---

### 6. **Improved Testing**
```yaml
# OLD:
- name: Test with pytest
  run: |
    conda install pytest
    pytest

# NEW:
- name: Test with pytest
  run: |
    conda install pytest pytest-cov
    pytest tests/ -v --cov=fastdatabroker_sdk --cov-report=xml
```
✅ Includes coverage reporting  
✅ Verbose output (-v)  
✅ Specified test directory  
✅ Generates XML for codecov integration

---

### 7. **Code Coverage Upload**
```yaml
# NEW SECTION:
- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v3
  with:
    file: ./coverage.xml
    flags: unittests
    name: codecov-umbrella
    fail_ci_if_error: false
```
✅ Tracks test coverage over time  
✅ Shows coverage badge in README  
✅ Monitors code quality trends

---

### 8. **Auto-Publish to PyPI (New Job)**
```yaml
# NEW JOB:
build-and-publish:
  runs-on: ubuntu-latest
  needs: build
  if: startsWith(github.ref, 'refs/tags')
  
  steps:
  - uses: actions/checkout@v4
  
  - name: Set up Python
    uses: actions/setup-python@v4
    with:
      python-version: '3.10'
  
  - name: Build distribution
    run: |
      pip install --upgrade build twine
      cd python && python -m build
  
  - name: Publish to PyPI
    uses: pypa/gh-action-pypi-publish@release/v1
    with:
      password: ${{ secrets.PYPI_API_TOKEN }}
      packages-dir: python/dist/
```

**When it runs**: Only when you create a git tag (release)
```bash
git tag v0.2.0
git push origin v0.2.0
# ✅ Automatically publishes to PyPI
```

---

### 9. **Docker Build & Push (New Job)**
```yaml
# NEW JOB:
docker-build:
  runs-on: ubuntu-latest
  if: startsWith(github.ref, 'refs/tags')
  
  steps:
  - uses: actions/checkout@v4
  
  - name: Set up Docker Buildx
    uses: docker/setup-buildx-action@v2
  
  - name: Login to Docker Hub
    uses: docker/login-action@v2
    with:
      username: ${{ secrets.DOCKER_USERNAME }}
      password: ${{ secrets.DOCKER_PASSWORD }}
  
  - name: Build and push Docker image
    uses: docker/build-push-action@v4
    with:
      context: .
      push: true
      tags: |
        ${{ secrets.DOCKER_USERNAME }}/fastdatabroker:${{ github.ref_name }}
        ${{ secrets.DOCKER_USERNAME }}/fastdatabroker:latest
```

**When it runs**: Only when you create a git tag  
**What it does**: Builds and pushes Docker image to Docker Hub

---

## 📋 GitHub Secrets You Need to Set Up

Add these in: **GitHub → Settings → Secrets and variables → Actions**

| Secret Name | Value | Purpose |
|------------|-------|---------|
| `PYPI_API_TOKEN` | [Get from pypi.org](https://pypi.org/help/#apitoken) | Auto-publish to PyPI |
| `DOCKER_USERNAME` | Your Docker Hub username | Push images to Docker Hub |
| `DOCKER_PASSWORD` | Your Docker Hub password | Authenticate to Docker Hub |

---

## 🚀 Workflow Steps Explained

### **When you push to main/develop** → Runs `build` job
1. ✅ Tests on Python 3.8-3.12
2. ✅ Lints code (flake8)
3. ✅ Type checks (mypy)
4. ✅ Runs pytest with coverage
5. ✅ Uploads coverage to Codecov

### **When you create a git tag** → Runs `build` + `build-and-publish` + `docker-build`
1. ✅ All of above
2. ✅ **Then** publishes to PyPI automatically
3. ✅ **Then** builds and pushes Docker image

---

## 📝 action.yml File Structure

```
.github/
├── workflows/
│   └── python-package-conda.yml  ← The file we created
```

Save it in your repo at: `.github/workflows/python-package-conda.yml`

---

## 🔑 Quick Setup (5 minutes)

1. **Copy the workflow file**
   ```bash
   mkdir -p .github/workflows
   # Copy content to .github/workflows/python-package-conda.yml
   ```

2. **Add secrets to GitHub**
   - Go to: `https://github.com/suraj202923/fastdatabroker/settings/secrets/actions`
   - Click "New repository secret"
   - Add: `PYPI_API_TOKEN`, `DOCKER_USERNAME`, `DOCKER_PASSWORD`

3. **Create git tag to test**
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

4. **Watch it run**
   - Go to: `https://github.com/suraj202923/fastdatabroker/actions`
   - See workflow run in real-time

---

## ✅ Benefits of This Setup

| Feature | Benefit |
|---------|---------|
| **Matrix testing** | Ensures code works on all Python versions |
| **Code coverage** | Tracks test coverage over time |
| **Type checking** | Catches bugs before runtime |
| **Auto-publish to PyPI** | No manual upload needed for releases |
| **Docker image** | Automatic container builds on release |
| **Branch protection** | Only main/develop trigger full CI |

---

## 📊 Example Output

```
✓ python-package-conda / build (3.8)     — All checks passed
✓ python-package-conda / build (3.9)     — All checks passed  
✓ python-package-conda / build (3.10)    — All checks passed
✓ python-package-conda / build (3.11)    — All checks passed
✓ python-package-conda / build (3.12)    — All checks passed
✓ python-package-conda / build-and-publish — Published to PyPI ✅
✓ python-package-conda / docker-build    — Pushed to Docker Hub ✅
```

---

## 🆘 Troubleshooting

### "Permission denied" error
→ Make sure `DOCKER_PASSWORD` and `PYPI_API_TOKEN` are set correctly

### "Tests fail on Python 3.12"  
→ Remove from matrix, or fix code for 3.12 compatibility

### "Docker push fails"
→ Check `DOCKER_USERNAME` matches your actual Docker Hub username

---

**Ready to go live?** 🚀 Create a tag and watch the magic happen!
