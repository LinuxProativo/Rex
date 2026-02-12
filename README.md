<p align="center">
  <img src="logo.png" width="280">
</p>

<h1 align="center">🦖 Rex - Static Rust Executable Generator and Runtime</h1> 
<h3 align="center">A high-performance, minimalist application bundler for Linux.</h3>

<p align="center">
  <img src="https://img.shields.io/badge/Size-954.6_KiB-brightgreen" alt="Size">
  <img src="https://img.shields.io/badge/Language-Rust_2024-orange" alt="Rust">
  <img src="https://img.shields.io/badge/Build-Makefile_/_Cargo-blue" alt="Build">
  <img src="https://img.shields.io/badge/License-MIT-blue" alt="License">
</p>

## 📖 Overview

**Rex** 🦖 is a specialized utility designed to transform dynamic Linux executables
into portable, self-contained units. Unlike traditional Flatpaks, AppImages or
containers, Rex targets the binary level by "stitching" the application, its
shared library dependencies (`.so`), extra helpers, and assets into a single
executable wrapper.

At runtime, **Rex** identifies its own payload, extracts it to a secure temporary
location, and executes the target using a bundled dynamic loader (`glibc` or `musl`).
This bypasses "dependency hell" by ensuring the application always runs against
the exact environment it was packaged with.

## ✨ Key Features

- **Ultra-Minimalist** 🪶  
  Total binary footprint of **`~954.6 KiB`**.

- **Zero External Runtime Deps** 🔋  
  Built with musl for static linking. No ldd or clap required at runtime.

- **Deep ELF Inspection** 🔍  
  Automatically resolves shared library trees using internal logic (rldd-rex).

- **Universal Compatibility** 🌍  
  Bundles the required dynamic loader to run across different Linux distributions.

- **Industrial Grade Compression** ⚡  
  Uses Zstd with Long Distance Matching (LDM) for maximum payload reduction.

- **Clean Execution**🔒  
  Automatic cleanup of temporary files upon process exit.

## 🏗️ Project Architecture

- `main.rs` **➜ Bootstrap**  
  Detects execution mode *(Builder vs. Stub)* and handles CLI parsing.

- `generator.rs` **➜ The Packer**  
  Performs staging, dependency resolution, and footer injection.

- `runtime.rs` **➜ The Stub**  
  Performs backwards footer scanning and managed execution via the bundled loader.

## 🛠️ Building

**Rex** uses a `Makefile` to orchestrate optimized builds.

### Requisites

- 📌 Rust Stable (2024 Edition).
- 📌 Linux environment.

### Build Command

```bash
make
# or manually:
cargo build --release
```

## 🕹️ CLI Usage (Builder Mode)

**Rex** features a custom, lightweight argument parser designed for speed and 
small binary size.

```bash
# Basic packaging
./Rex -t ./my_app

# Full-featured bundle
./Rex \
  -t ./my_app \
  -L 19 \
  -l /usr/lib/custom_lib.so \
  -b ./helper_tool \
  -f ./config_folder_or_files
```

## ⚙️ Options:

- `-t <file>`: Target binary to bundle **(Required)**.

- `-L <num>`: Zstd compression level (1–22, default: 5).

- `-l <file>`: Explicitly include additional shared libraries.

- `-b <file>`: Extra binaries **(Rex will also resolve their dependencies)**.

- `-f <path>`: Additional files or directories to include in the bundle root.

## ⚙️ Advanced Loader Handling

**Rex** ensures portability by managing the Linux dynamic linking process manually:

- **Bundled Loader** 📦  
  The generator locates the system loader (`ld-linux-x86-64.so.2` or
  `ld-musl-x86_64.so.1`) and includes it in `libs/`.

- **Execution Hijacking** 🎭  
  The runtime does not call the binary directly. Instead, it invokes the bundled
  loader and uses the `--library-path` flag to point to the extracted `libs/`
  directory. This ensures the target binary cannot link against incompatible
  host libraries.

- **Path Resolution** 🗺️  
  The `PATH` environment variable is temporarily prefixed with the internal `bins/`
  directory, allowing the target binary to call bundled helper tools seamlessly.

## 🏃 Runtime Behavior

When you execute a generated `.Rex` bundle:

1. **Extraction** 📂  
  The payload is extracted to `/tmp`.

2. **Environment Setup** 🛠️  
  Prefixes `PATH` with bundled binaries and configures the loader path.

3. **Managed Run** ⚡  
  Invokes the bundled loader to execute the target binary.

4. **Cleanup** 🧹  
  Automatically wipes the extraction directory once the app exits.

## 🛠️ Debug Features

- `--rex-extract`: Extracts the bundle into the **current directory**.

> ### *Note: This flag is only available in development builds (debug assertions enabled).*

## 📂 Internal Bundle Layout

The internal structure is optimized for loader resolution:

```
<target>_bundle/
├─ <target>    # Primary executable
├─ bins/       # Helper binaries (-b)
├─ libs/       # Shared libraries + Dynamic Loader (ld-linux/musl) + Extra libraries (-l)
└─ [assets]    # Files added via the -f flag
```

The runtime expects exactly this layout and looks up the active bundle
using the `target` name provided in the appended metadata.

## ⚠️ Important Considerations

- **Not an AppImage Alternative** 🚫  
  Rex is not designed to be an "AppImage-like" general-purpose desktop format.
  It is intended for specific use cases where static compilation is unfeasible or
  impossible **(e.g., closed-source/proprietary libraries or complex C dependencies)**.

- **Native Musl Optimization** 💎  
  Rex is natively configured for the `x86_64-unknown-linux-musl` target. Using Rex
  in a **musl-based environment (such as Alpine Linux)** to package your apps (`-t`) 
  yields superior results. Since musl libraries are significantly more lightweight
  than glibc, the resulting bundled payload is much smaller and the runtime remains
  completely static with zero reliance on the host.

- **Execution Overhead** ⏳  
  Because Rex extracts its payload to a temporary directory at every run, 
  there is a slight startup delay compared to an original static binary.

- **No Extra Environment Support** 🌐  
  Support for external environment variables to search for resources or dynamic
  paths will not be implemented. Rex is built for fixed, reliable execution
  environments.

### 🤝 Contribution Guidelines

- 🐛 Contributions are welcome only for **bug fixes** and **binary size reduction**.

- 📉 If you find a lighter `zstd` implementation or crate that reduces the
  footprint, feel free to submit a PR.

- 🛑 Feature creep that increases binary size will be rejected
  to maintain the sub-1MB goal.

## 📜 License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.

<p align="center">
  <i>Developed with precision in Rust. 🦖</i>
</p>
