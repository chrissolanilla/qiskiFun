#import pandas as pd
#import matplotlib.pyplot as plt
#csvFiles = ["data/results/lightsabre_deutsch_jozsa_decay.csv",
#        "data/results/lightsabre_deutsch_jozsa_basic.csv",
#        "data/results/lightsabre_deutsch_jozsa_lookahead.csv"
#    ]

#for file in csvFiles:
#    df = pd.read_csv(file)
#    grouped = df.groupby("num_qubits", as_index=False).agg({
#        "runtime_sec": "mean",
#        "added_cx": "mean",
#        "depth": "mean",
#    })

#    grouped.to_csv(f"{file}.csv")

##### old shit
#df = pd.read_csv("data/results/lightsabre_deutsch_jozsa_decay.csv")

#grouped = df.groupby("num_qubits", as_index=False).agg({
#    "runtime_sec": "mean",
#    "added_cx": "mean",
#    "depth": "mean",
#})

##runtime plot
#plt.figure(figsize=(7, 5))
#plt.plot(grouped["num_qubits"], grouped["runtime_sec"], marker="o")
#plt.yscale("log")
#plt.xlabel("Number of Qubits")
#plt.ylabel("Runtime (seconds, log scale)")
#plt.title("LightSABRE (decay) Runtime vs Qubits")
#plt.grid(True, which="both", alpha=0.3)
#plt.tight_layout()
#plt.savefig("data/results/lightPlots/lightsabre_runtime_decay.png", dpi=300)
#print("saved LightSABRE time plot")

##added cx plot
#plt.figure(figsize=(7, 5))
#plt.plot(grouped["num_qubits"], grouped["added_cx"], marker="o")
#plt.xlabel("Number of Qubits")
#plt.ylabel("Added CX Gates")
#plt.title("LightSABRE (decay) Added CX vs Qubits")
#plt.grid(True, alpha=0.3)
#plt.tight_layout()
#plt.savefig("data/results/lightPlots/lightsabre_added_cx_decay.png", dpi=300)

#print("saved LightSABRE cx plot")

import pandas as pd
import matplotlib.pyplot as plt
import os

csv_files = [
    "data/results/lightsabre_deutsch_jozsa_decay.csv",
    "data/results/lightsabre_deutsch_jozsa_basic.csv",
    "data/results/lightsabre_deutsch_jozsa_lookahead.csv",
]

labels = ["decay", "basic", "lookahead"]

all_data = {}
for file, label in zip(csv_files, labels):
    df = pd.read_csv(file)

    grouped = df.groupby("num_qubits", as_index=False).agg({
        "runtime_sec": "mean",
        "added_cx": "mean",
        "depth": "mean",
    })

    all_data[label] = grouped


os.makedirs("data/readded sults/lightPlots", exist_ok=True)

#runtime plot
plt.figure(figsize=(7, 5))

for label, data in all_data.items():
    plt.plot(data["num_qubits"], data["runtime_sec"], marker="o", label=label)

plt.yscale("log")
plt.xlabel("Number of Qubits")
plt.ylabel("Runtime (seconds, log scale)")
plt.title("LightSABRE Runtime vs Qubits")
plt.legend()
plt.grid(True, which="both", alpha=0.3)
plt.tight_layout()
plt.savefig("data/results/lightPlots/runtime_comparison.png", dpi=300)

print("saved runtime comparison")


#CX plot
plt.figure(figsize=(7, 5))

for label, data in all_data.items():
    plt.plot(data["num_qubits"], data["added_cx"], marker="o", label=label)

plt.xlabel("Number of Qubits")
plt.ylabel("Added CX Gates")
plt.title("LightSABRE Added CX vs Qubits")
plt.legend()
plt.grid(True, alpha=0.3)
plt.tight_layout()
plt.savefig("data/results/lightPlots/added_cx_comparison.png", dpi=300)

print("saved added cx comparison")


#depth plot
plt.figure(figsize=(7, 5))

for label, data in all_data.items():
    plt.plot(data["num_qubits"], data["depth"], marker="o", label=label)

plt.xlabel("Number of Qubits")
plt.ylabel("Circuit Depth")
plt.title("LightSABRE Depth vs Qubits")
plt.legend()
plt.grid(True, alpha=0.3)
plt.tight_layout()
plt.savefig("data/results/lightPlots/depth_comparison.png", dpi=300)

print("saved depth comparison")
