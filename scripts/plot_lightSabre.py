import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv("data/results/lightsabre_deutsch_jozsa_decay.csv")

grouped = df.groupby("num_qubits", as_index=False).agg({
    "runtime_sec": "mean",
    "added_cx": "mean",
    "depth": "mean",
})

#runtime plot
plt.figure(figsize=(7, 5))
plt.plot(grouped["num_qubits"], grouped["runtime_sec"], marker="o")
plt.yscale("log")
plt.xlabel("Number of Qubits")
plt.ylabel("Runtime (seconds, log scale)")
plt.title("LightSABRE (decay) Runtime vs Qubits")
plt.grid(True, which="both", alpha=0.3)
plt.tight_layout()
plt.savefig("data/results/lightPlots/lightsabre_runtime_decay.png", dpi=300)

#added cx plot
plt.figure(figsize=(7, 5))
plt.plot(grouped["num_qubits"], grouped["added_cx"], marker="o")
plt.xlabel("Number of Qubits")
plt.ylabel("Added CX Gates")
plt.title("LightSABRE (decay) Added CX vs Qubits")
plt.grid(True, alpha=0.3)
plt.tight_layout()
plt.savefig("data/results/lightPlots/lightsabre_added_cx_decay.png", dpi=300)

print("saved LightSABRE plots")
