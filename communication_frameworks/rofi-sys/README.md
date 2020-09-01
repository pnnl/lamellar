# rofi-sys #

This system crate provides Rust language bindings (via the use of Bindgen) to the Rust-OFI library.

## Dependencies

**rofi-sys** has the following dependencies:

* gcc 4.8.5
* openmpi 2.1.1
* clang 5.0.1
* rust-ofi 0.1

The **OFI_DIR** environment variable must be specified with the location of the OFI installation.

The **ROFI_DIR** environment variable must be specified with the location of the Rust-OFI installation.

## Authors

* **Mark Raugas**, PNNL
* **Ryan Friese**, PNNL
* **Roberto Gioiosa**, PNNL

## License

This project is licensed under the BSD License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

This work was supported by the High Performance Data Analytics (HPDA) Program at Pacific Northwest National Laboratory (PNNL),
a multi-program DOE laboratory operated by Battelle.
