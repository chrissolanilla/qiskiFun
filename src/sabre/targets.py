from qiskit.transpiler import CouplingMap

def line_topology(num_qubits: int) -> CouplingMap:
    return CouplingMap.from_line(num_qubits)

def grid_topology(rows: int, cols: int) -> CouplingMap:
    return CouplingMap.from_grid(rows, cols)
