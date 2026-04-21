# Cross-platform test runner wrapper
# Usage: .\RUN_TESTS.ps1 [-Category all|unit|python|go|java|javascript] [-Verbose] [-Pattern PATTERN]

param(
    [ValidateSet('all', 'unit', 'python', 'go', 'java', 'javascript', 'clustering', 'integration', 'performance')]
    [string]$Category = 'all',
    
    [switch]$Verbose,
    
    [string]$Pattern
)

# Build arguments for Python script
$args = @('scripts/run_tests.py')

if ($Category) {
    $args += @('--category', $Category)
}

if ($Verbose) {
    $args += '-v'
}

if ($Pattern) {
    $args += @('--pattern', $Pattern)
}

# Run the Python test runner
python @args
exit $LASTEXITCODE
