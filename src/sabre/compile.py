from qiskit import transpile

def compile_with_sabre(circuit, coupling_map, basis_gates=None, optimization_level=3, seed=1234):
    return transpile(
        circuit,
        coupling_map=coupling_map,
        basis_gates=basis_gates or ["rz", "sx", "x", "cx"],
        layout_method="sabre",
        routing_method="sabre",
        optimization_level=optimization_level,
        seed_transpiler=seed,
    )
