import os
import csv
from sabre.targets import line_topology
from sabre.experiments import run_experiment

def main():
    os.makedirs("data/results", exist_ok=True)

    coupling_map = line_topology(8)
    rows = []

    for circuit_seed in range(10):
        for transpile_seed in range(10):
            _, _, row = run_experiment(
                coupling_map=coupling_map,
                num_qubits=8,
                circuit_depth=40,
                circuit_seed=circuit_seed,
                transpile_seed=transpile_seed,
            )
            rows.append(row)

    with open("data/results/sabre_results.csv", "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=rows[0].keys())
        writer.writeheader()
        writer.writerows(rows)

    print("saved data/results/sabre_results.csv")

if __name__ == "__main__":
    main()
