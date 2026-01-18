## HexViewer

HexViewer is a desktop application for viewing and editing hex values, built with Rust and egui/eframe. It provides a bit-level visualization of a 64‑bit value and a configurable field view.

### Features

- 64‑bit register view
  - Two rows of bits:
    - Top row: bits [63:32]
    - Bottom row: bits [31:0]
- Bit view
  - Clicking bit button toggles the corresponding bit between 0 and 1
- Field view
  - Toggle between Bit and Field modes using a button in the header
  - In Field mode, a configuration string (for example `4:20:8`) defines the widths of consecutive fields
  - The application slices the register from high to low bits according to the widths and draws same‑width rectangles in a single row:
    - Each rectangle shows the bit range (`[high:low]`; if width is 1, it shows `[N]`)
    - Inside the rectangle, the field value is displayed in hexadecimal and (if there is enough space) decimal
  - The total sum of field widths is limited to 32 bits; any extra configuration is ignored
- Support always‑on‑top

### Build and Run

#### 1. Development run

```bash
cargo run
```

#### 2. Release build

```bash
cargo build --release
```