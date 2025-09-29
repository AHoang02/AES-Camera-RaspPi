Below is a comprehensive list of everything you’ll need to install on a new system to run this project (both the Rust backend and the Python GUI). This list assumes you’re targeting Windows (since we reference DXVA2 and Chocolatey), but similar principles apply on other platforms with appropriate alternatives.

---

### **System-Level Dependencies**

- **Operating System:**  
  – Windows 10 (or later) is assumed for DXVA2 hardware acceleration.

- **FFmpeg:**  
  – A build of FFmpeg that supports hardware acceleration (DXVA2 on Windows).  
  – **Installation Options:**  
  • Using Chocolatey:  
    `choco install ffmpeg`  
  • Or download a prebuilt FFmpeg package from [ffmpeg.org](https://ffmpeg.org/download.html) and add it to your system PATH.

- **Build Tools for Windows:**  
  – Visual Studio Build Tools (for compiling Rust code, if not already installed).  
  – Install via Chocolatey:  
  `choco install visualstudio2019buildtools` (or a similar package)  
  – Alternatively, install the official [Build Tools for Visual Studio](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2019).

- **Chocolatey (Optional):**  
  – A Windows package manager that simplifies installing system packages.  
  – Install from [chocolatey.org](https://chocolatey.org/install).

---

### **Rust Environment**

- **Rust Toolchain:**  
  – Install using [rustup](https://rustup.rs/).  
  – This includes the Rust compiler (`rustc`) and the package manager (`cargo`).

- **Project Crates (dependencies):**  
  Ensure your project’s `Cargo.toml` includes the following crates (versions can be set to the latest stable releases or those that work with your code):  
  - `aes`  
  - `ctr`  
  - `hkdf`  
  - `sha2`  
  - `x25519-dalek`  
  - `hex`  
  - `rand`  
  – Run `cargo build --release` to compile your Rust executable.

---

### **Python Environment**

- **Python Interpreter:**  
  – Install Python 3.8 or later.  
  – **Installation Options:**  
  • Download from [python.org](https://www.python.org/downloads/).  
  • Or use Chocolatey:  
    `choco install python`

- **pip:**  
  – Comes with modern Python installations. Verify with `pip --version`.

- **Python Packages:**  
  Install via pip:  
  - **PyAV:**  
    ```bash
    pip install av
    ```  
    *Note:* PyAV relies on FFmpeg libraries. The prebuilt wheels for Windows typically include FFmpeg support, but having FFmpeg installed system‑wide (and in your PATH) can be beneficial for hardware acceleration.  
  - **Pillow:**  
    ```bash
    pip install Pillow
    ```  
  - **NumPy:**  
    ```bash
    pip install numpy
    ```  
    *Note:* NumPy is often installed automatically as a dependency but list it explicitly if needed.

- **tkinter:**  
  – Usually included with standard Python installations on Windows. If missing, install via your system’s package manager.

---

### **Additional Considerations**

- **Hardware Acceleration:**  
  – Your Python GUI uses the option `'hwaccel': 'dxva2'` via PyAV. Ensure your system’s FFmpeg build and GPU drivers support DXVA2.  
  – If your system does not support DXVA2, you may need to remove or change this option.

- **Environment Variables:**  
  – Make sure that the installed FFmpeg binaries (if installed separately) are added to your system PATH so that PyAV can access them.

- **Packaging the Python App:**  
  – To compile your Python GUI into an executable, you can use [PyInstaller](https://www.pyinstaller.org/).  
    Install via pip:  
    ```bash
    pip install pyinstaller
    ```  
    Then run:  
    ```bash
    pyinstaller --onefile --noconsole your_python_gui_script.py
    ```

---

### **Summary**

To run your project on another system, ensure you have:
- Windows with up-to-date GPU drivers.
- FFmpeg (with DXVA2 support) installed (via Chocolatey or direct download).
- Visual Studio Build Tools (for compiling Rust).
- Rust installed via rustup (including cargo) with the required crates.
- Python 3.8+ installed along with pip.
- Python packages: PyAV, Pillow, and NumPy.
- Optionally, Chocolatey for simplified installation of system packages.
- If packaging the Python app, PyInstaller installed.

This list covers all the dependencies for both building and running your Rust and Python application. Let me know if you need further details or clarification on any of these points!
