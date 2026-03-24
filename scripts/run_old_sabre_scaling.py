import numpy as np

#ts pmo, numpy dep warnings
if not hasattr(np, "int"):
    np.int = int
if not hasattr(np, "product"):
    np.product = np.prod

import os
import csv
import glob
import time
import re

from qiskit import QuantumCircuit, transpile
from qiskit.transpiler import CouplingMap

BENCHMARKS = [
    "Bernstein-Vazirani",
    "Deutsch-Jozsa",
    # "Quantum Fourier Transform",
    # "Shor's Order Finding",
]

def family_slug(name: str) -> str:
    return name.lower().replace(" ", "_")

def line_coupling_map(n: int) -> CouplingMap:
    edges = []
    for i in range(n - 1):
        edges.append([i, i + 1])
        edges.append([i + 1, i])
    return CouplingMap(edges)

def count_ops_by_name(circ: QuantumCircuit, gate_name: str) -> int:
    return sum(1 for inst, qargs, cargs in circ.data if inst.name == gate_name)

def parse_qubits_from_name(path: str) -> int:
    name = os.path.basename(path)
    m = re.search(r'_(\d+)qubits_', name)
    if not m:
        raise ValueError(f"could not parse qubit count from filename: {name}")
    return int(m.group(1))

def run_family(family: str):
    os.makedirs("data/results", exist_ok=True)
    slug = family_slug(family)
    out_csv = f"data/results/old_sabre_{slug}.csv"
    qasm_paths = sorted(
        glob.glob(f"benchmark_circuits_qasm2_oldsafe/{family}/qasm/*.qasm")
    )

    if not qasm_paths:
        print(f"no benchmark qasm files found for {family}")
        return

    rows = []
    for path in qasm_paths:
        name = os.path.basename(path)
        qc = QuantumCircuit.from_qasm_file(path)
        n = qc.num_qubits
        coupling = line_coupling_map(n)
        print(f"[{family}] running {name} with {n} qubits...")

        orig_cx = count_ops_by_name(qc, "cx")
        t0 = time.perf_counter()
        compiled = transpile(
            qc,
            basis_gates=["u1", "u2", "u3", "cx"],
            coupling_map=coupling,
            optimization_level=3,
            layout_method="sabre",
            routing_method="sabre",
            seed_transpiler=0,
        )
        runtime = time.perf_counter() - t0
        comp_cx = count_ops_by_name(compiled, "cx")
        row = {
            "family": family,
            "file": name,
            "num_qubits": n,
            "runtime_sec": runtime,
            "orig_cx": orig_cx,
            "comp_cx": comp_cx,
            "added_cx": comp_cx - orig_cx,
            "depth": compiled.depth(),
            "size": compiled.size(),
        }
        rows.append(row)
        print(row)

    with open(out_csv, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=rows[0].keys())
        writer.writeheader()
        writer.writerows(rows)

    print(f"saved {out_csv}")

def main():
    for family in BENCHMARKS:
        run_family(family)

if __name__ == "__main__":
    main()
