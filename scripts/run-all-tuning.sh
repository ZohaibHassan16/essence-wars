#!/bin/bash
# Run all tuning commands sequentially with nice visuals
# Usage: ./scripts/run-all-tuning.sh [--quiet|-q]
#   --quiet, -q: Non-interactive mode with minimal output (AI-agent friendly)

set -e  # Exit on error

# Parse command-line arguments
QUIET_MODE=false
for arg in "$@"; do
    case $arg in
        -q|--quiet)
            QUIET_MODE=true
            shift
            ;;
        *)
            echo "Unknown argument: $arg"
            echo "Usage: $0 [--quiet|-q]"
            exit 1
            ;;
    esac
done

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
GRAY='\033[0;90m'
NC='\033[0m' # No Color

# Unicode symbols
CHECK="✓"
CROSS="✗"
ARROW="→"
STAR="★"
ROCKET="🚀"
FIRE="🔥"
BRAIN="🧠"
CHART="📊"
CLOCK="⏱️"
FOLDER="📁"

# Spinner characters
SPINNER=("⠋" "⠙" "⠹" "⠸" "⠼" "⠴" "⠦" "⠧" "⠇" "⠏")

# Track results
declare -a RESULTS
START_TIME=$(date +%s)

# Function to print header
print_header() {
    if [[ "$QUIET_MODE" == "true" ]]; then
        echo "$1"
    else
        echo -e "\n${WHITE}═══════════════════════════════════════════════════════════════════${NC}"
        echo -e "${CYAN}$1${NC}"
        echo -e "${WHITE}═══════════════════════════════════════════════════════════════════${NC}\n"
    fi
}

# Function to print section
print_section() {
    if [[ "$QUIET_MODE" == "true" ]]; then
        echo "$1"
    else
        echo -e "\n${BLUE}▶ $1${NC}"
    fi
}

# Function to print info
print_info() {
    if [[ "$QUIET_MODE" == "true" ]]; then
        return  # Skip info messages in quiet mode
    else
        echo -e "${GRAY}  $1${NC}"
    fi
}

# Function to print success
print_success() {
    if [[ "$QUIET_MODE" == "true" ]]; then
        echo "✓ $1"
    else
        echo -e "${GREEN}${CHECK} $1${NC}"
    fi
}

# Function to print error
print_error() {
    if [[ "$QUIET_MODE" == "true" ]]; then
        echo "ERROR: $1" >&2
    else
        echo -e "${RED}${CROSS} $1${NC}"
    fi
}

# Function to print warning
print_warning() {
    if [[ "$QUIET_MODE" == "true" ]]; then
        echo "WARNING: $1"
    else
        echo -e "${YELLOW}⚠ $1${NC}"
    fi
}

