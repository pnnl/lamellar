Lamellar - Rust HPC runtime
=================================================

Lamellar is an asynchronous tasking runtime for HPC systems developed in RUST  
(Crates.io: https://crates.io/crates/lamellar)  
(Main runtime repository: https://github.com/pnnl/lamellar-runtime)  

SUMMARY
-------

Lamellar is an investigation of the applicability of the Rust systems programming language for HPC as an alternative to C and C++, with a focus on PGAS approaches.

Lamellar provides several different communication patterns to distributed applications. 
First, Lamellar allows for sending and executing active messages on remote nodes in a distributed environments. 
The runtime supports two forms of active messages:
The first method works with Stable rust and requires the user the register the active message by implementing a runtime exported trait (LamellarAM) and calling a procedural macro (\#[lamellar::am]) on the implementation.
The second method only works on nightly, but allows users to write serializable closures that are transfered and exectued by the runtime without registration 
It also exposes the concept of remote memory regions, i.e. allocations of memory that can read/written into by remote nodes.

Lamellar relies on network providers called Lamellae to perform the transfer of data throughout the system.
Currently two such Lamellae exist, one used for single node development ("local"), and another based on the Rust OpenFabrics Interface Transport Layer (ROFI) (https://github.com/pnnl/rofi)

REPOS
------
Runtime: https://github.com/pnnl/lamellar-runtime  
Benchmarks: https://github.com/pnnl/lamellar-benchmarks  
Rofi: https://github.com/pnnl/rofi  
Rofi-sys: https://github.com/pnnl/rofi-sys  

STATUS
------
Lamellar is still under development, thus not all intended features are yet
implemented.

CONTACTS
--------
Ryan Friese     - ryan.friese@pnnl.gov  
Roberto Gioiosa - roberto.gioiosa@pnnl.gov  
Mark Raugas     - mark.raugas@pnnl.gov  

## License

This project is licensed under the BSD License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

This work was supported by the High Performance Data Analytics (HPDA) Program at Pacific Northwest National Laboratory (PNNL),
a multi-program DOE laboratory operated by Battelle.
