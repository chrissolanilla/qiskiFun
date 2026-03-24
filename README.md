# welcome to Qiskit fun
this is a repo that has lots of things like my brute force rust approach to transpiling circuits
to also images of plots and data figures of old sabre and lightSabre

## you will need to have two different versions of qiskit to benchmark old vs new qiskit sabre
steps:
get old qiskit, probably use tmux and pyenv for convenience
```
pyenv virtualenv 3.10.13 qiskit020
pyenv activate qiskit020

pip install "qiskit==0.20.0"
pip install "numpy==1.23.5"
pip install "setuptools<81"
```
dont upgrade packages randomly

## make a regular modern env for plotting and stuff
```
pyenv virtualenv 3.13.0 qiskit_new
pyenv activate qiskit_new

pip install qiskit
pip install matplotlib pandas
```

## convert old qasm v3 to v2 for old qiskit
```
pyenv activate qiskit_new
python scripts/convert_qasm3_to_qasm2.py
```

switch to the old env and run
```
python scripts/run_old_sabre_scaling.py
```

that will generate `data/results/old_sabre_*.csv`

next run:
```
python scripts/run_lightSabre_benchmarks.py
```

now you can plot your results:
```
python scripts/plot_old_sabre.py
python scripts/plot_lightsabre.py
```

## for rust
just have rust and cargo intalled.
cd into rust and do cargo run. it will provide you with help there with some commadns availbile for options

you can also view the run.txt file
