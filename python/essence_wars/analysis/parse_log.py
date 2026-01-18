#!/usr/bin/env python3
"""Parse tuning log output and generate CSV for analysis."""

import csv
import re
import sys


def parse_tuning_log(log_path: str) -> list[dict]:
    """Parse tuning log and extract generation data."""
    data = []
    cumulative_time = 0.0

    pattern = re.compile(
        r'Gen\s+(\d+):\s+best_fit=\s*([\d.]+),\s+best_wr=\s*([\d.]+)%,\s+sigma=([\d.]+),\s+time=([\d.]+)s'
    )

    with open(log_path) as f:
        for line in f:
            match = pattern.search(line)
            if match:
                gen = int(match.group(1))
                fitness = float(match.group(2))
                win_rate = float(match.group(3))
                sigma = float(match.group(4))
                gen_time = float(match.group(5))
                cumulative_time += gen_time

                data.append({
                    'generation': gen,
                    'fitness': fitness,
                    'win_rate': win_rate,
                    'sigma': sigma,
                    'gen_time': gen_time,
                    'cumulative_time': cumulative_time,
                    'cumulative_minutes': cumulative_time / 60.0,
                })

    return data

def print_summary(data: list[dict]):
    """Print summary statistics."""
    if not data:
        print("No data found in log.")
        return

    print("\n=== Tuning Scaling Analysis ===\n")

    # Key milestones
    milestones = [10, 25, 50, 100, 150, 200, 250, 300]
    print("Fitness by Generation:")
    print("-" * 60)
    print(f"{'Gen':>6} {'Fitness':>10} {'WinRate':>10} {'Sigma':>10} {'Time(min)':>12}")
    print("-" * 60)

    for gen in milestones:
        if gen < len(data):
            d = data[gen]
            print(f"{d['generation']:>6} {d['fitness']:>10.2f} {d['win_rate']:>9.1f}% {d['sigma']:>10.4f} {d['cumulative_minutes']:>12.1f}")

    # Last entry
    if data:
        d = data[-1]
        print("-" * 60)
        print(f"{d['generation']:>6} {d['fitness']:>10.2f} {d['win_rate']:>9.1f}% {d['sigma']:>10.4f} {d['cumulative_minutes']:>12.1f}")

    # Improvement rates
    print("\n\nImprovement Analysis:")
    print("-" * 50)

    intervals = [(0, 10), (10, 25), (25, 50), (50, 100), (100, 200), (200, 300)]
    for start, end in intervals:
        if end < len(data):
            start_fit = data[start]['fitness']
            end_fit = data[end]['fitness']
            improvement = end_fit - start_fit
            time_spent = data[end]['cumulative_minutes'] - data[start]['cumulative_minutes']
            rate = improvement / time_spent if time_spent > 0 else 0
            print(f"Gen {start:>3}-{end:>3}: +{improvement:>6.2f} fitness in {time_spent:>5.1f} min ({rate:>5.2f}/min)")

def export_csv(data: list[dict], output_path: str):
    """Export data to CSV for graphing."""
    if not data:
        return

    with open(output_path, 'w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=data[0].keys())
        writer.writeheader()
        writer.writerows(data)

    print(f"\nCSV exported to: {output_path}")

def main():
    if len(sys.argv) < 2:
        print("Usage: python parse_tuning_log.py <log_file> [output.csv]")
        sys.exit(1)

    log_path = sys.argv[1]
    data = parse_tuning_log(log_path)

    print_summary(data)

    if len(sys.argv) > 2:
        export_csv(data, sys.argv[2])
    else:
        # Default output
        export_csv(data, 'tuning_scaling_data.csv')

if __name__ == '__main__':
    main()
