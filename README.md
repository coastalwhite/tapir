<p align="center">
    <img alt="tapir logo" src="./assets/TapirLogoTitled.svg" />
</p>

A **T**est **P**rogram Generator for **R**ISC-V processor verification. This repository contains the source code that illustrates the our work on *Scalable Generation of Intricate RISC-V* programs.

This project aims to illustrate that processor fuzz input can be generated at much higher speeds and more intricate than the current state of the art. In a large part due to tight integration of the simulator, assembler and program generator.

This code is not meant for production usage and is made as part of a scientific research project.

# Overview

The code is divided up into several directories.

- `instr-encoding` contains the assembly / disassembly mapping for RISC-V instructions.
- `ivt-segment-tree` contains the memory backing datastructure.
- `program-gen` contains the code to generate programs
- `risico` contains the RISC-V ISA simulator
- `rsoftfloat` contains a wrapper around the Berkeley SoftFloat library to make it fit RISC-V.
- `rvisa` contains a datastructure to describe all RISC-V extensions.

# Acknowledgments

This project was written as part of a PhD at the TU Delft and under the EU Resilient Trust project.

# License

This project is licensed under an MIT license.
