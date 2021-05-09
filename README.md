Simple transations handling engine

Assumptions made:
- Only disputed transactions may be disputed
- One transaction may be disputed many times (after prior resolve)
- There is no need to check if a transaction id is globally unique


Correctness checked with unit tests.
Parsing correctnes delegated to serde crate.
Sample test data and result included in test_data dir.

This is single threaed application due to one input stream.
If it should work in multi threades enviroment then Engine and user functions (process_xyx ) should be modified to by async. Synchronization for users collection and particular users should be made by wrapping in Arc RwLock.

Errors are handled by returning Result<> of every operatioon that may fail. In case of transaction errors, transaction is ignored and error is just printed to stderr.

It may be good to consider changing internal type for money fields from f64 to some integral type by introducing some 1 base_unit = 0.0001 as rounding erors may occur for floats and 4 decimal precision is required.

Source code checked with clippy and formated with fmt.
Documentation might be wider though:)