import os
import csv
import glob
import time

from qiskit import QuantumCircuit
from qiskit.transpiler import CouplingMap, PassManager
from qiskit.transpiler.passes import SabreLayout, SabreSwap

BENCHMARKS = [
    "Bernstein-Vazirani",
    "Deutsch-Jozsa",
]

HEURISTICS = [
    "basic",
    "lookahead",
    "decay",
]


def family_slug(name: str) -> str:
    return (
        name.lower()
        .replace(" ", "_")
        .replace("-", "_")
        .replace("'", "")
    )


def line_coupling_map(n: int) -> CouplingMap:
    edges = []
    for i in range(n - 1):
        edges.append([i, i + 1])
        edges.append([i + 1, i])
    return CouplingMap(edges)


def count_ops_by_name(circ: QuantumCircuit, gate_name: str) -> int:
    return int(circ.count_ops().get(gate_name, 0))


def make_pass_manager(coupling: CouplingMap, heuristic: str) -> PassManager:
    return PassManager([
        SabreLayout(
            coupling_map=coupling,
            swap_trials=20,
            layout_trials=20,
            max_iterations=4,
            seed=0,
        ),
        SabreSwap(
            coupling_map=coupling,
            heuristic=heuristic,
            seed=0,
            trials=20,
        ),
    ])

def run_family_heuristic(family: str, heuristic: str):
    os.makedirs("data/results", exist_ok=True)

    slug = family_slug(family)
    out_csv = f"data/results/lightsabre_{slug}_{heuristic}.csv"

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
        pm = make_pass_manager(coupling, heuristic)

        print(f"[{family} | {heuristic}] running {name} with {n} qubits...")

        orig_cx = count_ops_by_name(qc, "cx")

        t0 = time.perf_counter()
        compiled = pm.run(qc)
        runtime = time.perf_counter() - t0

        comp_cx = count_ops_by_name(compiled, "cx")

        row = {
            "family": family,
            "heuristic": heuristic,
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
        for heuristic in HEURISTICS:
            run_family_heuristic(family, heuristic)


if __name__ == "__main__":
    main()
