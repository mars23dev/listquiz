ListQuiz portable build
=======================

Run start.bat. Your browser opens at http://localhost:8080.

All state stays in the data folder next to listquiz.exe:
    data\listquiz.db
Delete that folder to reset the game to a fresh state.

No installation and no Rust/Node.js are needed. The release is a portable
folder containing the compiled server and the built frontend.

Windows SmartScreen may warn because the executable has no code-signing
certificate. Use "More info" then "Run anyway", or start the game through
start.bat.

Playing on other devices
------------------------
The server listens on port 8080 on all network interfaces. Connect phones or
tablets to the same network and open:
    http://<host-lan-ip>:8080
Allow the app through the Windows Firewall prompt, or add an inbound rule for
TCP port 8080.

Stopping
--------
Close the start.bat window, or press Ctrl+C in it.
