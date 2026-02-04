#!/usr/bin/env bash
set -euo pipefail

# --- Paths ---
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROGRAMS_DIR="$SCRIPT_DIR/programs"
RESULTS_DIR="$SCRIPT_DIR/results"
OUTPUT_FILE="$SCRIPT_DIR/README.md"

# --- Defaults ---
WARMUP=3
RUNS=10

# --- Parse arguments ---
while [[ $# -gt 0 ]]; do
  case $1 in
  --warmup)
    if [[ $# -lt 2 ]]; then
      echo "Error: --warmup requires a value" >&2
      exit 1
    fi
    WARMUP="$2"
    shift 2
    ;;
  --runs)
    if [[ $# -lt 2 ]]; then
      echo "Error: --runs requires a value" >&2
      exit 1
    fi
    RUNS="$2"
    shift 2
    ;;
  -h | --help)
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --warmup N      Number of warmup runs (default: 3)"
    echo "  --runs N        Number of benchmark runs (default: 10)"
    echo "  -h, --help      Show this help"
    exit 0
    ;;
  *)
    echo "Error: Unknown option: $1" >&2
    exit 1
    ;;
  esac
done

if ! [[ "$WARMUP" =~ ^[0-9]+$ ]] || ! [[ "$RUNS" =~ ^[0-9]+$ ]]; then
  echo "Error: --warmup and --runs must be positive integers" >&2
  exit 1
fi

# --- Dependency checks ---
check_required() {
  if ! command -v "$1" &>/dev/null; then
    echo "Error: $1 is required but not found. Please install it." >&2
    exit 1
  fi
}

check_required hyperfine
check_required jq
check_required merx

check_required python3
check_required node
check_required ruby
check_required go
check_required rustc

# --- Collect language versions ---
declare -A LANG_VERSIONS=()
LANG_VERSIONS[merx]="$(merx --version)"
LANG_VERSIONS[python]="$(python3 --version)"
LANG_VERSIONS[node]="Node.js $(node --version)"
LANG_VERSIONS[ruby]="Ruby $(ruby --version | awk '{print $2}')"
LANG_VERSIONS[go]="$(go version | awk '{print $3}' | sed 's/^go/Go /')"
LANG_VERSIONS[rust]="$(rustc --version | awk '{print "Rust", $2}')"

MERX_BIN="$(command -v merx)"

# --- Build compiled programs ---
BUILD_DIR=$(mktemp -d)
echo "Compiling Go programs..."
go build -o "$BUILD_DIR/fizzbuzz_go" "$PROGRAMS_DIR/fizzbuzz/fizzbuzz.go"
go build -o "$BUILD_DIR/fibonacci_go" "$PROGRAMS_DIR/fibonacci/fibonacci.go"
go build -o "$BUILD_DIR/gcdsum_go" "$PROGRAMS_DIR/gcdsum/gcdsum.go"
go build -o "$BUILD_DIR/primecount_go" "$PROGRAMS_DIR/primecount/primecount.go"
go build -o "$BUILD_DIR/collatz_go" "$PROGRAMS_DIR/collatz/collatz.go"
echo "Compiling Rust programs..."
rustc -O -o "$BUILD_DIR/fizzbuzz_rust" "$PROGRAMS_DIR/fizzbuzz/fizzbuzz.rs"
rustc -O -o "$BUILD_DIR/fibonacci_rust" "$PROGRAMS_DIR/fibonacci/fibonacci.rs"
rustc -O -o "$BUILD_DIR/gcdsum_rust" "$PROGRAMS_DIR/gcdsum/gcdsum.rs"
rustc -O -o "$BUILD_DIR/primecount_rust" "$PROGRAMS_DIR/primecount/primecount.rs"
rustc -O -o "$BUILD_DIR/collatz_rust" "$PROGRAMS_DIR/collatz/collatz.rs"

cleanup() {
  rm -rf "$BUILD_DIR"
}
trap cleanup EXIT

# --- Prepare results directory ---
mkdir -p "$RESULTS_DIR"

# --- Helper: build hyperfine args for a benchmark ---
build_hyperfine_args() {
  local benchmark="$1"

  local -a args=()
  args+=(--warmup "$WARMUP" --runs "$RUNS")
  args+=(--export-json "$RESULTS_DIR/${benchmark}.json")

  # Verify all program files exist
  local -a required_files=(
    "${PROGRAMS_DIR}/${benchmark}/${benchmark}.mmd"
    "${PROGRAMS_DIR}/${benchmark}/${benchmark}.py"
    "${PROGRAMS_DIR}/${benchmark}/${benchmark}.js"
    "${PROGRAMS_DIR}/${benchmark}/${benchmark}.rb"
    "${BUILD_DIR}/${benchmark}_go"
    "${BUILD_DIR}/${benchmark}_rust"
  )
  for f in "${required_files[@]}"; do
    if [[ ! -f "$f" ]]; then
      echo "Error: Required benchmark file not found: $f" >&2
      exit 1
    fi
  done

  args+=(-n "merx" "${MERX_BIN} run ${PROGRAMS_DIR}/${benchmark}/${benchmark}.mmd > /dev/null")
  args+=(-n "python" "python3 ${PROGRAMS_DIR}/${benchmark}/${benchmark}.py > /dev/null")
  args+=(-n "node" "node ${PROGRAMS_DIR}/${benchmark}/${benchmark}.js > /dev/null")
  args+=(-n "ruby" "ruby ${PROGRAMS_DIR}/${benchmark}/${benchmark}.rb > /dev/null")
  args+=(-n "go" "${BUILD_DIR}/${benchmark}_go > /dev/null")
  args+=(-n "rust" "${BUILD_DIR}/${benchmark}_rust > /dev/null")

  printf '%s\n' "${args[@]}"
}

# --- Run benchmarks ---
echo ""
echo "=== Running FizzBuzz benchmark ==="
mapfile -t fizzbuzz_args < <(build_hyperfine_args "fizzbuzz")
hyperfine "${fizzbuzz_args[@]}"

echo ""
echo "=== Running Fibonacci benchmark (n=30) ==="
mapfile -t fibonacci_args < <(build_hyperfine_args "fibonacci")
hyperfine "${fibonacci_args[@]}"

echo ""
echo "=== Running GCD Sum benchmark (n=100) ==="
mapfile -t gcdsum_args < <(build_hyperfine_args "gcdsum")
hyperfine "${gcdsum_args[@]}"

echo ""
echo "=== Running Prime Count benchmark (n=10000) ==="
mapfile -t primecount_args < <(build_hyperfine_args "primecount")
hyperfine "${primecount_args[@]}"

echo ""
echo "=== Running Collatz Conjecture benchmark (n=10000) ==="
mapfile -t collatz_args < <(build_hyperfine_args "collatz")
hyperfine "${collatz_args[@]}"

# --- Generate Markdown report ---
echo ""
echo "Generating report..."

# System info
get_system_info() {
  local os_info cpu_info
  os_info="$(uname -srm)"
  if [[ "$(uname -s)" == "Darwin" ]]; then
    cpu_info="$(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo 'Unknown')"
  else
    cpu_info="$(grep -m1 'model name' /proc/cpuinfo 2>/dev/null | cut -d: -f2 | xargs || echo 'Unknown')"
  fi
  echo "- **OS**: ${os_info}"
  echo "- **CPU**: ${cpu_info}"
  echo "- **Date**: $(date +%Y-%m-%d)"
}

# Build versions JSON safely using jq
build_versions_json() {
  jq -n \
    --arg merx "${LANG_VERSIONS[merx]}" \
    --arg python "${LANG_VERSIONS[python]}" \
    --arg node "${LANG_VERSIONS[node]}" \
    --arg ruby "${LANG_VERSIONS[ruby]}" \
    --arg go "${LANG_VERSIONS[go]}" \
    --arg rust "${LANG_VERSIONS[rust]}" \
    '{merx: $merx, python: $python, node: $node, ruby: $ruby, go: $go, rust: $rust}'
}

# Generate table from hyperfine JSON with version as language name
generate_table() {
  local json_file="$1"
  local versions_json="$2"

  echo "| Language | Mean (ms) | Min (ms) | Max (ms) | Relative |"
  echo "|----------|-----------|----------|----------|----------|"

  jq -r --argjson versions "$versions_json" '
    (.results | map(.mean) | min) as $min_mean |
    .results | sort_by(.mean)[] |
    ($versions[.command] // .command) as $ver |
    (if .command == "merx" then "| **\($ver)** | **\((.mean * 1000 * 100 | round) / 100)** | **\((.min * 1000 * 100 | round) / 100)** | **\((.max * 1000 * 100 | round) / 100)** | **\((.mean / $min_mean * 100 | round) / 100)** |"
     else "| \($ver) | \((.mean * 1000 * 100 | round) / 100) | \((.min * 1000 * 100 | round) / 100) | \((.max * 1000 * 100 | round) / 100) | \((.mean / $min_mean * 100 | round) / 100) |"
     end)
  ' "$json_file"
}

VERSIONS_JSON="$(build_versions_json)"
TEMP_OUTPUT="$(mktemp)"

{
  echo "# merx Benchmarks"
  echo ""
  echo "## System Information"
  echo ""
  get_system_info
  echo ""
  echo "## Configuration"
  echo ""
  echo "- **Warmup runs**: ${WARMUP}"
  echo "- **Benchmark runs**: ${RUNS}"
  echo "- **Tool**: [hyperfine](https://github.com/sharkdp/hyperfine)"
  echo ""

  echo "## FizzBuzz (n=1..100)"
  echo ""
  echo "Programs: [./programs/fizzbuzz/](./programs/fizzbuzz/)"
  echo ""
  generate_table "$RESULTS_DIR/fizzbuzz.json" "$VERSIONS_JSON"
  echo ""

  echo "## Fibonacci (n=30)"
  echo ""
  echo "Programs: [./programs/fibonacci/](./programs/fibonacci/)"
  echo ""
  generate_table "$RESULTS_DIR/fibonacci.json" "$VERSIONS_JSON"
  echo ""

  echo "## GCD Sum (n=100)"
  echo ""
  echo "Programs: [./programs/gcdsum/](./programs/gcdsum/)"
  echo ""
  generate_table "$RESULTS_DIR/gcdsum.json" "$VERSIONS_JSON"
  echo ""

  echo "## Prime Count (n=10000)"
  echo ""
  echo "Programs: [./programs/primecount/](./programs/primecount/)"
  echo ""
  generate_table "$RESULTS_DIR/primecount.json" "$VERSIONS_JSON"
  echo ""

  echo "## Collatz Conjecture (n=10000)"
  echo ""
  echo "Programs: [./programs/collatz/](./programs/collatz/)"
  echo ""
  generate_table "$RESULTS_DIR/collatz.json" "$VERSIONS_JSON"
  echo ""
} >"$TEMP_OUTPUT"

mv "$TEMP_OUTPUT" "$OUTPUT_FILE"
echo "Report written to: $OUTPUT_FILE"
