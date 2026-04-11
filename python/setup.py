"""
FastDataBroker Python SDK setup configuration
"""

from setuptools import setup, find_packages
import os

# Try to read README from multiple locations
readme_path = None
for path in ["README.md", "../README.md", "../../README.md"]:
    if os.path.exists(path):
        readme_path = path
        break

if readme_path:
    with open(readme_path, "r", encoding="utf-8") as fh:
        long_description = fh.read()
else:
    long_description = "FastDataBroker SDK for Python - Advanced messaging system with multi-channel notifications"

setup(
    name="FastDataBroker-sdk",
    version="0.1.14",
    author="FastDataBroker Team",
    author_email="suraj202923@gmail.com",
    description="FastDataBroker SDK for Python - Advanced messaging system with multi-channel notifications",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/suraj202923/FastDataBroker",
    project_urls={
        "Documentation": "https://github.com/suraj202923/FastDataBroker#documentation",
        "Bug Tracker": "https://github.com/suraj202923/FastDataBroker/issues",
        "Source Code": "https://github.com/suraj202923/FastDataBroker",
    },
    packages=find_packages(),
    py_modules=["fastdatabroker_sdk"],
    classifiers=[
        "Development Status :: 4 - Beta",
        "Environment :: Console",
        "Intended Audience :: Developers",
        "Intended Audience :: System Administrators",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Topic :: Software Development :: Libraries :: Python Modules",
        "Topic :: Communications :: Email",
        "Topic :: Internet",
    ],
    python_requires=">=3.8",
    install_requires=[
        # Phase 4: Add QUIC client dependencies
        # "quic-client>=1.0.0",
    ],
    extras_require={
        "dev": [
            "black>=23.0.0",
            "flake8>=6.0.0",
            "isort>=5.12.0",
            "mypy>=1.0.0",
            "pytest>=7.0.0",
            "pytest-asyncio>=0.20.0",
            "pytest-cov>=4.0.0",
        ],
    },
    entry_points={
        "console_scripts": [
            "FastDataBroker-py=FastDataBroker.cli:main",
        ],
    },
    keywords=[
        "FastDataBroker",
        "messaging",
        "notifications",
        "email",
        "websocket",
        "push notifications",
        "webhooks",
        "quic",
        "async",
    ],
    license="MIT",
    zip_safe=False,
    include_package_data=True,
)
