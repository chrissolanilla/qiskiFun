def circuit_metrics(qc):
    ops = qc.count_ops()
    return {
        "depth": qc.depth(),
        "size": qc.size(),
        "num_qubits": qc.num_qubits,
        "cx_count": int(ops.get("cx", 0)),
        "swap_count": int(ops.get("swap", 0)),
        "measure_count": int(ops.get("measure", 0)),
    }
