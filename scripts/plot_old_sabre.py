import os
import pandas as pd
import matplotlib.pyplot as plt

BENCHMARKS = [
    "bernstein-vazirani",
    "deutsch-jozsa",
    "quantum_fourier_transform",
    "shor's_order_finding",
]


def main():
    os.makedirs("data/results/plots", exist_ok=True)
    for slug in BENCHMARKS:
        csv_path = f"data/results/old_sabre_{slug}.csv"

        if not os.path.exists(csv_path):
            print(f"missing {csv_path}, skipping")
            continue

        df = pd.read_csv(csv_path)

        grouped = df.groupby("num_qubits", as_index=False).agg({
            "runtime_sec": "mean",
            "added_cx": "mean",
            "depth": "mean",
            "size": "mean",
        })

        pretty_name = slug.replace("_", " ").title()

        #runtime plot
        plt.figure(figsize=(7, 5))
        plt.plot(grouped["num_qubits"], grouped["runtime_sec"], marker="o")
        plt.yscale("log")
        plt.xlabel("Number of Qubits")
        plt.ylabel("Runtime (seconds, log scale)")
        plt.title(f"Old SABRE Runtime vs Qubits\n{pretty_name}")
        plt.grid(True, which="both", alpha=0.3)
        plt.tight_layout()
        plt.savefig(f"data/results/plots/{slug}_runtime.png", dpi=300)
        plt.close()

        #added cx plot
        plt.figure(figsize=(7, 5))
        plt.plot(grouped["num_qubits"], grouped["added_cx"], marker="o")
        plt.xlabel("Number of Qubits")
        plt.ylabel("Added CX Gates")
        plt.title(f"Old SABRE Added CX vs Qubits\n{pretty_name}")
        plt.grid(True, alpha=0.3)
        plt.tight_layout()
        plt.savefig(f"data/results/plots/{slug}_added_cx.png", dpi=300)
        plt.close()

        print(f"saved plots for {pretty_name}")


if __name__ == "__main__":
    main()
