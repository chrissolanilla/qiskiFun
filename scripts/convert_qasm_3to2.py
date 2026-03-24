import os
import glob

from qiskit import transpile, qasm3
from qiskit.qasm2 import dumps

BENCHMARKS = [
    "Bernstein-Vazirani",
    "Deutsch-Jozsa",
    "Quantum Fourier Transform",
    "Shor's Order Finding",
]

def convert_family(family: str):
    in_dir = f"benchmark_circuits/{family}/qasm"
    out_dir = f"benchmark_circuits_qasm2_oldsafe/{family}/qasm"
    os.makedirs(out_dir, exist_ok=True)

    paths = sorted(glob.glob(os.path.join(in_dir, "*.qasm")))
    if not paths:
        print(f"no qasm files found for {family}")
        return

    for path in paths:
        qc = qasm3.load(path)
        qc_oldsafe = transpile(
            qc,
            basis_gates=["u1", "u2", "u3", "cx"],
            optimization_level=0,
        )
        out_text = dumps(qc_oldsafe)
        out_path = os.path.join(out_dir, os.path.basename(path))
        with open(out_path, "w") as f:
            f.write(out_text)
        print(f"converted {path} -> {out_path}")

def main():
    for family in BENCHMARKS:
        convert_family(family)

if __name__ == "__main__":
    main()