# Function to show spinner with live output (or simple progress in quiet mode)
run_with_spinner() {
    local cmd="$1"
    local desc="$2"
    local tag="$3"
    local log_file="tuning_${tag}.log"
    
    # Start the command in background
    eval "$cmd" > "$log_file" 2>&1 &
    local pid=$!
    
    local i=0
    local last_gen=""
    local last_fitness=""
    local last_wr=""
    local start_time=$(date +%s)
    
    if [[ "$QUIET_MODE" == "true" ]]; then
        # Quiet mode: just tail log and show every 5 generations
        echo "Starting: $desc"
        while kill -0 $pid 2>/dev/null; do
            if [[ -f "$log_file" ]]; then
                local latest=$(tail -20 "$log_file" | grep -E "Gen\s+[0-9]+:" | tail -1)
                if [[ -n "$latest" ]]; then
                    gen=$(echo "$latest" | grep -oP 'Gen\s+\K[0-9]+')
                    
                    # Show every 5 generations or gen 0
                    if [[ -n "$gen" ]] && [[ "$gen" != "$last_gen" ]] && (( gen % 5 == 0 )); then
                        echo "$latest"
                        last_gen="$gen"
                    fi
                fi
            fi
            sleep 1
        done
    else
        # Fancy mode: show spinner with live updates
        while kill -0 $pid 2>/dev/null; do
            # Parse latest generation info from log
            if [[ -f "$log_file" ]]; then
                local latest=$(tail -20 "$log_file" | grep -E "Gen\s+[0-9]+:" | tail -1)
                if [[ -n "$latest" ]]; then
                    # Extract gen, fitness, win rate
                    gen=$(echo "$latest" | grep -oP 'Gen\s+\K[0-9]+')
                    fitness=$(echo "$latest" | grep -oP 'best_fit=\s*\K[0-9.]+')
                    wr=$(echo "$latest" | grep -oP 'best_wr=\s*\K[0-9.]+')
                    
                    if [[ -n "$gen" ]] && [[ "$gen" != "$last_gen" ]]; then
                        last_gen="$gen"
                        last_fitness="$fitness"
                        last_wr="$wr"
                        
                        # Calculate elapsed time
                        local now=$(date +%s)
                        local elapsed=$((now - start_time))
                        local mins=$((elapsed / 60))
                        local secs=$((elapsed % 60))
                        
                        # Print update on same line
                        printf "\r${SPINNER[$i]} ${CYAN}$desc${NC} ${GRAY}|${NC} Gen ${WHITE}$gen${NC} ${GRAY}|${NC} Fitness: ${YELLOW}$fitness${NC} ${GRAY}|${NC} WR: ${GREEN}$wr%%${NC} ${GRAY}|${NC} ${CLOCK} ${mins}m ${secs}s  "
                    fi
                fi
            fi
            
            i=$(( (i + 1) % ${#SPINNER[@]} ))
            sleep 0.1
        done
        
        # Clear spinner line
        printf "\r$(printf ' %.0s' {1..120})\r"
    fi
    
    # Wait for command to finish and get exit code
    wait $pid
    local exit_code=$?
    
    if [[ $exit_code -eq 0 ]]; then
        # Parse final results
        local best_wr=$(tail -50 "$log_file" | grep "Best win rate:" | tail -1 | grep -oP '\d+\.\d+')
        local best_fitness=$(tail -50 "$log_file" | grep "Best fitness:" | tail -1 | grep -oP '\d+\.\d+')
        local total_time=$(tail -50 "$log_file" | grep "Total time:" | tail -1 | grep -oP '\d+\.\d+')
        local generations=$(tail -50 "$log_file" | grep "Generations:" | tail -1 | grep -oP '\d+')
        local deploy_path=$(tail -30 "$log_file" | grep "Auto-deployed to" | tail -1 | sed 's/.*Auto-deployed to "\(.*\)"/\1/')
        
        print_success "${desc} completed"
        
        if [[ "$QUIET_MODE" == "true" ]]; then
            echo "  Win Rate: ${best_wr}% | Fitness: ${best_fitness} | Time: ${total_time}s | Gens: ${generations}"
            if [[ -n "$deploy_path" ]]; then
                echo "  Deployed to: ${deploy_path}"
            fi
        else
            echo -e "  ${GRAY}${ARROW}${NC} Win Rate: ${GREEN}${best_wr}%${NC}  |  Fitness: ${YELLOW}${best_fitness}${NC}  |  Time: ${CYAN}${total_time}s${NC}  |  Gens: ${WHITE}${generations}${NC}"
            if [[ -n "$deploy_path" ]]; then
                echo -e "  ${GRAY}${ARROW}${NC} ${FOLDER} ${BLUE}${deploy_path}${NC}"
            fi
        fi
        
        # Store results
        RESULTS+=("${desc}|${best_wr}|${best_fitness}|${total_time}|${generations}")
        
        return 0
    else
        print_error "${desc} failed (exit code: $exit_code)"
        if [[ "$QUIET_MODE" != "true" ]]; then
            echo -e "  ${GRAY}See ${log_file} for details${NC}"
        else
            echo "  See ${log_file} for details"
        fi
        return 1
    fi
}

# Print banner
if [[ "$QUIET_MODE" != "true" ]]; then
    clear
    echo -e "${MAGENTA}"
    cat << "EOF"
  ╔═══════════════════════════════════════════════════════════════╗
  ║                                                               ║
  ║    ███████╗███████╗███████╗███████╗███╗   ██╗ ██████╗███████╗║
  ║    ██╔════╝██╔════╝██╔════╝██╔════╝████╗  ██║██╔════╝██╔════╝║
  ║    █████╗  ███████╗███████╗█████╗  ██╔██╗ ██║██║     █████╗  ║
  ║    ██╔══╝  ╚════██║╚════██║██╔══╝  ██║╚██╗██║██║     ██╔══╝  ║
  ║    ███████╗███████║███████║███████╗██║ ╚████║╚██████╗███████╗║
  ║    ╚══════╝╚══════╝╚══════╝╚══════╝╚═╝  ╚═══╝ ╚═════╝╚══════╝║
  ║                                                               ║
  ║              🎯 COMPREHENSIVE TUNING SUITE v0.4               ║
  ║                                                               ║
  ╚═══════════════════════════════════════════════════════════════╝
EOF
    echo -e "${NC}"
    
    print_header "${BRAIN} Essence Wars Bot Training Pipeline"
    
    print_info "This will train 4 configurations sequentially:"
    print_info "  1. ${STAR} Generalist     - Universal weights for all decks"
    print_info "  2. ${FIRE} Argentum       - Defensive control specialist"
    print_info "  3. ${FIRE} Symbiote       - Aggressive tempo specialist"
    print_info "  4. ${FIRE} Obsidion       - Burst damage specialist"
    echo ""
    print_info "Each run: 100 generations × 100 games vs Random/Greedy/MCTS"
    print_info "Estimated total time: ${CYAN}~40-60 minutes${NC}"
    echo ""
    
    # Ask for confirmation
    echo -e -n "${YELLOW}Continue? [y/N]${NC} "
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        print_warning "Aborted by user"
        exit 0
    fi
else
    # Quiet mode: just state what we're doing
    echo "Essence Wars Bot Training Pipeline"
    echo "Training 4 configurations: Generalist, Argentum, Symbiote, Obsidion"
    echo "100 generations × 100 games each"
    echo ""
fi

# Change to project root
cd "$(dirname "$0")/.."

# Verify cargo is available
if ! command -v cargo &> /dev/null; then
    print_error "cargo not found. Please install Rust."
    exit 1
fi

# Build first
print_section "Building release binary..."
if cargo build --release --bin tune --quiet 2>&1 | grep -i error; then
    print_error "Build failed"
    exit 1
fi
print_success "Build complete"

# Run each tuning configuration
print_header "${ROCKET} Training Phase 1/4: Generalist"
run_with_spinner \
    "cargo run --release --bin tune -- --tag generalist-v0.4 --mode generalist --generations 100 --games 100 --mcts-sims 50" \
    "Generalist Training" \
    "generalist"

print_header "${ROCKET} Training Phase 2/4: Argentum Specialist"
run_with_spinner \
    "cargo run --release --bin tune -- --tag argentum-specialist-v0.4 --mode faction-specialist --faction argentum --generations 100 --games 100 --mcts-sims 50" \
    "Argentum Specialist Training" \
    "argentum"

print_header "${ROCKET} Training Phase 3/4: Symbiote Specialist"
run_with_spinner \
    "cargo run --release --bin tune -- --tag symbiote-specialist-v0.4 --mode faction-specialist --faction symbiote --generations 100 --games 100 --mcts-sims 50" \
    "Symbiote Specialist Training" \
    "symbiote"

print_header "${ROCKET} Training Phase 4/4: Obsidion Specialist"
run_with_spinner \
    "cargo run --release --bin tune -- --tag obsidion-specialist-v0.4 --mode faction-specialist --faction obsidion --generations 100 --games 100 --mcts-sims 50" \
    "Obsidion Specialist Training" \
    "obsidion"

# Calculate total time
END_TIME=$(date +%s)
TOTAL_TIME=$((END_TIME - START_TIME))
TOTAL_MINS=$((TOTAL_TIME / 60))
TOTAL_SECS=$((TOTAL_TIME % 60))

# Print summary
print_header "Training Summary"

if [[ "$QUIET_MODE" == "true" ]]; then
    # Quiet mode: simple text table
    echo "FINAL RESULTS"
    echo "─────────────────────────────────────────────────────────────────"
    for result in "${RESULTS[@]}"; do
        IFS='|' read -r name wr fitness time gens <<< "$result"
        printf "%-25s WR: %5.1f%%  F: %6.2f  %6.1fs  %3d gen\n" \
            "$name" "$wr" "$fitness" "$time" "$gens"
    done
    echo "─────────────────────────────────────────────────────────────────"
    printf "Total Time: %d minutes %d seconds\n" "$TOTAL_MINS" "$TOTAL_SECS"
    echo ""
    echo "✓ All training runs completed!"
    echo ""
    echo "Weights deployed to:"
    echo "  → data/weights/generalist.toml"
    echo "  → data/weights/specialists/argentum.toml"
    echo "  → data/weights/specialists/symbiote.toml"
    echo "  → data/weights/specialists/obsidion.toml"
    echo ""
else
    # Fancy mode: keep original formatting
    echo -e "${WHITE}┌─────────────────────────────────────────────────────────────────┐${NC}"
    echo -e "${WHITE}│                     ${CYAN}FINAL RESULTS${WHITE}                            │${NC}"
    echo -e "${WHITE}├─────────────────────────────────────────────────────────────────┤${NC}"
    
    for result in "${RESULTS[@]}"; do
        IFS='|' read -r name wr fitness time gens <<< "$result"
        printf "${WHITE}│${NC} %-25s ${GREEN}WR: %5.1f%%${NC}  ${YELLOW}F: %6.2f${NC}  ${CYAN}%6.1fs${NC} ${GRAY}%3d gen${NC} ${WHITE}│${NC}\n" \
            "$name" "$wr" "$fitness" "$time" "$gens"
    done
    
    echo -e "${WHITE}├─────────────────────────────────────────────────────────────────┤${NC}"
    printf "${WHITE}│${NC} ${CLOCK} Total Time: ${CYAN}%d minutes %d seconds${WHITE}                        │${NC}\n" "$TOTAL_MINS" "$TOTAL_SECS"
    echo -e "${WHITE}└─────────────────────────────────────────────────────────────────┘${NC}"
    
    echo ""
    print_success "All training runs completed!"
    echo ""
    print_info "Weights deployed to:"
    print_info "  ${ARROW} ${BLUE}data/weights/generalist.toml${NC}"
    print_info "  ${ARROW} ${BLUE}data/weights/specialists/argentum.toml${NC}"
    print_info "  ${ARROW} ${BLUE}data/weights/specialists/symbiote.toml${NC}"
    print_info "  ${ARROW} ${BLUE}data/weights/specialists/obsidion.toml${NC}"
    echo ""
    print_info "Experiment data saved to:"
    print_info "  ${ARROW} ${GRAY}experiments/mcts/$(date +%Y-%m-%d)_*/${NC}"
    echo ""
    print_info "Next steps:"
    print_info "  1. Run ${CYAN}./scripts/analyze-tuning.sh --all${NC} to generate visualizations"
    print_info "  2. Test weights with ${CYAN}cargo run --release --bin arena${NC}"
    print_info "  3. Commit weights if performance improved"
    echo ""
    
    echo -e "${GREEN}${STAR}${STAR}${STAR} Training pipeline complete! ${STAR}${STAR}${STAR}${NC}\n"
fi

# Cleanup temp logs
rm -f tuning_*.log
