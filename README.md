# Headers Security Checker (HSC GUI)

A sleek, lightweight, and modern cross-platform desktop application designed to analyze, verify, and compare HTTP response headers to audit website security configurations. Built with **Tauri v2** and **Rust** on the backend, and vanilla **HTML5 / CSS3 / JavaScript** on the frontend.

---

## ✨ Features

- **🔍 Single URL Scan**: Perform a deep analysis of security headers on any public webpage. Includes options to follow redirects and inject custom request headers (e.g. for authentication).
- **📦 Batch URL Scan**: Concurrently analyze multiple URLs at once, saving time when auditing multiple domains.
- **📄 Raw Header Import**: Analyze response headers directly by pasting raw HTTP header text or importing them from local text files.
- **⚔️ Side-by-Side Comparison**:
  - **URL vs. URL**: Compare the header profiles of two different environments (e.g., staging vs. production).
  - **URL vs. File/Raw**: Compare a live site's headers against a reference file or raw text input.
- **📜 Scan History**: Keeps a local history log of your previous scans for quick retrieval and review.
- **📊 Export Reports**: Export security analysis reports directly to **JSON** or clean **Markdown** files.
- **🌗 Responsive Dark/Light UI**: Eye-pleasing theme toggle for day and night security auditing.

---

## 🛡️ Security Header Inspections

The application checks for and rates:
1. **Transport Security**: HSTS configuration (`Strict-Transport-Security`).
2. **CORS Policies**: Cross-Origin Resource Sharing setups.
3. **Cookie Security**: Flags missing `Secure`, `HttpOnly`, and `SameSite` options on response cookies.
4. **Vulnerabilities & Leaks**: Identifies information leaks from banners/headers (`Server`, `X-Powered-By`, etc.).
5. **Missing Best-Practice Headers**: Alerts for missing `Content-Security-Policy`, `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy`, and others.

---

## 🛠️ Technology Stack

- **Backend**: 
  - [Rust](https://www.rust-lang.org/) (Actively utilizing Tauri v2 framework)
  - `reqwest` & `tokio` for high-performance concurrent asynchronous web scanning
  - `serde` & `serde_json` for serialization
- **Frontend**:
  - Vanilla HTML5 / modern CSS3
  - Custom JavaScript (compiled statically)
- **CI/CD**:
  - GitHub Actions for automated, cross-platform build artifacts (`.dmg`, `.deb`, `.AppImage`, `.msi`, `.exe`) and GitHub Releases.

---

## 🚀 Getting Started

### 📋 Prerequisites

Before running or building the app, make sure you have the following installed on your machine:
- **Rust**: [Rustup installer](https://rustup.rs/) (Stable channel).
- **Tauri Prerequisites**: Follow the official guide to set up system packages for your operating system:
  - [Tauri Prerequisites Guide](https://v2.tauri.app/start/prerequisites/)

### 📦 Setup & Run

1. Clone the repository:
   ```bash
   git clone https://github.com/NonBytes/hsc-gui.git
   cd hsc-gui
   ```

2. Install the Tauri CLI globally (if you haven't already):
   ```bash
   cargo install tauri-cli --version "^2"
   ```

3. Run the application in development mode:
   ```bash
   cargo tauri dev
   ```

4. Build the application for production:
   ```bash
   cargo tauri build
   ```
   *The compiled installer/bundle for your system will be saved in `src-tauri/target/release/bundle/`.*

---

## 🤖 CI/CD Workflow

The project contains a pre-configured GitHub Actions workflow located in `.github/workflows/release.yml` that builds cross-platform executables and automatically attaches them to a new GitHub Release.

### How to trigger a Release:
- **Manual Trigger (Recommended)**: Go to the GitHub Actions tab, select the **Release** workflow, and click **Run workflow**. You can specify the tag version (e.g. `v0.1.1`).
- **Tag Trigger**: Push a tag starting with `v` (e.g., `v0.1.1`):
  ```bash
  git tag v0.1.1
  git push origin v0.1.1
  ```
