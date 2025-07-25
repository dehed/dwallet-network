# To add a new native Move function

1. Add a new `./ika-move-packages/{name}.move` file or find an appropriate `.move`.
2. Add the signature of the function you are adding in `{name}.move`. 
3. Add the rust implementation of the function under `./ika-framework/src/natives` with name `{name}.rs`.
4. Link the move interface with the native function in [all_natives](https://github.com/MystenLabs/sui/blob/main/crates/ika-framework/src/natives/mod.rs#L23)
5. Write some tests in `{name}_tests.move` and pass `run_framework_move_unit_tests`.
6. Optionally, update the mock move VM value in [gas_tests.rs](https://github.com/MystenLabs/sui/blob/276356e168047cdfce71814cb14403f4653a3656/crates/ika-core/src/unit_tests/gas_tests.rs) since the ika-framework package will increase the gas metering.
7. Optionally, run `cargo insta test` and `cargo insta review` since the ika-framework build will change the empty genesis config.

Note: The gas metering for native functions is currently a WIP; use a dummy value for now and please open an issue with `move` label.
