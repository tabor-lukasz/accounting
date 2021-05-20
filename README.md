Simple transations handling engine

Assumptions made:
- Only deposit transactions may be disputed
- One transaction may be disputed many times (after prior resolve)
- There is no need to check if a transaction id is globally unique


Correctness checked with unit tests.
Parsing correctnes delegated to serde crate.
Sample test data and result included in test_data dir.

This is single threaed application due to one input stream.

Source code checked with clippy and formated with fmt.
Documentation might be wider though:)
