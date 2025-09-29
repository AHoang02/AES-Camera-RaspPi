import subprocess
import tkinter as tk
from tkinter import scrolledtext, messagebox
import threading
import time
import av
import io
from PIL import Image, ImageTk

# Custom TeeStream: wraps a binary stream, logs raw data, and passes data to PyAV.
class TeeStream(io.RawIOBase):
    def __init__(self, stream, log_callback):
        self.stream = stream
        self.log_callback = log_callback

    def read(self, size=-1):
        data = self.stream.read(size)
        if data:
            # Log a summary: hex of first 32 bytes.
            summary = data[:32].hex()
            self.log_callback(summary)
        return data

    def fileno(self):
        return self.stream.fileno()

    def readable(self):
        return True

class AESGUI(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("AES PC Side GUI")
        # Initial window size; will be updated based on video resolution.
        self.geometry("800x600")

        # Variable for key size.
        self.key_size = tk.IntVar(value=128)

        # Frame for key size selection.
        key_frame = tk.Frame(self)
        key_frame.pack(pady=10)
        tk.Label(key_frame, text="Select AES Key Size:").pack(side=tk.LEFT, padx=5)
        tk.Radiobutton(key_frame, text="128", variable=self.key_size, value=128).pack(side=tk.LEFT, padx=5)
        tk.Radiobutton(key_frame, text="192", variable=self.key_size, value=192).pack(side=tk.LEFT, padx=5)
        tk.Radiobutton(key_frame, text="256", variable=self.key_size, value=256).pack(side=tk.LEFT, padx=5)

        # Start and Stop buttons.
        button_frame = tk.Frame(self)
        button_frame.pack(pady=10)
        self.start_button = tk.Button(button_frame, text="Start", command=self.start_backend)
        self.start_button.pack(side=tk.LEFT, padx=5)
        self.stop_button = tk.Button(button_frame, text="Stop", command=self.stop_backend, state=tk.DISABLED)
        self.stop_button.pack(side=tk.LEFT, padx=5)

        # Container for video display.
        video_container_label = tk.Label(self, text="Video Display:")
        video_container_label.pack()
        # Using a Canvas for efficient image updates.
        self.video_canvas = tk.Canvas(self, bg="black")
        self.video_canvas.pack(pady=5, fill=tk.BOTH, expand=True)

        # Scrolled text widget for logging output.
        log_label = tk.Label(self, text="Log Output:")
        log_label.pack()
        self.log_text = scrolledtext.ScrolledText(self, width=100, height=15)
        self.log_text.pack(pady=10)

        self.process = None
        self.native_set = False  # Flag to set window size based on native video resolution.
        self.canvas_image = None  # Canvas image item for efficient updates.

        # Bind close window event.
        self.protocol("WM_DELETE_WINDOW", self.on_closing)

    def start_backend(self):
        key_size = self.key_size.get()
        rust_executable = "aes_backend.exe"  # Path to your Rust executable.
        try:
            self.process = subprocess.Popen(
                [rust_executable, str(key_size)],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                bufsize=0  # Unbuffered binary mode.
            )
        except Exception as e:
            messagebox.showerror("Error", f"Failed to start backend: {e}")
            return

        self.start_button.config(state=tk.DISABLED)
        self.stop_button.config(state=tk.NORMAL)

        threading.Thread(target=self.decode_and_display, daemon=True).start()
        threading.Thread(target=self.read_stderr, daemon=True).start()

    def stop_backend(self):
        """Terminates the Rust process and closes the GUI."""
        if self.process:
            self.process.terminate()
            self.process = None
        self.start_button.config(state=tk.NORMAL)
        self.stop_button.config(state=tk.DISABLED)
        self.destroy()

    def on_closing(self):
        """Ensure the backend process is terminated when the GUI is closed."""
        self.stop_backend()

    def read_stderr(self):
        """Reads stderr from the Rust process and displays it in the log text widget."""
        for line in iter(self.process.stderr.readline, b''):
            try:
                text = line.decode('utf-8')
            except UnicodeDecodeError:
                text = str(line)
            self.log_text.insert(tk.END, "ERR: " + text)
            self.log_text.see(tk.END)
        self.process.wait()
        self.log_text.insert(tk.END, f"\nProcess exited with return code {self.process.returncode}\n")
        self.start_button.config(state=tk.NORMAL)
        self.stop_button.config(state=tk.DISABLED)

    def decode_and_display(self):
        """Uses PyAV to decode the H.264 MPEG-TS stream from the Rust process stdout
           and updates the video display. Also logs raw data using a TeeStream wrapper."""
        def log_raw(summary):
            def append_log():
                self.log_text.insert(tk.END, f"Raw Data: {summary}\n")
                self.log_text.see(tk.END)
                # Keep log to at most 20 lines; delete oldest 10 lines if necessary.
                try:
                    total_lines = int(self.log_text.index('end-1c').split('.')[0])
                    if total_lines > 20:
                        self.log_text.delete("1.0", "10.0")
                except Exception:
                    pass
            self.after(0, append_log)

        try:
            tee_stream = TeeStream(self.process.stdout, log_raw)
            container = av.open(
                tee_stream,
                format='mpegts',
                mode='r',
                options={
                    'hwaccel': 'dxva2',            # Hardware acceleration option.
                    'fflags': 'nobuffer',
                    'flags': 'low_delay',
                    'probesize': '4096',           # Reduced probesize for lower latency.
                    'analyzeduration': '200000',    # 200 ms buffering.
                    'max_interleave_delta': '500000',
                    'threads': 'auto',
                    'seekable': '0'
                }
            )
        except Exception as e:
            self.log_text.insert(tk.END, f"Failed to open video stream: {e}\n")
            return

        for frame in container.decode(video=0):
            try:
                # Set native resolution and update GUI window only once.
                if not self.native_set:
                    native_width, native_height = frame.width, frame.height
                    self.native_set = True
                    new_geometry = f"{native_width}x{native_height+300}"
                    self.geometry(new_geometry)
                    self.video_canvas.config(width=native_width, height=native_height)
                # Convert the frame to a PIL Image.
                img = frame.to_image()
                photo = ImageTk.PhotoImage(img)
                # Update the canvas image efficiently.
                if self.canvas_image is None:
                    self.canvas_image = self.video_canvas.create_image(0, 0, anchor=tk.NW, image=photo)
                else:
                    self.video_canvas.itemconfig(self.canvas_image, image=photo)
                self.video_canvas.image = photo
            except Exception as e:
                self.log_text.insert(tk.END, f"Error decoding/displaying frame: {e}\n")
            time.sleep(0.01)

if __name__ == "__main__":
    app = AESGUI()
    app.mainloop()
