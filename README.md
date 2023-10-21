# ptchecker
A Model Checker on Petri Net

## build & run

```cargo build```

```cargo run -- INPUT_DIR```

Input directory should include following files:

model.pnml -- model file
LTLFireability.xml
LTLCardinality.xml
CTLFireability.xml
CTLCardinality.xml -- input formulas

There is one example in data, and more could be found in following link:

https://mcc.lip6.fr/2023/archives/INPUTS-2023.tar.gz