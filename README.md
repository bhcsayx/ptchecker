# ptchecker
Purdue CS560 course project, a model checker on petri net written in rust

# Prerequisites
Recent stable rust build should be fine.

## Build & run
To build the tool, run:
```cargo build```

To run:
```cargo run --bin BINARY_NAME -- INPUT_DIR```

Here BINARY_NAME could be either ```standalone```(For CTL checking) or ```ltl```(For LTL checking)

INPUT_DIR should be a directory including following files:

model.pnml -- model file
LTLFireability.xml
CTLFireability.xml -- input formulas
(We did not implement checking for cardinality properties due to time limit)

There are two examples in data, and more could be found in following link:

https://mcc.lip6.fr/2023/archives/INPUTS-2023.tar.gz

(To get results shown in presentation, please use "SatelliteMemory-PT-X00010Y0003" as INPUT_DIR)
