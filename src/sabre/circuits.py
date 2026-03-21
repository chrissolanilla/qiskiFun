from qiskit import QuantumCircuit
import random

def random_cx_circuit(num_qubits: int, depth: int, seed: int) -> QuantumCircuit:
    rng = random.Random(seed)
    qc = QuantumCircuit(num_qubits)

    for _ in range(depth):
        gate_type = rng.choice(["h", "x", "rz", "cx"])

        if gate_type == "h":
            q = rng.randrange(num_qubits)
            qc.h(q)
        elif gate_type == "x":
            q = rng.randrange(num_qubits)
            qc.x(q)
        elif gate_type == "rz":
            q = rng.randrange(num_qubits)
            angle = rng.random() * 3.14159
            qc.rz(angle, q)
        else:
            q1, q2 = rng.sample(range(num_qubits), 2)
            qc.cx(q1, q2)

    qc.measure_all()
    return qc
