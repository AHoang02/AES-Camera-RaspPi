Encrypted Video Streaming System using Raspberry Pi

This project implements a secure, real-time video streaming pipeline using a Raspberry Pi 5 as the source device and a PC as the receiver. The system demonstrates end-to-end handling of sensitive data by combining embedded video capture, network transport, and applied cryptography into a modular and extensible design.

Video is captured using the Raspberry Pi Camera Module 2.1 and processed frame by frame. Each frame is encrypted before leaving the Pi, ensuring that raw video data never traverses the network in plaintext. Encryption was first implemented in AES-CBC mode and later migrated to AES-CCM, which provides both confidentiality and message authentication. The implementation supports AES key sizes of 128, 192, and 256 bits, with keys generated dynamically from entropy sources such as CPU temperature readings.

Encrypted frames are transmitted to a receiving PC over a TCP socket, with support for both Wi-Fi and Ethernet connections. The communication layer was built in Rust for reliability and efficiency, with custom framing and buffering logic to preserve video synchronization and tolerate network interruptions. On the receiving side, the stream is decrypted and displayed through a Python-based GUI, enabling real-time playback of the secure video feed.

Two encryption pipelines were implemented: a standard Rust software version and an alternate hardware-accelerated version that leverages the Raspberry Pi 5’s onboard crypto engine via OpenSSL FFI. This dual approach allowed performance benchmarking between pure software and engine-assisted encryption, providing practical insight into throughput, latency, and system resource usage.

The project highlights several engineering considerations, including secure transport of high-bandwidth data, authenticated encryption for integrity protection, and optimization of embedded resources to support real-time workloads. By balancing encryption overhead with network efficiency, the system achieves a functional demonstration of secure video communication in constrained environments.

The result is a working prototype capable of securely streaming live camera output across local networks, adaptable to multiple key sizes, transport mediums, and encryption backends. It serves as a foundation for future extensions such as secure key exchange protocols, cloud integration, or deployment on additional lightweight platforms.
